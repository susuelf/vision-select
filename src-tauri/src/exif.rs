// VisionSelect AI - EXIF Metadata Extractor
// EXIF metaadatok kinyerése RAW és JPEG fájlokból

use chrono::{DateTime, NaiveDateTime, Utc};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::types::{ExifData, VisionError};

/// EXIF metaadatok kinyerése fájlból
pub fn extract_exif(path: &Path) -> Result<ExifData, VisionError> {
    let file =
        File::open(path).map_err(|e| VisionError::ExifError(format!("Cannot open file: {}", e)))?;

    let mut reader = BufReader::new(&file);
    let exif_reader = exif::Reader::new();

    let exif = exif_reader
        .read_from_container(&mut reader)
        .map_err(|e| VisionError::ExifError(format!("Cannot read EXIF: {}", e)))?;

    let mut data = ExifData::default();

    // ISO
    if let Some(field) = exif.get_field(exif::Tag::PhotographicSensitivity, exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            data.iso = Some(val);
        }
    }

    // Shutter speed
    if let Some(field) = exif.get_field(exif::Tag::ExposureTime, exif::In::PRIMARY) {
        data.shutter_speed = Some(field.display_value().to_string());
    }

    // Aperture
    if let Some(field) = exif.get_field(exif::Tag::FNumber, exif::In::PRIMARY) {
        if let exif::Value::Rational(ref vals) = field.value {
            if let Some(r) = vals.first() {
                data.aperture = Some(r.num as f32 / r.denom as f32);
            }
        }
    }

    // Focal length
    if let Some(field) = exif.get_field(exif::Tag::FocalLength, exif::In::PRIMARY) {
        if let exif::Value::Rational(ref vals) = field.value {
            if let Some(r) = vals.first() {
                data.focal_length = Some(r.num as f32 / r.denom as f32);
            }
        }
    }

    // Capture date
    if let Some(field) = exif.get_field(exif::Tag::DateTimeOriginal, exif::In::PRIMARY) {
        if let exif::Value::Ascii(ref vecs) = field.value {
            if let Some(bytes) = vecs.first() {
                if let Ok(date_str) = std::str::from_utf8(bytes) {
                    // Format: "2024:12:19 15:30:45"
                    if let Ok(naive) = NaiveDateTime::parse_from_str(date_str, "%Y:%m:%d %H:%M:%S")
                    {
                        data.capture_date =
                            Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
                    }
                }
            }
        }
    }

    // Camera make
    if let Some(field) = exif.get_field(exif::Tag::Make, exif::In::PRIMARY) {
        data.camera_make = Some(
            field
                .display_value()
                .to_string()
                .trim_matches('"')
                .to_string(),
        );
    }

    // Camera model
    if let Some(field) = exif.get_field(exif::Tag::Model, exif::In::PRIMARY) {
        data.camera_model = Some(
            field
                .display_value()
                .to_string()
                .trim_matches('"')
                .to_string(),
        );
    }

    // Lens model
    if let Some(field) = exif.get_field(exif::Tag::LensModel, exif::In::PRIMARY) {
        data.lens_model = Some(
            field
                .display_value()
                .to_string()
                .trim_matches('"')
                .to_string(),
        );
    }

    // Orientation
    if let Some(field) = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY) {
        if let Some(val) = field.value.get_uint(0) {
            data.orientation = Some(val as u16);
        }
    }

    Ok(data)
}

/// EXIF összefoglaló string formázása
pub fn format_exif_summary(exif: &ExifData) -> String {
    let mut parts = Vec::new();

    if let Some(iso) = exif.iso {
        parts.push(format!("ISO {}", iso));
    }

    if let Some(ref ss) = exif.shutter_speed {
        parts.push(ss.clone());
    }

    if let Some(ap) = exif.aperture {
        parts.push(format!("f/{:.1}", ap));
    }

    if let Some(fl) = exif.focal_length {
        parts.push(format!("{}mm", fl as u32));
    }

    parts.join(" | ")
}
