//! Windows klasör ikonu işlemleri.
//!
//! Nasıl çalışır:
//! 1. Arayüzden gelen PNG (256x256) çok boyutlu bir `.ico` dosyasına çevrilir.
//! 2. `.ico` klasörün İÇİNE gizli dosya olarak yazılır ve göreli yolla bağlanır.
//!    Böylece klasör taşınsa/kopyalansa da ikon korunur.
//! 3. Bağlama işini Windows'un kendi API'si `SHGetSetFolderCustomSettings` yapar
//!    (Özellikler > Özelleştir menüsünün kullandığı API). desktop.ini'yi elle yazmak
//!    Explorer'ın desktop.ini önbelleğini tazelemediği için yeniden uygulamada
//!    ikon varsayılana düşüyordu; API bu önbelleği kendisi geçersiz kılar.
//! 4. Explorer'a SHChangeNotify ile "bu klasör değişti" denir ve işlemesi beklenir.
//!
//! Explorer'ın ikon önbelleği takılmasın diye her uygulamada dosya adı benzersiz
//! olur (`.folderstudio-<zaman>.ico`) ve eskileri silinir.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use windows_sys::Win32::UI::Shell::{
    FCS_FORCEWRITE, FCSM_ICONFILE, SHCNE_UPDATEDIR, SHCNE_UPDATEITEM, SHCNF_FLUSH, SHCNF_PATHW,
    SHChangeNotify, SHFOLDERCUSTOMSETTINGS, SHGetSetFolderCustomSettings,
};

const ICON_PREFIX: &str = ".folderstudio-";
const ICON_SIZES: [u32; 9] = [16, 20, 24, 32, 40, 48, 64, 128, 256];
const SECTION: &str = "[.ShellClassInfo]";

const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;

/// Klasöre verilen PNG'yi ikon olarak uygular.
pub fn apply(folder: &Path, png: &[u8]) -> Result<(), String> {
    ensure_folder(folder)?;
    let ico = build_ico(png)?;

    let icon_name = format!("{ICON_PREFIX}{}.ico", unix_millis());
    write_hidden_file(&folder.join(&icon_name), &ico)?;
    if let Err(e) = set_folder_icon(folder, Some(&icon_name)) {
        let _ = fs::remove_file(folder.join(&icon_name));
        return Err(e);
    }

    remove_our_icons(folder, Some(&icon_name));
    notify_explorer(folder);
    Ok(())
}

/// Klasörü varsayılan ikona döndürür. desktop.ini'deki diğer ayarlara dokunmaz.
pub fn reset(folder: &Path) -> Result<(), String> {
    ensure_folder(folder)?;
    let ini_path = folder.join("desktop.ini");
    if ini_path.exists() {
        // Önce API ile ikonu kaldır (Explorer önbelleği de tazelenir).
        // API, desktop.ini'de başka ayar kalmadıysa dosyayı kendisi siler.
        set_folder_icon(folder, None)?;
        if ini_path.exists() {
            let current = read_desktop_ini(folder);
            let edited = edit_ini(&current, None);
            if !ini_has_content(&edited) {
                fs::remove_file(&ini_path).map_err(|e| format!("desktop.ini silinemedi: {e}"))?;
            } else if edited.trim() != current.trim() {
                write_desktop_ini(folder, &edited)?;
            }
        }
    }
    // Başka özelleştirme kalmadıysa klasörün ReadOnly işaretini de kaldır.
    if !ini_path.exists() {
        set_readonly(folder, false)?;
    }
    remove_our_icons(folder, None);
    notify_explorer(folder);
    Ok(())
}

/// Windows'un klasör özelleştirme API'si. `icon` None ise ikon ayarı kaldırılır.
fn set_folder_icon(folder: &Path, icon: Option<&str>) -> Result<(), String> {
    let mut icon_w: Option<Vec<u16>> = icon.map(|n| n.encode_utf16().chain(Some(0)).collect());
    let folder_w = wide(folder);

    let mut settings: SHFOLDERCUSTOMSETTINGS = unsafe { std::mem::zeroed() };
    settings.dwSize = size_of::<SHFOLDERCUSTOMSETTINGS>() as u32;
    settings.dwMask = FCSM_ICONFILE;
    settings.pszIconFile = icon_w.as_mut().map_or(std::ptr::null_mut(), |v| v.as_mut_ptr());
    settings.iIconIndex = 0;

    let hr = unsafe { SHGetSetFolderCustomSettings(&mut settings, folder_w.as_ptr(), FCS_FORCEWRITE) };
    if hr < 0 {
        return Err(format!("Windows klasör ayarını kabul etmedi (HRESULT {hr:#010x})"));
    }
    Ok(())
}

