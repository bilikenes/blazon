mod eyedropper;
mod folder_icon;
mod menu;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{Emitter, Manager};

/// Sağ tık menüsünden (veya komut satırından) verilen klasör yolu.
#[derive(Default)]
struct LaunchFolder(Mutex<Option<String>>);

/// Argümanlar arasından var olan ilk klasör yolunu bulur.
fn folder_from_args(args: &[String]) -> Option<String> {
    args.iter()
        .skip(1)
        .find(|a| !a.starts_with('-') && PathBuf::from(a).is_dir())
        .cloned()
}

/// Arayüzde çizilen 256x256 PNG'yi klasöre ikon olarak uygular.
#[tauri::command]
async fn apply_folder_icon(
    folder: String,
    png: Vec<u8>,
    look: Option<folder_icon::Look>,
) -> Result<(), folder_icon::IconError> {
    folder_icon::apply(&PathBuf::from(folder), &png, look.as_ref())
}

/// Klasörde Blazon ile kaydedilmiş görünüm varsa döndürür.
#[tauri::command]
async fn read_folder_look(folder: String) -> Option<folder_icon::Look> {
    folder_icon::read_look(&PathBuf::from(folder))
}

/// Klasörü varsayılan Windows ikonuna döndürür.
#[tauri::command]
async fn reset_folder_icon(folder: String) -> Result<(), folder_icon::IconError> {
    folder_icon::reset(&PathBuf::from(folder))
}

/// Uygulama bir klasörle başlatıldıysa yolunu verir (bir kez).
#[tauri::command]
fn take_launch_folder(state: tauri::State<'_, LaunchFolder>) -> Option<String> {
    state.0.lock().ok().and_then(|mut g| g.take())
}

/// Windows 11 ana sağ tık menüsü seçeneğinin durumu.
#[tauri::command]
async fn menu_status() -> menu::MenuStatus {
    menu::status()
}

/// Ana sağ tık menüsünü açar ya da kapatır (yönetici onayı isteyebilir) ve yeni durumu verir.
#[tauri::command]
async fn set_menu(enabled: bool) -> Result<menu::MenuStatus, menu::MenuError> {
    if enabled {
        menu::enable()?;
    } else {
        menu::disable(true)?;
    }
    Ok(menu::status())
}

/// Ekrandan renk seçtirir; vazgeçilirse `None`.
#[tauri::command]
async fn pick_screen_color(window: tauri::WebviewWindow) -> Option<String> {
    let color = tauri::async_runtime::spawn_blocking(eyedropper::pick)
        .await
        .ok()
        .flatten();
    let _ = window.set_focus();
    color
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    // Kurulum programının çağırdığı, pencere açmayan işlemler.
    if let Some(code) = menu::run_cli(&args) {
        std::process::exit(code);
    }
    let launch = LaunchFolder(Mutex::new(folder_from_args(&args)));

    let builder = tauri::Builder::default();

    // Uygulama zaten açıksa yeni pencere açma; klasörü mevcut pencereye ilet.
    #[cfg(desktop)]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
        if let Some(folder) = folder_from_args(&args) {
            let _ = app.emit("open-folder", folder);
        }
    }));

    builder
        .manage(launch)
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            apply_folder_icon,
            reset_folder_icon,
            read_folder_look,
            take_launch_folder,
            menu_status,
            set_menu,
            pick_screen_color
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
