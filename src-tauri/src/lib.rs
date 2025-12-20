// VisionSelect AI - Main Library
// Tauri alkalmazás fő modulja

// Modulok
mod commands;
mod database;
mod exif;
mod grouping;
mod quality_analyzer;
mod scanner;
mod thumbnail;
mod types;

// Re-exportálás
pub use commands::*;
pub use grouping::*;
pub use quality_analyzer::*;
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
            // Fájlrendszer
            commands::scan_folder,
            commands::scan_and_save,
            // Thumbnail
            commands::get_thumbnail,
            commands::get_thumbnails,
            // EXIF
            commands::get_exif,
            commands::get_exif_summary,
            // Adatbázis
            commands::save_image,
            commands::get_images_from_db,
            // AI Elemzés (Phase II)
            commands::analyze_image,
            commands::analyze_batch,
            commands::get_image_groups,
            // Verzió
            commands::get_app_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
