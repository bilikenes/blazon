mod folder_icon;

use std::path::PathBuf;

/// Arayüzde çizilen 256x256 PNG'yi klasöre ikon olarak uygular.
#[tauri::command]
async fn apply_folder_icon(folder: String, png: Vec<u8>) -> Result<(), String> {
    folder_icon::apply(&PathBuf::from(folder), &png)
}

/// Klasörü varsayılan Windows ikonuna döndürür.
#[tauri::command]
async fn reset_folder_icon(folder: String) -> Result<(), String> {
    folder_icon::reset(&PathBuf::from(folder))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![apply_folder_icon, reset_folder_icon])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