fn ensure_folder(folder: &Path) -> Result<(), String> {
    if !folder.is_dir() {
        return Err("Seçilen yol bir klasör değil.".into());
    }
    if folder.parent().is_none() {
        return Err("Sürücü köküne ikon atanamaz.".into());
    }
    Ok(())
}

// ---------- ICO üretimi ----------

fn build_ico(png: &[u8]) -> Result<Vec<u8>, String> {
    use image::imageops::{self, FilterType};

    let src = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map_err(|e| format!("PNG okunamadı: {e}"))?
        .to_rgba8();
    if src.width() != src.height() {
        return Err("İkon görseli kare olmalı.".into());
    }

    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
    for size in ICON_SIZES {
        let frame = if src.width() == size {
            src.clone()
        } else {
            imageops::resize(&src, size, size, FilterType::Lanczos3)
        };
        let image = ico::IconImage::from_rgba_data(size, size, frame.into_raw());
        let entry = ico::IconDirEntry::encode(&image).map_err(|e| format!("ICO hatası: {e}"))?;
        dir.add_entry(entry);
    }

    let mut out = Vec::new();
    dir.write(&mut out).map_err(|e| format!("ICO yazılamadı: {e}"))?;
    Ok(out)
}

// ---------- desktop.ini ----------

fn read_desktop_ini(folder: &Path) -> String {
    let Ok(bytes) = fs::read(folder.join("desktop.ini")) else {
        return String::new();
    };
    if let [0xFF, 0xFE, rest @ ..] = bytes.as_slice() {
        let units: Vec<u16> = rest
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        return String::from_utf16_lossy(&units);
    }
    let text = String::from_utf8_lossy(&bytes);
    text.trim_start_matches('\u{FEFF}').to_string()
}

/// desktop.ini'yi UTF-16LE (BOM'lu) yazar; Türkçe karakterli yollar için gerekli.
fn write_desktop_ini(folder: &Path, content: &str) -> Result<(), String> {
    let mut bytes = vec![0xFF, 0xFE];
    for unit in content.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    write_hidden_file(&folder.join("desktop.ini"), &bytes)
}

/// [.ShellClassInfo] bölümündeki ikon satırlarını kaldırır, `icon` verildiyse yenisini ekler.
/// Diğer bölümler ve satırlar olduğu gibi kalır.
fn edit_ini(content: &str, icon: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut in_section = false;
    let mut found_section = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed.eq_ignore_ascii_case(SECTION);
            lines.push(line.to_string());
            if in_section {
                found_section = true;
                if let Some(name) = icon {
                    lines.push(format!("IconResource={name},0"));
                }
            }
            continue;
        }
        if in_section {
            let key = trimmed.split('=').next().unwrap_or("").trim();
            if ["IconResource", "IconFile", "IconIndex"]
                .iter()
                .any(|k| key.eq_ignore_ascii_case(k))
            {
                continue;
            }
        }
        lines.push(line.to_string());
    }

    if !found_section {
        if let Some(name) = icon {
            lines.insert(0, format!("IconResource={name},0"));
            lines.insert(0, SECTION.to_string());
        }
    }

    let mut out = lines.join("\r\n");
    out.push_str("\r\n");
    out
}

fn ini_has_content(content: &str) -> bool {
    content.lines().any(|l| {
        let t = l.trim();
        !t.is_empty() && !t.starts_with('[') && !t.starts_with(';')
    })
}

// ---------- Dosya sistemi yardımcıları ----------

/// Gizli+sistem dosyası yazar. Var olan gizli dosyanın üzerine doğrudan yazmak
/// Windows'ta "erişim engellendi" verdiği için önce silinir.
fn write_hidden_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        if let Ok(meta) = fs::metadata(path) {
            let mut perms = meta.permissions();
            if perms.readonly() {
                #[allow(clippy::permissions_set_readonly_false)]
                perms.set_readonly(false);
                let _ = fs::set_permissions(path, perms);
            }
        }
        fs::remove_file(path).map_err(|e| format!("{} silinemedi: {e}", path.display()))?;
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .attributes(FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM)
        .open(path)
        .map_err(|e| format!("{} yazılamadı: {e}", path.display()))?;
    file.write_all(bytes)
        .map_err(|e| format!("{} yazılamadı: {e}", path.display()))
}

