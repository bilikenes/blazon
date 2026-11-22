//! Windows 11 ana sağ tık menüsünü açıp kapatır (isteğe bağlı; varsayılan klasik menüdür).
//!
//! Ana menüye yalnızca paket kimliği olan komutlar girebilir. Açınca iki şey olur:
//!  1. Blazon'un sertifikası yerel makinenin "Güvenilen Kişiler" deposuna eklenir (yönetici onayı).
//!  2. İmzalı menü paketi, kurulum klasörünü göstererek bu kullanıcıya kaydedilir.
//!
//! Sertifika yalnızca bu paketi imzalamak için üretildi, gizli anahtarı imzadan sonra silindi
//! (bkz. `shell-menu/package/build-package.ps1`). Kurulum programı ve uygulama aynı kodu kullanır:
//! `blazon.exe --menu status | cert-status | enable | disable | disable-keep-cert | sync`.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::{null, null_mut};

use serde::Serialize;
use windows::Foundation::Uri;
use windows::Management::Deployment::{AddPackageOptions, DeploymentResult, PackageManager};
use windows::core::HSTRING;
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_CANCELLED, ERROR_SUCCESS, GetLastError, WAIT_OBJECT_0,
};
use windows_sys::Win32::Security::Cryptography::{
    CERT_FIND_EXISTING, CERT_STORE_ADD_REPLACE_EXISTING, CERT_STORE_OPEN_EXISTING_FLAG,
    CERT_STORE_PROV_SYSTEM_W, CERT_STORE_READONLY_FLAG, CERT_SYSTEM_STORE_LOCAL_MACHINE,
    CertAddEncodedCertificateToStore, CertCloseStore, CertCreateCertificateContext,
    CertDeleteCertificateFromStore, CertFindCertificateInStore, CertFreeCertificateContext,
    CertOpenStore, HCERTSTORE, X509_ASN_ENCODING,
};
use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoUninitialize};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey,
    RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};
use windows_sys::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SW_HIDE};

/// `shell-menu/package/AppxManifest.xml` içindeki Identity ile aynı olmalı.
const PACKAGE_NAME: &str = "EnesBilik.Blazon";
const PACKAGE_PUBLISHER: &str = "CN=Enes Bilik";

/// Yönetici olarak çalışan yardımcı, sertifikayı diskten değil exe'nin içinden alır.
const CERTIFICATE: &[u8] = include_bytes!("../../shell-menu/package/signed/blazon-menu.cer");

const CLASSIC_KEYS: [&str; 2] = [
    r"Software\Classes\Directory\shell\Blazon",
    r"Software\Classes\Directory\Background\shell\Blazon",
];
const WINDOWS_11_BUILD: u32 = 22000;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MenuStatus {
    /// Windows 10 ya da kurulu olmayan (geliştirme) kopya: seçenek gösterilmez.
    Unavailable,
    Off,
    On,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum MenuError {
    /// Kullanıcı yönetici onayını reddetti.
    Cancelled,
    Unavailable,
    /// Sertifika eklenemedi (ör. kurum politikası).
    Trust,
    /// Paket kaydedilemedi ya da kaldırılamadı; `hresult` Windows'un verdiği kod.
    Package {
        hresult: i32,
    },
}

type Result<T> = std::result::Result<T, MenuError>;

impl From<windows::core::Error> for MenuError {
    fn from(e: windows::core::Error) -> Self {
        Self::Package {
            hresult: e.code().0,
        }
    }
}

fn wide(s: impl AsRef<OsStr>) -> Vec<u16> {
    s.as_ref().encode_wide().chain([0]).collect()
}

// ---- Kurulum dosyaları ------------------------------------------------------------------

struct Files {
    dir: PathBuf,
    package: PathBuf,
}

/// Menü dosyaları blazon.exe'nin yanındaysa yollarını verir.
fn files() -> Option<Files> {
    let dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let package = dir.join("blazon-menu.msix");
    (package.is_file() && dir.join("blazon_shell_menu.dll").is_file())
        .then_some(Files { dir, package })
}

fn windows_build() -> u32 {
    let key = wide(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let name = wide("CurrentBuildNumber");
    let mut buf = [0u16; 16];
    let mut size = size_of_val(&buf) as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            null_mut(),
            buf.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != ERROR_SUCCESS {
        return 0;
    }
    let len = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len]).parse().unwrap_or(0)
}

// ---- Paket ------------------------------------------------------------------------------

/// Bu kullanıcıya kayıtlı menü paketlerinin tam adları.
fn registered() -> windows::core::Result<Vec<HSTRING>> {
    let packages = PackageManager::new()?.FindPackagesByUserSecurityIdNamePublisher(
        &HSTRING::new(),
        &HSTRING::from(PACKAGE_NAME),
        &HSTRING::from(PACKAGE_PUBLISHER),
    )?;
    packages.into_iter().map(|p| p.Id()?.FullName()).collect()
}

fn check(result: DeploymentResult) -> windows::core::Result<()> {
    result.ExtendedErrorCode()?.ok()
}

