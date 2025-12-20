// VisionSelect AI - Stable RAW Thumbnail Extractor
// jpeg-decoder scale() támogatással + 4 szál
//
// A jpeg-decoder támogatja a scale() metódust, ami hardveres IDCT scaling-et végez!

use base64::{engine::general_purpose::STANDARD, Engine};
use byteorder::{BigEndian, ByteOrder};
use jpeg_decoder::Decoder as JpegDecoder;
use memchr::memchr_iter;
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::Cursor;
use std::panic;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};

use crate::types::{ThumbnailData, VisionError};

// === Constants ===

const MAX_HEADER_SIZE: usize = 5 * 1024 * 1024;
const MIN_JPEG_SIZE: usize = 10_000;
const TARGET_THUMB_SIZE: u32 = 256;

// === RAW Format Detection ===

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RawFormat {
    Arw,
    Cr2,
    Cr3,
    Nef,
    Dng,
    Orf,
    Rw2,
    Raf,
    Pef,
    Jpeg,
    Png,
    Unknown,
}

impl RawFormat {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "arw" | "srf" | "sr2" => Self::Arw,
            "cr2" => Self::Cr2,
            "cr3" => Self::Cr3,
            "nef" | "nrw" => Self::Nef,
            "dng" => Self::Dng,
            "orf" => Self::Orf,
            "rw2" => Self::Rw2,
            "raf" => Self::Raf,
            "pef" => Self::Pef,
            "jpg" | "jpeg" => Self::Jpeg,
            "png" => Self::Png,
            _ => Self::Unknown,
        }
    }
}

// === Batch Processing ===

pub struct ThumbnailResult {
    pub path: PathBuf,
    pub result: Result<ThumbnailData, VisionError>,
}

pub struct ThumbnailExtractor {
    pub max_size: u32,
    pub num_threads: usize,
}

impl Default for ThumbnailExtractor {
    fn default() -> Self {
        Self {
            max_size: TARGET_THUMB_SIZE,
            // 2 szál a stabilitás érdekében (RAM limitation)
            num_threads: 2,
        }
    }
}

impl ThumbnailExtractor {
    pub fn extract_batch(&self, paths: Vec<PathBuf>) -> Receiver<ThumbnailResult> {
        let (tx, rx): (Sender<ThumbnailResult>, Receiver<ThumbnailResult>) = mpsc::channel();
        let max_size = self.max_size;

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build()
            .unwrap_or_else(|_| {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(1)
                    .build()
                    .unwrap()
            });

        std::thread::spawn(move || {
            pool.install(|| {
                paths.into_par_iter().for_each_with(tx, |sender, path| {
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        extract_thumbnail(&path, max_size)
                    }));

                    let final_result = match result {
                        Ok(r) => r,
                        Err(_) => Err(VisionError::ThumbnailError("Panic".into())),
                    };

                    let _ = sender.send(ThumbnailResult {
                        path,
                        result: final_result,
                    });
                });
            });
        });

        rx
    }
}

// === Main Entry Point ===

