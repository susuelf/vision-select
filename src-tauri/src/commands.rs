// VisionSelect AI - Tauri Commands
// Frontend-backend kommunikációs parancsok

use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

use crate::database::{get_database_path, Database};
use crate::exif::{extract_exif, format_exif_summary};
use crate::scanner::scan_directory;
use crate::thumbnail::extract_thumbnail;
use crate::types::{ExifData, ImageRecord, RawFile, ScanResult, ThumbnailData, VisionError};

/// Alkalmazás állapot (megosztott adatbázis kapcsolat)
pub struct AppState {
    pub db: Mutex<Database>,
}

impl AppState {
    pub fn new() -> Result<Self, VisionError> {
        let db_path = get_database_path();
        let db = Database::new(&db_path)?;
        Ok(Self { db: Mutex::new(db) })
    }

    pub fn new_in_memory() -> Result<Self, VisionError> {
        let db = Database::new_in_memory()?;
        Ok(Self { db: Mutex::new(db) })
    }
}

/// Mappa szkennelése RAW fájlokért
#[tauri::command]
pub fn scan_folder(path: String, recursive: bool) -> Result<ScanResult, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("A mappa nem létezik: {}", path));
    }

    if !path_buf.is_dir() {
        return Err(format!("Nem mappa: {}", path));
    }

    let result = scan_directory(&path_buf, recursive);
    Ok(result)
}

/// Egyetlen kép thumbnail-jének lekérése
#[tauri::command]
pub fn get_thumbnail(path: String, max_size: u32) -> Result<ThumbnailData, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("A fájl nem létezik: {}", path));
    }

    let max_size = if max_size == 0 { 512 } else { max_size };

    extract_thumbnail(&path_buf, max_size).map_err(|e| e.to_string())
}

/// Batch thumbnail lekérés (több kép egyszerre)
#[tauri::command]
pub fn get_thumbnails(paths: Vec<String>, max_size: u32) -> Vec<Result<ThumbnailData, String>> {
    let max_size = if max_size == 0 { 512 } else { max_size };

    paths
        .iter()
        .map(|path| {
            let path_buf = PathBuf::from(path);
            extract_thumbnail(&path_buf, max_size).map_err(|e| e.to_string())
        })
        .collect()
}

/// EXIF adatok lekérése
#[tauri::command]
pub fn get_exif(path: String) -> Result<ExifData, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("A fájl nem létezik: {}", path));
    }

    extract_exif(&path_buf).map_err(|e| e.to_string())
}

/// EXIF összefoglaló string
#[tauri::command]
pub fn get_exif_summary(path: String) -> Result<String, String> {
    let exif = get_exif(path)?;
    Ok(format_exif_summary(&exif))
}

/// Kép mentése az adatbázisba
#[tauri::command]
pub fn save_image(state: State<AppState>, file: RawFile) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let record = ImageRecord {
        id: None,
        file_path: file.path.to_string_lossy().to_string(),
        filename: file.filename,
        extension: file.extension,
        file_size: file.size_bytes,
        file_hash: None,
        thumbnail_path: None,
        exif: None,
        created_at: None,
    };

    db.upsert_image(&record).map_err(|e| e.to_string())
}

/// Képek lekérése mappából (adatbázisból)
#[tauri::command]
pub fn get_images_from_db(
    state: State<AppState>,
    folder: String,
) -> Result<Vec<ImageRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let folder_path = PathBuf::from(&folder);

    db.get_images_by_folder(&folder_path)
        .map_err(|e| e.to_string())
}

/// Szkennelés és adatbázisba mentés egyben
#[tauri::command]
pub fn scan_and_save(
    state: State<AppState>,
    path: String,
    recursive: bool,
) -> Result<ScanResult, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() || !path_buf.is_dir() {
        return Err(format!("Érvénytelen mappa: {}", path));
    }

    // Szkennelés
    let result = scan_directory(&path_buf, recursive);

    // Mentés adatbázisba
    let db = state.db.lock().map_err(|e| e.to_string())?;

    for file in &result.files {
        let record = ImageRecord {
            id: None,
            file_path: file.path.to_string_lossy().to_string(),
            filename: file.filename.clone(),
            extension: file.extension.clone(),
            file_size: file.size_bytes,
            file_hash: None,
            thumbnail_path: None,
            exif: None,
            created_at: None,
        };

        if let Ok(image_id) = db.upsert_image(&record) {
            // EXIF mentése ha sikerül kiolvasni
            if let Ok(exif) = extract_exif(&file.path) {
                let _ = db.save_exif(image_id, &exif);
            }
        }
    }

    Ok(result)
}

/// App verzió lekérdezése
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// === AI Elemzési Parancsok (Phase II) ===

use crate::grouping::{
    group_by_timestamp, rank_group_images, GroupingConfig, ImageGroup, RankedGroup,
};
use crate::quality_analyzer::{QualityAnalyzer, QualityScore};
use std::collections::HashMap;

/// Egyetlen kép minőségi elemzése
#[tauri::command]
pub fn analyze_image(path: String) -> Result<QualityScore, String> {
    let path_buf = PathBuf::from(&path);

    if !path_buf.exists() {
        return Err(format!("A fájl nem létezik: {}", path));
    }

    // Kép betöltése
    let img = image::open(&path_buf).map_err(|e| format!("Kép betöltési hiba: {}", e))?;

    // Elemzés
    let analyzer = QualityAnalyzer::default();
    analyzer.analyze(&img).map_err(|e| e.to_string())
}

/// Batch kép elemzés eredménye
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchAnalysisResult {
    pub scores: HashMap<String, QualityScore>,
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
}

/// Több kép elemzése egyszerre
#[tauri::command]
pub fn analyze_batch(paths: Vec<String>) -> Result<BatchAnalysisResult, String> {
    let analyzer = QualityAnalyzer::default();
    let mut scores: HashMap<String, QualityScore> = HashMap::new();
    let mut successful = 0;
    let mut failed = 0;
    let total = paths.len();

    for path in &paths {
        let path_buf = PathBuf::from(path);

        if !path_buf.exists() {
            failed += 1;
            continue;
        }

        match image::open(&path_buf) {
            Ok(img) => match analyzer.analyze(&img) {
                Ok(score) => {
                    scores.insert(path.clone(), score);
                    successful += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            },
            Err(_) => {
                failed += 1;
            }
        }
    }

    Ok(BatchAnalysisResult {
        scores,
        total,
        successful,
        failed,
    })
}

/// Képek csoportosítása időbélyeg alapján
#[tauri::command]
pub fn get_image_groups(
    state: State<AppState>,
    folder: String,
    threshold_secs: Option<i64>,
) -> Result<Vec<ImageGroup>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let folder_path = PathBuf::from(&folder);

    // Képek lekérése az adatbázisból
    let images = db
        .get_images_by_folder(&folder_path)
        .map_err(|e| e.to_string())?;

    // Csoportosítási konfiguráció
    let config = GroupingConfig {
        burst_threshold_secs: threshold_secs.unwrap_or(3),
        ..Default::default()
    };

    // Csoportosítás
    let groups = group_by_timestamp(&images, &config);

    Ok(groups)
}

/// Csoport rangsorolása AI pontszámokkal
#[tauri::command]
pub fn get_ranked_group(
    group: ImageGroup,
    scores: HashMap<String, QualityScore>,
) -> Result<RankedGroup, String> {
    Ok(rank_group_images(&group, &scores))
}
