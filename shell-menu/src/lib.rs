//! Windows 11 sağ tık ana menüsündeki "Blazon ile ikonu değiştir" komutu.
//!
//! Komut, `package/AppxManifest.xml` ile kaydedilen COM sınıfıdır. Explorer DLL'i bir vekil
//! süreçte (dllhost.exe) yükler; tıklanınca DLL ile aynı klasördeki `blazon.exe` başlatılır.
#![cfg(windows)]

use std::cell::RefCell;
use std::ffi::{OsString, c_void};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use std::process::Command;

use windows::Win32::Foundation::{
    CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_NOTIMPL, E_POINTER, HMODULE,
    S_FALSE,
};
use windows::Win32::System::Com::{
    CLSCTX_LOCAL_SERVER, CoCreateInstance, CoTaskMemFree, IBindCtx, IClassFactory,
    IClassFactory_Impl, IDispatch, IServiceProvider,
};
use windows::Win32::System::LibraryLoader::{
    GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
    GetModuleFileNameW, GetModuleHandleExW,
};
use windows::Win32::System::Ole::{IObjectWithSite, IObjectWithSite_Impl};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Shell::{
    CSIDL_DESKTOP, ECF_DEFAULT, ECS_ENABLED, ECS_HIDDEN, IEnumExplorerCommand, IExplorerCommand,
    IExplorerCommand_Impl, IFolderView, IShellBrowser, IShellDispatch2, IShellFolderViewDual,
    IShellItem, IShellItemArray, IShellWindows, SHStrDupW, SID_STopLevelBrowser, SIGDN_FILESYSPATH,
    SVGIO_BACKGROUND, SWC_DESKTOP, SWFO_NEEDDISPATCH, ShellWindows,
};
use windows::Win32::UI::WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow, SW_SHOWNORMAL};
use windows_core::{
    BOOL, BSTR, GUID, HRESULT, HSTRING, IUnknown, Interface, PCWSTR, PWSTR, Ref, Result, implement,
};

/// AppxManifest.xml'deki `Clsid` ile aynı olmalı.
pub const CLSID_BLAZON_COMMAND: GUID = GUID::from_u128(0x7C1F6C0B_5E0A_4B43_9F0D_3A6B52D8E914);

const TITLE: &str = "Blazon ile ikonu değiştir";
const EXE: &str = "blazon.exe";
const ICON: &str = "blazon-menu.ico";

/// Bu DLL'in bulunduğu klasördeki `blazon.exe`.
fn blazon_exe() -> Option<PathBuf> {
    let mut module = HMODULE::default();
    let mut buf = vec![0u16; 32 * 1024];
    let len = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            PCWSTR(blazon_exe as *const () as *const u16),
            &mut module,
        )
        .ok()?;
        GetModuleFileNameW(Some(module), &mut buf) as usize
    };
    if len == 0 || len == buf.len() {
        return None;
    }
    let dll = PathBuf::from(OsString::from_wide(&buf[..len]));
    Some(dll.parent()?.join(EXE))
}

/// Öğenin dosya sistemi yolu; sanal öğelerde (ör. Bu bilgisayar) yoktur.
fn item_path(item: &IShellItem) -> Option<PathBuf> {
    unsafe {
        let raw = item.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
        let path = OsString::from_wide(raw.as_wide());
        CoTaskMemFree(Some(raw.0 as *const c_void));
        Some(PathBuf::from(path))
    }
}

/// Tek klasör seçiliyse onu, seçim yoksa (boş alana sağ tık) görünümde açık olan klasörü verir.
fn target_folder(items: Option<&IShellItemArray>, site: Option<&IUnknown>) -> Option<PathBuf> {
    unsafe {
        let count = match items {
            Some(items) => items.GetCount().ok()?,
            None => 0,
        };
        let item: IShellItem = match (count, items) {
            (1, Some(items)) => items.GetItemAt(0).ok()?,
            (0, _) => {
                let provider: IServiceProvider = site?.cast().ok()?;
                let view: IFolderView = provider.QueryService(&IFolderView::IID).ok()?;
                view.GetFolder().ok()?
            }
            _ => return None,
        };
        item_path(&item)
    }
}