pub fn extract_thumbnail(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();

    if !path.exists() {
        return Err(VisionError::ThumbnailError(format!(
            "Not found: {}",
            path_str
        )));
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let format = RawFormat::from_extension(ext);

    let result = match format {
        RawFormat::Jpeg | RawFormat::Png => extract_standard_image(path, max_size),
        RawFormat::Cr3 => extract_cr3_thumbnail_safe(path, max_size)
            .or_else(|_| extract_jpeg_simd(path, max_size)),
        RawFormat::Arw
        | RawFormat::Cr2
        | RawFormat::Nef
        | RawFormat::Dng
        | RawFormat::Orf
        | RawFormat::Rw2
        | RawFormat::Raf
        | RawFormat::Pef => extract_exif_thumbnail_fast(path, max_size)
            .or_else(|_| extract_jpeg_simd(path, max_size)),
        RawFormat::Unknown => {
            extract_jpeg_simd(path, max_size).or_else(|_| extract_standard_image(path, max_size))
        }
    };

    result.or_else(|_| create_placeholder(&path_str))
}

// === CR3 Safe Parser ===

fn extract_cr3_thumbnail_safe(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();
    let file = File::open(path).map_err(|e| VisionError::ThumbnailError(e.to_string()))?;
    let mmap =
        unsafe { Mmap::map(&file) }.map_err(|e| VisionError::ThumbnailError(e.to_string()))?;

    let data_len = mmap.len().min(MAX_HEADER_SIZE);
    let data = &mmap[..data_len];
    let mut pos = 0;

    while pos + 8 <= data_len {
        let size = BigEndian::read_u32(&data[pos..pos + 4]) as usize;

        // size == 0 végtelen ciklus ellen
        if size == 0 || size < 8 {
            break;
        }

        let next_pos = pos.saturating_add(size);
        if next_pos > data_len {
            break;
        }

        let box_type = &data[pos + 4..pos + 8];

        if matches!(box_type, b"PRVW" | b"THMB" | b"prvw" | b"thmb") {
            let box_data = &data[pos + 8..next_pos];
            // Biztonsági ellenőrzés: ha túl nagy a box (> 50MB), valószínűleg full preview, ami fagyást okozhat
            if box_data.len() > 50 * 1024 * 1024 {
                pos = next_pos;
                continue;
            }

            if let Some(jpeg) = find_jpeg_in_box(box_data) {
                if jpeg.len() >= MIN_JPEG_SIZE {
                    return decode_jpeg_scaled_safe(&path_str, jpeg, max_size);
                }
            }
        }

        if box_type == b"mdat" {
            pos = next_pos;
            continue;
        }

        if matches!(box_type, b"moov" | b"trak" | b"mdia" | b"minf") {
            let inner = &data[pos + 8..next_pos];
            if let Some(jpeg) = find_jpeg_in_box(inner) {
                if jpeg.len() >= MIN_JPEG_SIZE {
                    return decode_jpeg_scaled_safe(&path_str, jpeg, max_size);
                }
            }
        }

        pos = next_pos;
    }

    Err(VisionError::ThumbnailError("No CR3 preview".into()))
}

fn find_jpeg_in_box(data: &[u8]) -> Option<&[u8]> {
    if data.len() < 10 {
        return None;
    }
    for i in memchr_iter(0xFF, data) {
        if i + 2 < data.len() && data[i + 1] == 0xD8 && data[i + 2] == 0xFF {
            if let Some(end) = find_jpeg_end(&data[i..]) {
                let size = end + 2;
                if i + size <= data.len() && size >= MIN_JPEG_SIZE {
                    return Some(&data[i..i + size]);
                }
            }
        }
    }
    None
}

// === EXIF & SIMD ===

fn extract_exif_thumbnail_fast(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();
    let file = File::open(path).map_err(|e| VisionError::ThumbnailError(e.to_string()))?;
    let mmap =
        unsafe { Mmap::map(&file) }.map_err(|e| VisionError::ThumbnailError(e.to_string()))?;
    let header_len = mmap.len().min(MAX_HEADER_SIZE);

    let reader = std::io::Cursor::new(&mmap[..header_len]);
    let mut bufreader = std::io::BufReader::new(reader);

    if let Ok(exif) = exif::Reader::new().read_from_container(&mut bufreader) {
        if let Some(jpeg) = get_exif_jpeg_data(&exif, &mmap) {
            return decode_jpeg_scaled_safe(&path_str, jpeg, max_size);
        }
    }
    Err(VisionError::ThumbnailError("No EXIF thumbnail".into()))
}

fn get_exif_jpeg_data<'a>(exif: &exif::Exif, mmap: &'a Mmap) -> Option<&'a [u8]> {
    for ifd in [exif::In::THUMBNAIL, exif::In::PRIMARY] {
        if let (Some(offset_field), Some(length_field)) = (
            exif.get_field(exif::Tag::JPEGInterchangeFormat, ifd),
            exif.get_field(exif::Tag::JPEGInterchangeFormatLength, ifd),
        ) {
            if let (Some(offset), Some(length)) = (
                offset_field.value.get_uint(0),
                length_field.value.get_uint(0),
            ) {
                let (off, len) = (offset as usize, length as usize);
                if len >= 1000 && off.saturating_add(len) <= mmap.len() {
                    return Some(&mmap[off..off + len]);
                }
            }
        }
    }
    None
}

