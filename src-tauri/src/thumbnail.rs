// VisionSelect AI - Thumbnail Extractor
// RAW fájlokból előnézeti képek kinyerése

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;
use std::path::Path;

use crate::types::{ThumbnailData, VisionError};

/// RAW fájlból thumbnail kinyerése
/// Először rawloader-rel próbál, majd fallback standard image formátumokra
pub fn extract_thumbnail(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    // Próbáljuk meg a rawloader-rel
    match extract_raw_thumbnail(path, max_size) {
        Ok(thumb) => Ok(thumb),
        Err(_) => {
            // Fallback: próbáljuk meg standard image crate-tel
            extract_thumbnail_from_standard_image(path, max_size)
        }
    }
}

/// RAW fájlból thumbnail kinyerése rawloader segítségével
fn extract_raw_thumbnail(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();

    // RAW fájl betöltése
    let raw_img = rawloader::decode_file(&path_str)
        .map_err(|e| VisionError::ThumbnailError(format!("RAW decode error: {}", e)))?;

    let width = raw_img.width as u32;
    let height = raw_img.height as u32;

    // A hivatalos dokumentáció szerint: if let RawImageData::Integer(data) = raw_img.data
    let pixels: Vec<u8> = if let rawloader::RawImageData::Integer(data) = raw_img.data {
        // 16-bit adat, konvertálás 8-bitre
        data.iter().map(|pix| (*pix >> 8) as u8).collect()
    } else {
        return Err(VisionError::ThumbnailError(
            "Non-integer raw format not supported".into(),
        ));
    };

    // Egyszerű szürkeárnyalatos kép
    let expected_gray = (width * height) as usize;

    let dynamic_img = if pixels.len() >= expected_gray {
        let gray_pixels: Vec<u8> = pixels.into_iter().take(expected_gray).collect();
        if let Some(gray_img) = image::GrayImage::from_raw(width, height, gray_pixels) {
            DynamicImage::ImageLuma8(gray_img)
        } else {
            return Err(VisionError::ThumbnailError(
                "Failed to create gray buffer".into(),
            ));
        }
    } else {
        return Err(VisionError::ThumbnailError(format!(
            "Not enough pixels: {} expected, {} got",
            expected_gray,
            pixels.len()
        )));
    };

    // Átméretezés
    let resized = resize_image(&dynamic_img, max_size);
    let final_width = resized.width();
    let final_height = resized.height();

    // JPEG-be kódolás
    let mut buffer = Cursor::new(Vec::new());
    resized
        .write_to(&mut buffer, ImageFormat::Jpeg)
        .map_err(|e| VisionError::ThumbnailError(format!("JPEG encode error: {}", e)))?;

    // Base64 kódolás
    let base64_data = STANDARD.encode(buffer.into_inner());

    Ok(ThumbnailData {
        image_path: path_str,
        data_base64: format!("data:image/jpeg;base64,{}", base64_data),
        width: final_width,
        height: final_height,
    })
}

/// Kép átméretezése a megadott maximális méretre
/// Megtartja az arányokat
fn resize_image(img: &DynamicImage, max_size: u32) -> DynamicImage {
    let (width, height) = (img.width(), img.height());

    // Ha a kép már kisebb, nem méretezünk
    if width <= max_size && height <= max_size {
        return img.clone();
    }

    // Arány megtartásával méretezés
    let ratio = (max_size as f32) / (width.max(height) as f32);
    let new_width = (width as f32 * ratio) as u32;
    let new_height = (height as f32 * ratio) as u32;

    img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3)
}

/// Thumbnail kinyerése JPEG/PNG fájlból (nem RAW)
pub fn extract_thumbnail_from_standard_image(
    path: &Path,
    max_size: u32,
) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();

    let loaded_img = image::open(path)
        .map_err(|e| VisionError::ThumbnailError(format!("Image open error: {}", e)))?;

    let resized = resize_image(&loaded_img, max_size);
    let final_width = resized.width();
    let final_height = resized.height();

    let mut buffer = Cursor::new(Vec::new());
    resized
        .write_to(&mut buffer, ImageFormat::Jpeg)
        .map_err(|e| VisionError::ThumbnailError(format!("JPEG encode error: {}", e)))?;

    let base64_data = STANDARD.encode(buffer.into_inner());

    Ok(ThumbnailData {
        image_path: path_str,
        data_base64: format!("data:image/jpeg;base64,{}", base64_data),
        width: final_width,
        height: final_height,
    })
}