fn register(files: &Files) -> windows::core::Result<()> {
    let options = AddPackageOptions::new()?;
    options.SetExternalLocationUri(&Uri::CreateUri(&HSTRING::from(files.dir.as_os_str()))?)?;
    let package = Uri::CreateUri(&HSTRING::from(files.package.as_os_str()))?;
    check(
        PackageManager::new()?
            .AddPackageByUriAsync(&package, &options)?
            .join()?,
    )
}

fn unregister() -> windows::core::Result<()> {
    let manager = PackageManager::new()?;
    for name in registered()? {
        check(manager.RemovePackageAsync(&name)?.join()?)?;
    }
    Ok(())
}

// ---- Sertifika --------------------------------------------------------------------------

/// "Güvenilen Kişiler" deposunu açar, `f` bitince kapatır.
fn with_store<T>(flags: u32, f: impl FnOnce(HCERTSTORE) -> T) -> Option<T> {
    let name = wide("TrustedPeople");
    let store = unsafe {
        CertOpenStore(
            CERT_STORE_PROV_SYSTEM_W,
            0,
            0,
            CERT_SYSTEM_STORE_LOCAL_MACHINE | flags,
            name.as_ptr().cast(),
        )
    };
    if store.is_null() {
        return None;
    }
    let out = f(store);
    unsafe { CertCloseStore(store, 0) };
    Some(out)
}

fn certificate_trusted() -> bool {
    let flags = CERT_STORE_READONLY_FLAG | CERT_STORE_OPEN_EXISTING_FLAG;
    with_store(flags, |store| unsafe {
        let cert = CertCreateCertificateContext(
            X509_ASN_ENCODING,
            CERTIFICATE.as_ptr(),
            CERTIFICATE.len() as u32,
        );
        if cert.is_null() {
            return false;
        }
        let found = CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING,
            0,
            CERT_FIND_EXISTING,
            cert.cast(),
            null(),
        );
        CertFreeCertificateContext(cert);
        if found.is_null() {
            return false;
        }
        CertFreeCertificateContext(found);
        true
    })
    .unwrap_or(false)
}

/// Yönetici olarak çalışan yardımcıda çağrılır.
fn trust_certificate() -> bool {
    with_store(0, |store| unsafe {
        CertAddEncodedCertificateToStore(
            store,
            X509_ASN_ENCODING,
            CERTIFICATE.as_ptr(),
            CERTIFICATE.len() as u32,
            CERT_STORE_ADD_REPLACE_EXISTING,
            null_mut(),
        ) != 0
    })
    .unwrap_or(false)
}

/// Yönetici olarak çalışan yardımcıda çağrılır.
fn untrust_certificate() -> bool {
    with_store(0, |store| unsafe {
        let cert = CertCreateCertificateContext(
            X509_ASN_ENCODING,
            CERTIFICATE.as_ptr(),
            CERTIFICATE.len() as u32,
        );
        if cert.is_null() {
            return false;
        }
        let found = CertFindCertificateInStore(
            store,
            X509_ASN_ENCODING,
            0,
            CERT_FIND_EXISTING,
            cert.cast(),
            null(),
        );
        CertFreeCertificateContext(cert);
        // CertDeleteCertificateFromStore bağlamı her durumda serbest bırakır.
        found.is_null() || CertDeleteCertificateFromStore(found) != 0
    })
    .unwrap_or(false)
}

/// blazon.exe'yi yönetici olarak `--menu <action>` ile çalıştırıp bitmesini bekler.
fn run_elevated(action: &str) -> Result<()> {
    let exe = std::env::current_exe().map_err(|_| MenuError::Trust)?;
    let (verb, exe, params) = (wide("runas"), wide(exe), wide(format!("--menu {action}")));
    unsafe {
        // UAC istemi için COM'un hazır olması önerilir; zaten farklı kipte hazırsa dokunma.
        let com = CoInitializeEx(null(), COINIT_APARTMENTTHREADED as u32) >= 0;
        let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
        info.cbSize = size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS;
        info.hwnd = GetForegroundWindow();
        info.lpVerb = verb.as_ptr();
        info.lpFile = exe.as_ptr();
        info.lpParameters = params.as_ptr();
        info.nShow = SW_HIDE;

        let result = if ShellExecuteExW(&mut info) == 0 {
            Err(if GetLastError() == ERROR_CANCELLED {
                MenuError::Cancelled
            } else {
                MenuError::Trust
            })
        } else {
            let mut code = 1u32;
            let done = WaitForSingleObject(info.hProcess, INFINITE) == WAIT_OBJECT_0
                && GetExitCodeProcess(info.hProcess, &mut code) != 0;
            CloseHandle(info.hProcess);
            if done && code == 0 {
                Ok(())
            } else {
                Err(MenuError::Trust)
            }
        };
        if com {
            CoUninitialize();
        }
        result
    }
}

// ---- Klasik menü ------------------------------------------------------------------------

