// VisionSelect AI - Main Library
// Tauri alkalmazás fő modulja

// Modulok
mod commands;
mod database;
mod exif;
mod scanner;
mod thumbnail;
mod types;

// Re-exportálás
pub use commands::*;
pub use types::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // AppState inicializálása - in-memory adatbázist használunk a stabilitásért
    let app_state =
        commands::AppState::new_in_memory().expect("Failed to create in-memory database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::scan_folder,
            commands::get_thumbnail,
            commands::get_thumbnails,
            commands::get_exif,
            commands::get_exif_summary,
            commands::save_image,
            commands::get_images_from_db,
            commands::scan_and_save,
            commands::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