/// Tırnaklı klasör argümanı. Windows yolları tırnak içeremez; yalnızca sondaki `\` (ör. `D:\`)
/// kapanış tırnağını bozmasın diye ikilenir.
fn quoted(folder: &Path) -> OsString {
    let slash = u16::from(b'\\');
    let wide: Vec<u16> = folder.as_os_str().encode_wide().collect();
    let trailing = wide.iter().rev().take_while(|c| **c == slash).count();

    let mut arg = OsString::from("\"");
    arg.push(folder);
    arg.push("\\".repeat(trailing));
    arg.push("\"");
    arg
}

/// Blazon'u Explorer üzerinden başlatır. Doğrudan başlatılırsa vekil sürecin paket kimliğini
/// devralır ve görev çubuğunda Başlat menüsündeki Blazon'dan ayrı bir uygulama sayılabilir.
fn launch_from_explorer(exe: &Path, folder: &Path) -> Result<()> {
    unsafe {
        let windows: IShellWindows = CoCreateInstance(&ShellWindows, None, CLSCTX_LOCAL_SERVER)?;
        let mut hwnd = 0;
        let desktop = windows.FindWindowSW(
            &VARIANT::from(CSIDL_DESKTOP as i32),
            &VARIANT::default(),
            SWC_DESKTOP,
            &mut hwnd,
            SWFO_NEEDDISPATCH,
        )?;
        let browser: IShellBrowser = desktop
            .cast::<IServiceProvider>()?
            .QueryService(&SID_STopLevelBrowser)?;
        let background: IDispatch = browser
            .QueryActiveShellView()?
            .GetItemObject(SVGIO_BACKGROUND)?;
        let shell: IShellDispatch2 = background
            .cast::<IShellFolderViewDual>()?
            .Application()?
            .cast()?;
        shell.ShellExecute(
            &BSTR::from(exe.to_string_lossy().as_ref()),
            &VARIANT::from(BSTR::from_wide(
                &quoted(folder).encode_wide().collect::<Vec<u16>>(),
            )),
            &VARIANT::from(BSTR::from(
                exe.parent().unwrap_or(exe).to_string_lossy().as_ref(),
            )),
            &VARIANT::default(),
            &VARIANT::from(SW_SHOWNORMAL.0),
        )
    }
}

fn launch(exe: &Path, folder: &Path) -> Result<()> {
    // Tıklamayla gelen ön plan iznini devret; yoksa pencere Explorer'ın arkasında açılır.
    let _ = unsafe { AllowSetForegroundWindow(ASFW_ANY) };
    launch_from_explorer(exe, folder).or_else(|_| {
        // Masaüstü kabuğu yoksa (ör. farklı bir kabuk kullanılıyorsa) doğrudan başlat.
        Command::new(exe).arg(folder).spawn()?;
        Ok(())
    })
}

#[implement(IExplorerCommand, IObjectWithSite)]
#[derive(Default)]
struct BlazonCommand {
    site: RefCell<Option<IUnknown>>,
}

impl BlazonCommand {
    fn target(&self, items: Ref<IShellItemArray>) -> Option<PathBuf> {
        // Kabuk çağrıları sırasında SetSite gelebilir; ödünç almayı açık bırakma.
        let site = self.site.borrow().clone();
        target_folder(items.as_ref(), site.as_ref())
    }
}

impl IObjectWithSite_Impl for BlazonCommand_Impl {
    fn SetSite(&self, site: Ref<IUnknown>) -> Result<()> {
        let old = self.site.replace(site.cloned());
        drop(old);
        Ok(())
    }

    fn GetSite(&self, iid: *const GUID, out: *mut *mut c_void) -> Result<()> {
        if iid.is_null() || out.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { *out = std::ptr::null_mut() };
        let site = self.site.borrow().clone().ok_or(E_FAIL)?;
        unsafe { site.query(iid, out).ok() }
    }
}

impl IExplorerCommand_Impl for BlazonCommand_Impl {
    fn GetTitle(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        unsafe { SHStrDupW(&HSTRING::from(TITLE)) }
    }