/// Ana menü açıkken klasik kayıtlar gizlenir, kapalıyken yeniden görünür.
fn hide_classic(hide: bool) {
    let name = wide("LegacyDisable");
    for path in CLASSIC_KEYS {
        let path = wide(path);
        let mut key: HKEY = null_mut();
        unsafe {
            if RegOpenKeyExW(HKEY_CURRENT_USER, path.as_ptr(), 0, KEY_SET_VALUE, &mut key)
                != ERROR_SUCCESS
            {
                continue;
            }
            if hide {
                let empty = [0u16];
                RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, empty.as_ptr().cast(), 2);
            } else {
                RegDeleteValueW(key, name.as_ptr());
            }
            RegCloseKey(key);
        }
    }
}

// ---- Dışa açılan işlemler ---------------------------------------------------------------

pub fn status() -> MenuStatus {
    if windows_build() < WINDOWS_11_BUILD || files().is_none() {
        return MenuStatus::Unavailable;
    }
    match registered() {
        Ok(list) if !list.is_empty() => MenuStatus::On,
        _ => MenuStatus::Off,
    }
}

pub fn enable() -> Result<()> {
    if windows_build() < WINDOWS_11_BUILD {
        return Err(MenuError::Unavailable);
    }
    let files = files().ok_or(MenuError::Unavailable)?;
    if registered()?.is_empty() {
        if !certificate_trusted() {
            run_elevated("trust")?;
            if !certificate_trusted() {
                return Err(MenuError::Trust);
            }
        }
        register(&files)?;
    }
    hide_classic(true);
    Ok(())
}

/// Menüyü kapatır. Sertifikayı kaldırmak yönetici onayı ister; reddedilirse menü yine kapanır,
/// sertifika kalır ve `Cancelled` döner.
pub fn disable(remove_certificate: bool) -> Result<()> {
    unregister()?;
    hide_classic(false);
    if remove_certificate && certificate_trusted() {
        run_elevated("untrust")?;
    }
    Ok(())
}

/// Kurulum/güncelleme klasik kayıtları yeniden yazdıktan sonra görünürlüklerini eşitler.
fn sync() -> Result<()> {
    hide_classic(!registered()?.is_empty());
    Ok(())
}

/// `--menu <işlem>` ile çalıştırıldıysa işlemi yapar ve çıkış kodunu verir.
pub fn run_cli(args: &[String]) -> Option<i32> {
    if args.get(1).map(String::as_str) != Some("--menu") {
        return None;
    }
    let result = match args.get(2).map(String::as_str) {
        // Kurulum programı sorar: 0 açık, 1 kapalı, 3 kullanılamaz.
        Some("status") => {
            return Some(match status() {
                MenuStatus::On => 0,
                MenuStatus::Off => 1,
                MenuStatus::Unavailable => 3,
            });
        }
        Some("cert-status") => return Some(if certificate_trusted() { 0 } else { 1 }),
        Some("enable") => enable(),
        Some("disable") => disable(true),
        Some("disable-keep-cert") => disable(false),
        Some("sync") => sync(),
        Some("trust") => trust_certificate().then_some(()).ok_or(MenuError::Trust),
        Some("untrust") => untrust_certificate().then_some(()).ok_or(MenuError::Trust),
        _ => return Some(2),
    };
    Some(match result {
        Ok(()) => 0,
        Err(MenuError::Cancelled) => ERROR_CANCELLED as i32,
        Err(MenuError::Unavailable) => 3,
        Err(MenuError::Trust) => 4,
        Err(MenuError::Package { hresult }) => hresult,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn cli_ignores_normal_launches() {
        assert_eq!(run_cli(&args(&["blazon.exe"])), None);
        assert_eq!(run_cli(&args(&["blazon.exe", r"C:\Klasör"])), None);
        // Klasörden sonra gelen bayrak komut sayılmaz.
        assert_eq!(run_cli(&args(&["blazon.exe", r"C:\x", "--menu"])), None);
    }

    #[test]
    fn cli_rejects_unknown_actions() {
        assert_eq!(run_cli(&args(&["blazon.exe", "--menu"])), Some(2));
        assert_eq!(run_cli(&args(&["blazon.exe", "--menu", "x"])), Some(2));
    }

    #[test]
    fn embedded_certificate_is_readable() {
        unsafe {
            let cert = CertCreateCertificateContext(
                X509_ASN_ENCODING,
                CERTIFICATE.as_ptr(),
                CERTIFICATE.len() as u32,
            );
            assert!(!cert.is_null());
            CertFreeCertificateContext(cert);
        }
    }

    #[test]
    fn identity_matches_manifest() {
        let manifest = include_str!("../../shell-menu/package/AppxManifest.xml");
        let identity =
            format!("<Identity Name=\"{PACKAGE_NAME}\" Publisher=\"{PACKAGE_PUBLISHER}\"");
        assert!(manifest.contains(&identity));
    }

    #[test]
    fn development_copy_has_no_option() {
        // Test exe'sinin yanında menü dosyaları yoktur.
        assert_eq!(status(), MenuStatus::Unavailable);
        assert_eq!(enable(), Err(MenuError::Unavailable));
    }

    #[test]
    fn package_lookup_and_build_work() {
        assert!(windows_build() > 10_000);
        assert!(registered().is_ok());
    }
}
