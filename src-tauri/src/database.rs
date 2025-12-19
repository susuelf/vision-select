// VisionSelect AI - SQLite Database
// Lokális adatbázis a képek metaadatainak és AI pontszámoknak tárolásához

use rusqlite::{params, Connection};
use std::path::{Path, PathBuf};

use crate::types::{ExifData, ImageRecord, VisionError};

/// Adatbázis kapcsolat
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Új adatbázis kapcsolat létrehozása
    /// Az app data könyvtárban tárolja a db fájlt
    pub fn new(db_path: &Path) -> Result<Self, VisionError> {
        // Mappa létrehozása ha nem létezik
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                VisionError::DatabaseError(format!("Cannot create db directory: {}", e))
            })?;
        }

        let conn = Connection::open(db_path)
            .map_err(|e| VisionError::DatabaseError(format!("Cannot open database: {}", e)))?;

        let db = Self { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// In-memory adatbázis (teszteléshez)
    pub fn new_in_memory() -> Result<Self, VisionError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| VisionError::DatabaseError(format!("Cannot open in-memory db: {}", e)))?;

        let db = Self { conn };
        db.init_schema()?;

        Ok(db)
    }

    /// Séma inicializálása
    fn init_schema(&self) -> Result<(), VisionError> {
        self.conn
            .execute_batch(
                r#"
            -- Képek táblája
            CREATE TABLE IF NOT EXISTS images (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT UNIQUE NOT NULL,
                filename TEXT NOT NULL,
                extension TEXT NOT NULL,
                file_size INTEGER,
                file_hash TEXT,
                thumbnail_path TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                modified_at DATETIME
            );

            -- EXIF adatok
            CREATE TABLE IF NOT EXISTS exif_data (
                image_id INTEGER PRIMARY KEY,
                iso INTEGER,
                shutter_speed TEXT,
                aperture REAL,
                focal_length REAL,
                capture_date DATETIME,
                camera_make TEXT,
                camera_model TEXT,
                lens_model TEXT,
                orientation INTEGER,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            -- AI elemzési eredmények (jövőbeli használatra)
            CREATE TABLE IF NOT EXISTS ai_scores (
                image_id INTEGER PRIMARY KEY,
                category TEXT,
                overall_score REAL,
                sharpness_score REAL,
                exposure_score REAL,
                face_score REAL,
                analyzed_at DATETIME,
                model_version TEXT,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            -- Csoportok (sorozatfelvételek)
            CREATE TABLE IF NOT EXISTS image_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS group_members (
                group_id INTEGER,
                image_id INTEGER,
                is_winner INTEGER DEFAULT 0,
                rank INTEGER,
                PRIMARY KEY (group_id, image_id),
                FOREIGN KEY (group_id) REFERENCES image_groups(id) ON DELETE CASCADE,
                FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE
            );

            -- Indexek
            CREATE INDEX IF NOT EXISTS idx_images_path ON images(file_path);
            CREATE INDEX IF NOT EXISTS idx_exif_date ON exif_data(capture_date);
            "#,
            )
            .map_err(|e| VisionError::DatabaseError(format!("Schema init error: {}", e)))?;

        Ok(())
    }

    /// Kép beszúrása vagy frissítése
    pub fn upsert_image(&self, record: &ImageRecord) -> Result<i64, VisionError> {
        self.conn.execute(
            r#"
            INSERT INTO images (file_path, filename, extension, file_size, file_hash, thumbnail_path)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(file_path) DO UPDATE SET
                file_size = excluded.file_size,
                file_hash = excluded.file_hash,
                thumbnail_path = excluded.thumbnail_path,
                modified_at = CURRENT_TIMESTAMP
            "#,
            params![
                record.file_path,
                record.filename,
                record.extension,
                record.file_size,
                record.file_hash,
                record.thumbnail_path,
            ],
        ).map_err(|e| VisionError::DatabaseError(format!("Insert error: {}", e)))?;

        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    /// EXIF adatok mentése
    pub fn save_exif(&self, image_id: i64, exif: &ExifData) -> Result<(), VisionError> {
        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO exif_data 
            (image_id, iso, shutter_speed, aperture, focal_length, capture_date, camera_make, camera_model, lens_model, orientation)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                image_id,
                exif.iso,
                exif.shutter_speed,
                exif.aperture,
                exif.focal_length,
                exif.capture_date.map(|d| d.to_rfc3339()),
                exif.camera_make,
                exif.camera_model,
                exif.lens_model,
                exif.orientation,
            ],
        ).map_err(|e| VisionError::DatabaseError(format!("EXIF save error: {}", e)))?;

        Ok(())
    }

    /// Képek lekérdezése mappa alapján
    pub fn get_images_by_folder(&self, folder: &Path) -> Result<Vec<ImageRecord>, VisionError> {
        let folder_str = folder.to_string_lossy().to_string();
        let pattern = format!("{}%", folder_str);

        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, file_path, filename, extension, file_size, file_hash, thumbnail_path, created_at
            FROM images
            WHERE file_path LIKE ?1
            ORDER BY created_at DESC
            "#
        ).map_err(|e| VisionError::DatabaseError(format!("Query error: {}", e)))?;

        let rows = stmt
            .query_map([pattern], |row| {
                Ok(ImageRecord {
                    id: Some(row.get(0)?),
                    file_path: row.get(1)?,
                    filename: row.get(2)?,
                    extension: row.get(3)?,
                    file_size: row.get(4)?,
                    file_hash: row.get(5)?,
                    thumbnail_path: row.get(6)?,
                    exif: None,
                    created_at: None,
                })
            })
            .map_err(|e| VisionError::DatabaseError(format!("Query map error: {}", e)))?;

        let mut records = Vec::new();
        for row in rows {
            if let Ok(record) = row {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Kép lekérdezése path alapján
    pub fn get_image_by_path(&self, path: &str) -> Result<Option<ImageRecord>, VisionError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, file_path, filename, extension, file_size, file_hash, thumbnail_path, created_at
            FROM images
            WHERE file_path = ?1
            "#
        ).map_err(|e| VisionError::DatabaseError(format!("Query error: {}", e)))?;

        let mut rows = stmt
            .query([path])
            .map_err(|e| VisionError::DatabaseError(format!("Query error: {}", e)))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| VisionError::DatabaseError(e.to_string()))?
        {
            Ok(Some(ImageRecord {
                id: Some(
                    row.get(0)
                        .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                ),
                file_path: row
                    .get(1)
                    .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                filename: row
                    .get(2)
                    .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                extension: row
                    .get(3)
                    .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                file_size: row
                    .get(4)
                    .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                file_hash: row
                    .get(5)
                    .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                thumbnail_path: row
                    .get(6)
                    .map_err(|e| VisionError::DatabaseError(e.to_string()))?,
                exif: None,
                created_at: None,
            }))
        } else {
            Ok(None)
        }
    }
}

/// App data könyvtár elérése
pub fn get_app_data_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("VisionSelect")
}

/// Adatbázis fájl elérési útja
pub fn get_database_path() -> PathBuf {
    get_app_data_path().join("visionselect.db")
}