fn extract_jpeg_simd(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();
    let file = File::open(path).map_err(|e| VisionError::ThumbnailError(e.to_string()))?;
    let mmap =
        unsafe { Mmap::map(&file) }.map_err(|e| VisionError::ThumbnailError(e.to_string()))?;
    let data = &mmap[..mmap.len().min(MAX_HEADER_SIZE)];

    let mut best: Option<&[u8]> = None;
    for i in memchr_iter(0xFF, data) {
        if i + 2 < data.len() && data[i + 1] == 0xD8 && data[i + 2] == 0xFF {
            if let Some(end) = find_jpeg_end(&data[i..]) {
                let size = end + 2;
                if i + size <= data.len() && size >= MIN_JPEG_SIZE {
                    if best.is_none() || size > best.unwrap().len() {
                        best = Some(&data[i..i + size]);
                    }
                }
            }
        }
    }
    if let Some(jpeg) = best {
        return decode_jpeg_scaled_safe(&path_str, jpeg, max_size);
    }
    Err(VisionError::ThumbnailError("No JPEG found".into()))
}

fn find_jpeg_end(data: &[u8]) -> Option<usize> {
    let max_search = data.len().min(10 * 1024 * 1024);
    if max_search < 10 {
        return None;
    }
    for i in memchr_iter(0xFF, &data[2..max_search]) {
        let pos = i + 2;
        if pos + 1 < data.len() && data[pos + 1] == 0xD9 {
            return Some(pos + 1);
        }
    }
    None
}

// === jpeg-decoder scale() DECODE ===