    fn GetIcon(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        let exe = blazon_exe().ok_or(E_FAIL)?;
        let ico = exe.with_file_name(ICON);
        let icon = if ico.is_file() {
            ico.into_os_string()
        } else {
            let mut icon = exe.into_os_string();
            icon.push(",0");
            icon
        };
        unsafe { SHStrDupW(&HSTRING::from(icon)) }
    }

    fn GetToolTip(&self, _items: Ref<IShellItemArray>) -> Result<PWSTR> {
        Err(E_NOTIMPL.into())
    }

    fn GetCanonicalName(&self) -> Result<GUID> {
        Ok(CLSID_BLAZON_COMMAND)
    }

    fn GetState(&self, items: Ref<IShellItemArray>, _ok_to_be_slow: BOOL) -> Result<u32> {
        let usable = self.target(items).is_some() && blazon_exe().is_some_and(|exe| exe.is_file());
        Ok(if usable { ECS_ENABLED.0 } else { ECS_HIDDEN.0 } as u32)
    }

    fn Invoke(&self, items: Ref<IShellItemArray>, _ctx: Ref<IBindCtx>) -> Result<()> {
        let folder = self.target(items).ok_or(E_FAIL)?;
        let exe = blazon_exe().ok_or(E_FAIL)?;
        launch(&exe, &folder)
    }

    fn GetFlags(&self) -> Result<u32> {
        Ok(ECF_DEFAULT.0 as u32)
    }

    fn EnumSubCommands(&self) -> Result<IEnumExplorerCommand> {
        Err(E_NOTIMPL.into())
    }
}

#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(
        &self,
        outer: Ref<IUnknown>,
        iid: *const GUID,
        out: *mut *mut c_void,
    ) -> Result<()> {
        if iid.is_null() || out.is_null() {
            return Err(E_POINTER.into());
        }
        unsafe { *out = std::ptr::null_mut() };
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let command: IExplorerCommand = BlazonCommand::default().into();
        unsafe { command.query(iid, out).ok() }
    }

    fn LockServer(&self, _lock: BOOL) -> Result<()> {
        Ok(())
    }
}

/// COM giriş noktası.
///
/// # Safety
/// COM tarafından geçerli işaretçilerle çağrılır.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllGetClassObject(
    clsid: *const GUID,
    iid: *const GUID,
    out: *mut *mut c_void,
) -> HRESULT {
    if clsid.is_null() || iid.is_null() || out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = std::ptr::null_mut();
        if *clsid != CLSID_BLAZON_COMMAND {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = Factory.into();
        factory.query(iid, out)
    }
}

/// DLL, vekil süreç kapanana kadar yüklü kalır.
#[unsafe(no_mangle)]
pub extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_spaces_and_unicode() {
        assert_eq!(
            quoted(Path::new(r"C:\Yeni klasör\ığüş")),
            r#""C:\Yeni klasör\ığüş""#
        );
    }

    #[test]
    fn doubles_trailing_backslash_for_drive_root() {
        assert_eq!(quoted(Path::new(r"D:\")), r#""D:\\""#);
    }

    #[test]
    fn creates_command_through_class_factory() {
        unsafe {
            let mut out = std::ptr::null_mut();
            DllGetClassObject(&CLSID_BLAZON_COMMAND, &IClassFactory::IID, &mut out).unwrap();
            let factory = IClassFactory::from_raw(out);
            let command: IExplorerCommand = factory.CreateInstance(None).unwrap();
            let title = command.GetTitle(None).unwrap();
            assert_eq!(title.to_string().unwrap(), TITLE);
            CoTaskMemFree(Some(title.0 as *const c_void));
            assert_eq!(command.GetState(None, false).unwrap(), ECS_HIDDEN.0 as u32);

            let other = GUID::zeroed();
            let mut out = std::ptr::null_mut();
            let hr = DllGetClassObject(&other, &IClassFactory::IID, &mut out);
            assert_eq!(hr, CLASS_E_CLASSNOTAVAILABLE);
        }
    }
}
