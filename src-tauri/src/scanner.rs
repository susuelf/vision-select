// VisionSelect AI - Directory Scanner
// Fájlrendszer szkennelő RAW fájlok kereséséhez

use chrono::{DateTime, Utc};
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

use crate::types::{is_raw_extension, RawFile, ScanResult};

/// Rekurzívan bejárja a megadott mappát és összegyűjti a RAW fájlokat
pub fn scan_directory(path: &Path, recursive: bool) -> ScanResult {
    let start = Instant::now();
    let mut files = Vec::new();
    let mut total_size: u64 = 0;

    let walker = if recursive {
        WalkDir::new(path)
    } else {
        WalkDir::new(path).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();

        // Csak fájlok érdekelnek
        if !entry_path.is_file() {
            continue;
        }

        // Kiterjesztés ellenőrzése
        let extension = match entry_path.extension() {
            Some(ext) => ext.to_string_lossy().to_string(),
            None => continue,
        };

        if !is_raw_extension(&extension) {
            continue;
        }

        // Fájl metaadatok
        let metadata = match entry_path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size = metadata.len();
        total_size += size;

        // Módosítási idő
        let modified_at = metadata.modified().ok().map(|t| DateTime::<Utc>::from(t));

        let filename = entry_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        files.push(RawFile {
            path: entry_path.to_path_buf(),
            filename,
            extension: extension.to_lowercase(),
            size_bytes: size,
            modified_at,
        });
    }

    // Rendezés dátum szerint (legújabb elöl)
    files.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    let duration = start.elapsed();

    ScanResult {
        total_count: files.len(),
        total_size_bytes: total_size,
        scan_duration_ms: duration.as_millis() as u64,
        files,
    }
}

/// Egyetlen fájl ellenőrzése
pub fn is_valid_raw_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    match path.extension() {
        Some(ext) => is_raw_extension(&ext.to_string_lossy()),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_raw_extension() {
        assert!(is_raw_extension("arw"));
        assert!(is_raw_extension("ARW"));
        assert!(is_raw_extension(".cr2"));
        assert!(is_raw_extension("NEF"));
        assert!(!is_raw_extension("jpg"));
        assert!(!is_raw_extension("png"));
    }
}