fn decode_jpeg_scaled_safe(
    path_str: &str,
    jpeg_data: &[u8],
    max_size: u32,
) -> Result<ThumbnailData, VisionError> {
    if jpeg_data.len() < 100 {
        return Err(VisionError::ThumbnailError("JPEG too small".into()));
    }

    // Explicit memory handling & Panic catch
    // A jpeg-decoder scale() hardveres gyorsítást (IDCT) használ, így kevesebb RAM-ot eszik.
    // De ha mégis túl nagy, elkapjuk a pánikot.
    let decode_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || -> Result<image::DynamicImage, String> {
            let mut decoder = JpegDecoder::new(Cursor::new(jpeg_data));

            // Info olvasása
            decoder
                .read_info()
                .map_err(|e| format!("Read info: {:?}", e))?;
            let info = decoder.info().ok_or("No info")?;

            // Méret ellenőrzés (pl. 8000x8000 felett gyanús lehet, de hagyjuk, ha scale van)
            // Hard limit a raw buffer méretre (pl 200MB)
            let estimated_raw_size = info.width as u64 * info.height as u64 * 3;
            if estimated_raw_size > 200 * 1024 * 1024 {
                // Ha nem tudunk skálázni, ez túl nagy lenne. De próbálunk skálázni.
            }

            // IDCT scaling
            let max_dim = info.width.max(info.height) as u32;
            if max_dim > max_size {
                match decoder.scale(max_size as u16, max_size as u16) {
                    Ok(_) => {}
                    Err(e) => return Err(format!("Scale setup failed: {:?}", e)),
                }
            }

            // Dekódolás (itt történhet OOM, amit elkapunk)
            let pixels = decoder.decode().map_err(|e| format!("Decode: {:?}", e))?;

            // Info frissítése a dekódolás után (a méret változhatott a scale miatt)
            let info = decoder.info().ok_or("No info after decode")?;

            // Buffer konverzió (move semantics - pixels moved into form_raw)
            let img_buffer =
                image::RgbImage::from_raw(info.width as u32, info.height as u32, pixels)
                    .ok_or("Buffer mismatch")?;

            Ok(image::DynamicImage::ImageRgb8(img_buffer))
        },
    ));

    // Ha pánik volt vagy hiba, NEM próbálkozunk fallback-kel, mert az valószínűleg OOM miatt volt
    let img = match decode_result {
        Ok(Ok(image)) => image,
        Ok(Err(e)) => return Err(VisionError::ThumbnailError(format!("JPEG Error: {}", e))),
        Err(_) => {
            return Err(VisionError::ThumbnailError(
                "JPEG Decode Panic (OOM protection)".into(),
            ))
        }
    };

    // Végső resize ha szükséges (bár a scale() már megcsinálta nagyjából)
    let (w, h) = (img.width(), img.height());
    // Ha a scale() nem volt pontos, vagy valamiért nagyobb lett
    let resized = if w > max_size || h > max_size {
        let ratio = (max_size as f32) / (w.max(h) as f32);
        let nw = ((w as f32 * ratio) as u32).max(1);
        let nh = ((h as f32 * ratio) as u32).max(1);
        img.resize_exact(nw, nh, image::imageops::FilterType::Nearest)
    } else {
        img
    };

    let mut buffer = Cursor::new(Vec::with_capacity(20_000));
    resized
        .write_to(&mut buffer, image::ImageFormat::Jpeg)
        .map_err(|e| VisionError::ThumbnailError(format!("Encode: {}", e)))?;

    Ok(ThumbnailData {
        image_path: path_str.to_string(),
        data_base64: format!(
            "data:image/jpeg;base64,{}",
            STANDARD.encode(buffer.into_inner())
        ),
        width: resized.width(),
        height: resized.height(),
    })
}

// === Standard Image ===

fn extract_standard_image(path: &Path, max_size: u32) -> Result<ThumbnailData, VisionError> {
    let path_str = path.to_string_lossy().to_string();
    let img = image::open(path).map_err(|e| VisionError::ThumbnailError(format!("Open: {}", e)))?;

    let (w, h) = (img.width(), img.height());
    let ratio = (max_size as f32) / (w.max(h) as f32);
    let (nw, nh) = if ratio < 1.0 {
        (
            ((w as f32 * ratio) as u32).max(1),
            ((h as f32 * ratio) as u32).max(1),
        )
    } else {
        (w, h)
    };

    let resized = img.resize(nw, nh, image::imageops::FilterType::Triangle);
    let mut buffer = Cursor::new(Vec::with_capacity(30_000));
    resized
        .write_to(&mut buffer, image::ImageFormat::Jpeg)
        .map_err(|e| VisionError::ThumbnailError(format!("Encode: {}", e)))?;

    Ok(ThumbnailData {
        image_path: path_str,
        data_base64: format!(
            "data:image/jpeg;base64,{}",
            STANDARD.encode(buffer.into_inner())
        ),
        width: nw,
        height: nh,
    })
}

fn create_placeholder(path_str: &str) -> Result<ThumbnailData, VisionError> {
    let img = image::RgbImage::from_fn(120, 90, |_, _| image::Rgb([35u8, 35u8, 35u8]));
    let mut buffer = Cursor::new(Vec::with_capacity(5000));
    image::DynamicImage::ImageRgb8(img)
        .write_to(&mut buffer, image::ImageFormat::Jpeg)
        .map_err(|e| VisionError::ThumbnailError(e.to_string()))?;
    Ok(ThumbnailData {
        image_path: path_str.to_string(),
        data_base64: format!(
            "data:image/jpeg;base64,{}",
            STANDARD.encode(buffer.into_inner())
        ),
        width: 120,
        height: 90,
    })
}
