// VisionSelect AI - Common Types
// Közös típusdefiníciók a projekt számára

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// RAW fájl alapinformációk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFile {
    pub path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
}

/// EXIF metaadatok
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExifData {
    pub iso: Option<u32>,
    pub shutter_speed: Option<String>,
    pub aperture: Option<f32>,
    pub focal_length: Option<f32>,
    pub capture_date: Option<DateTime<Utc>>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub orientation: Option<u16>,
}

/// Kép rekord az adatbázisban
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecord {
    pub id: Option<i64>,
    pub file_path: String,
    pub filename: String,
    pub extension: String,
    pub file_size: u64,
    pub file_hash: Option<String>,
    pub thumbnail_path: Option<String>,
    pub exif: Option<ExifData>,
    pub created_at: Option<DateTime<Utc>>,
}

/// Támogatott RAW formátumok
pub const RAW_EXTENSIONS: &[&str] = &[
    "arw",  // Sony
    "cr2",  // Canon
    "cr3",  // Canon (új)
    "nef",  // Nikon
    "nrw",  // Nikon (kompakt)
    "raf",  // Fujifilm
    "orf",  // Olympus
    "rw2",  // Panasonic
    "pef",  // Pentax
    "srw",  // Samsung
    "dng",  // Adobe DNG
];

/// Ellenőrzi, hogy a kiterjesztés RAW formátum-e
pub fn is_raw_extension(ext: &str) -> bool {
    let ext_lower = ext.to_lowercase();
    let ext_clean = ext_lower.trim_start_matches('.');
    RAW_EXTENSIONS.contains(&ext_clean)
}

/// Thumbnail adat base64 kódolással
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailData {
    pub image_path: String,
    pub data_base64: String,
    pub width: u32,
    pub height: u32,
}

/// Szkennelési eredmény
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub files: Vec<RawFile>,
    pub total_count: usize,
    pub total_size_bytes: u64,
    pub scan_duration_ms: u64,
}

/// Hiba típusok
#[derive(Debug, thiserror::Error)]
pub enum VisionError {
    #[error("Fájl nem található: {0}")]
    FileNotFound(String),
    
    #[error("Nem támogatott formátum: {0}")]
    UnsupportedFormat(String),
    
    #[error("EXIF olvasási hiba: {0}")]
    ExifError(String),
    
    #[error("Thumbnail kinyerési hiba: {0}")]
    ThumbnailError(String),
    
    #[error("Adatbázis hiba: {0}")]
    DatabaseError(String),
    
    #[error("IO hiba: {0}")]
    IoError(#[from] std::io::Error),
}

// Serialize implementáció a Tauri számára
impl Serialize for VisionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