fn set_readonly(folder: &Path, readonly: bool) -> Result<(), String> {
    let mut perms = fs::metadata(folder)
        .map_err(|e| format!("Klasör okunamadı: {e}"))?
        .permissions();
    if perms.readonly() == readonly {
        return Ok(());
    }
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(readonly);
    fs::set_permissions(folder, perms).map_err(|e| format!("Klasör özniteliği ayarlanamadı: {e}"))
}

fn remove_our_icons(folder: &Path, keep: Option<&str>) {
    let Ok(entries) = fs::read_dir(folder) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let ours = name.starts_with(ICON_PREFIX) && name.ends_with(".ico");
        if ours && Some(name.as_str()) != keep {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn wide(p: &Path) -> Vec<u16> {
    p.as_os_str().encode_wide().chain(Some(0)).collect()
}

/// Explorer'a klasörün değiştiğini bildirir. SHCNF_FLUSH ile bildirim işlenene kadar
/// beklenir; böylece pencere F5'e gerek kalmadan hemen güncellenir.
fn notify_explorer(folder: &Path) {
    let item = wide(folder);
    unsafe { SHChangeNotify(SHCNE_UPDATEITEM as i32, SHCNF_PATHW | SHCNF_FLUSH, item.as_ptr().cast(), std::ptr::null()) };
    if let Some(parent) = folder.parent() {
        let parent = wide(parent);
        unsafe { SHChangeNotify(SHCNE_UPDATEDIR as i32, SHCNF_PATHW | SHCNF_FLUSH, parent.as_ptr().cast(), std::ptr::null()) };
    }
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_section_when_missing() {
        let out = edit_ini("", Some("a.ico"));
        assert_eq!(out, "[.ShellClassInfo]\r\nIconResource=a.ico,0\r\n");
    }

    #[test]
    fn replaces_old_icon_and_keeps_other_keys() {
        let src = "[.ShellClassInfo]\r\nIconFile=x.ico\r\nIconIndex=0\r\nLocalizedResourceName=Foo\r\n[Other]\r\nIconFile=keep\r\n";
        let out = edit_ini(src, Some("b.ico"));
        assert_eq!(
            out,
            "[.ShellClassInfo]\r\nIconResource=b.ico,0\r\nLocalizedResourceName=Foo\r\n[Other]\r\nIconFile=keep\r\n"
        );
    }

    /// Shell'in (Explorer'ın) bu klasör için kullandığı ikon dosyasının yolu.
    fn shell_icon_location(folder: &Path) -> String {
        use windows_sys::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICONLOCATION, SHGetFileInfoW};
        com_init();
        let path: Vec<u16> = folder.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut info: SHFILEINFOW = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            SHGetFileInfoW(path.as_ptr(), 0, &mut info, size_of::<SHFILEINFOW>() as u32, SHGFI_ICONLOCATION)
        };
        assert_ne!(ok, 0, "SHGetFileInfoW başarısız");
        let len = info.szDisplayName.iter().position(|&c| c == 0).unwrap_or(0);
        String::from_utf16_lossy(&info.szDisplayName[..len])
    }

    fn test_png(rgb: [u8; 3]) -> Vec<u8> {
        let mut png = Vec::new();
        image::RgbaImage::from_pixel(256, 256, image::Rgba([rgb[0], rgb[1], rgb[2], 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        png
    }

    fn current_icon_name(folder: &Path) -> String {
        fs::read_dir(folder)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .find(|n| n.starts_with(ICON_PREFIX))
            .expect("ikon dosyası yok")
    }

    fn com_init() {
        use windows_sys::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
        unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    }

    /// Explorer'ın bu klasör için çizdiği ikonun sistem ikon listesindeki numarası.
    fn sys_icon_index(folder: &Path) -> i32 {
        use windows_sys::Win32::UI::Shell::{SHFILEINFOW, SHGFI_SYSICONINDEX, SHGetFileInfoW};
        com_init();
        let path: Vec<u16> = folder.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut info: SHFILEINFOW = unsafe { std::mem::zeroed() };
        unsafe { SHGetFileInfoW(path.as_ptr(), 0, &mut info, size_of::<SHFILEINFOW>() as u32, SHGFI_SYSICONINDEX) };
        info.iIcon
    }

    /// Kullanıcının bildirdiği hata: zaten özelleştirilmiş klasöre yeniden uygulayınca
    /// Explorer yeni ikonu görmüyor, varsayılana düşüyordu. Shell'in gördüğü ikon her
    /// uygulamadan sonra diskteki güncel dosya olmalı; sıfırlayınca varsayılana dönmeli.
    #[test]
    fn shell_sees_icon_after_reapply_and_reset() {
        let base = std::env::temp_dir().join(format!("folderstudio-shell-{}", unix_millis()));
        let plain = base.join("plain");
        let dir = base.join("hedef ğüş");
        fs::create_dir_all(&plain).unwrap();
        fs::create_dir_all(&dir).unwrap();
        let default_index = sys_icon_index(&plain);

        for (round, rgb) in [[200, 60, 60], [60, 60, 200], [60, 200, 60]].into_iter().enumerate() {
            std::thread::sleep(std::time::Duration::from_millis(20));
            apply(&dir, &test_png(rgb)).unwrap();
            let name = current_icon_name(&dir);
            let seen = shell_icon_location(&dir);
            assert!(seen.ends_with(&name), "{}. uygulama: shell {seen:?} görüyor, beklenen {name}", round + 1);
            assert_ne!(sys_icon_index(&dir), default_index, "{}. uygulama: Explorer varsayılan ikonu çiziyor", round + 1);
        }

        reset(&dir).unwrap();
        assert_eq!(sys_icon_index(&dir), default_index, "sıfırlama sonrası varsayılan ikon olmalı");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);

        fs::remove_dir_all(&base).unwrap();
    }

    /// Gerçek dosya sistemi üzerinde uygula → kontrol et → sıfırla döngüsü.
    #[test]
    fn apply_and_reset_round_trip() {
        use std::os::windows::fs::MetadataExt;

        let dir = std::env::temp_dir().join(format!("folderstudio-test-ğüş-{}", unix_millis()));
        fs::create_dir_all(&dir).unwrap();

        let mut png = Vec::new();
        image::RgbaImage::from_pixel(256, 256, image::Rgba([80, 140, 240, 255]))
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();

        // İki kez uygula: ikinci seferde eski .ico silinmeli, desktop.ini üzerine yazılabilmeli.
        apply(&dir, &png).unwrap();
        apply(&dir, &png).unwrap();

        let icons: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(ICON_PREFIX))
            .collect();
        assert_eq!(icons.len(), 1, "eski ikon dosyaları temizlenmeli");
        let ico = fs::read(icons[0].path()).unwrap();
        assert_eq!(&ico[..4], &[0, 0, 1, 0], "geçerli ICO başlığı");
        assert_eq!(u16::from_le_bytes([ico[4], ico[5]]) as usize, ICON_SIZES.len());

        let ini = read_desktop_ini(&dir);
        let icon_name = icons[0].file_name().to_string_lossy().to_string();
        assert!(ini.contains(&format!("IconResource={icon_name},0")));
        let ini_attrs = fs::metadata(dir.join("desktop.ini")).unwrap().file_attributes();
        assert_eq!(ini_attrs & (FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM), FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM);
        assert!(fs::metadata(&dir).unwrap().permissions().readonly(), "klasör ReadOnly olmalı");

        reset(&dir).unwrap();
        assert!(!dir.join("desktop.ini").exists());
        assert!(!fs::metadata(&dir).unwrap().permissions().readonly());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0, "klasörde artık dosya kalmamalı");

        fs::remove_dir(&dir).unwrap();
    }

    #[test]
    fn reset_detects_empty_ini() {
        let out = edit_ini("[.ShellClassInfo]\r\nIconResource=a.ico,0\r\n", None);
        assert!(!ini_has_content(&out));
        let out = edit_ini("[.ShellClassInfo]\r\nIconResource=a.ico,0\r\nInfoTip=hi\r\n", None);
        assert!(ini_has_content(&out));
    }
}
