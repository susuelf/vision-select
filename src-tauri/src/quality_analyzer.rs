// VisionSelect AI - Image Quality Analyzer
// Képminőség elemzés: élesség, expozíció, kontraszt
//
// Ez a modul NEM használ ML modelleket, hanem klasszikus képfeldolgozási
// algoritmusokat (Laplacian variance, hisztogram elemzés) a gyors futás érdekében.

use image::{DynamicImage, GenericImageView, GrayImage, Luma, Rgb, RgbImage};

use crate::types::VisionError;
use serde::{Deserialize, Serialize};

// === Quality Score Types ===

/// Teljes minőségi pontszám egy képhez
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScore {
    /// Élesség pontszám (0.0 - 1.0)
    pub sharpness: f32,
    /// Expozíció pontszám (0.0 - 1.0)
    pub exposure: f32,
    /// Kontraszt pontszám (0.0 - 1.0)
    pub contrast: f32,
    /// Zaj becslés (0.0 = zajos, 1.0 = tiszta)
    pub noise_estimate: f32,
    /// Összesített pontszám (súlyozott átlag)
    pub overall: f32,
}

impl Default for QualityScore {
    fn default() -> Self {
        Self {
            sharpness: 0.5,
            exposure: 0.5,
            contrast: 0.5,
            noise_estimate: 0.5,
            overall: 0.5,
        }
    }
}

/// Expozíció részletes elemzés
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureAnalysis {
    /// Túlexponált pixelek aránya (0.0 - 1.0)
    pub overexposed_ratio: f32,
    /// Alulexponált pixelek aránya (0.0 - 1.0) 
    pub underexposed_ratio: f32,
    /// Átlagos fényerő (0 - 255)
    pub mean_brightness: f32,
    /// Megfelelő expozíció-e
    pub is_properly_exposed: bool,
    /// Expozíció pontszám (0.0 - 1.0)
    pub score: f32,
}

/// Súlyozási konfiguráció különböző képtípusokhoz
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    pub sharpness: f32,
    pub exposure: f32,
    pub contrast: f32,
    pub noise: f32,
}

impl Default for ScoringWeights {
    fn default() -> Self {
        Self {
            sharpness: 0.40,
            exposure: 0.30,
            contrast: 0.20,
            noise: 0.10,
        }
    }
}

// === Quality Analyzer ===

pub struct QualityAnalyzer {
    weights: ScoringWeights,
}

impl Default for QualityAnalyzer {
    fn default() -> Self {
        Self {
            weights: ScoringWeights::default(),
        }
    }
}

impl QualityAnalyzer {
    /// Új analyzer létrehozása egyedi súlyozással
    pub fn with_weights(weights: ScoringWeights) -> Self {
        Self { weights }
    }

    /// Teljes minőségi elemzés egy képen
    pub fn analyze(&self, img: &DynamicImage) -> Result<QualityScore, VisionError> {
        // Konvertálás szükséges formátumokba
        let gray = img.to_luma8();
        let rgb = img.to_rgb8();

        // Individuális metrikák számítása
        let sharpness = analyze_sharpness(&gray);
        let exposure_analysis = analyze_exposure(&rgb);
        let contrast = analyze_contrast(&gray);
        let noise_estimate = estimate_noise(&gray);

        // Összesített pontszám
        let overall = self.weights.sharpness * sharpness
            + self.weights.exposure * exposure_analysis.score
            + self.weights.contrast * contrast
            + self.weights.noise * noise_estimate;

        Ok(QualityScore {
            sharpness,
            exposure: exposure_analysis.score,
            contrast,
            noise_estimate,
            overall: overall.clamp(0.0, 1.0),
        })
    }

    /// Gyors elemzés (csak élesség és expozíció)
    pub fn analyze_fast(&self, img: &DynamicImage) -> Result<QualityScore, VisionError> {
        let gray = img.to_luma8();
        let rgb = img.to_rgb8();

        let sharpness = analyze_sharpness(&gray);
        let exposure_analysis = analyze_exposure(&rgb);

        // Egyszerűsített overall (csak 2 metrika)
        let overall = 0.6 * sharpness + 0.4 * exposure_analysis.score;

        Ok(QualityScore {
            sharpness,
            exposure: exposure_analysis.score,
            contrast: 0.5, // Alapértelmezett
            noise_estimate: 0.5,
            overall: overall.clamp(0.0, 1.0),
        })
    }
}

// === Sharpness Analysis (Laplacian Variance) ===

/// Élesség mérése Laplacian Variance algoritmussal
/// 
/// A Laplacian operátor kiemeli a kép éleit. A variancia magasabb értéke
/// élesebb képet jelent.
pub fn analyze_sharpness(gray: &GrayImage) -> f32 {
    let (width, height) = gray.dimensions();
    
    if width < 3 || height < 3 {
        return 0.0;
    }

    let mut sum: f64 = 0.0;
    let mut sum_sq: f64 = 0.0;
    let mut count: u64 = 0;

    // Laplacian kernel: [0, 1, 0], [1, -4, 1], [0, 1, 0]
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center = gray.get_pixel(x, y).0[0] as i32;
            let top = gray.get_pixel(x, y - 1).0[0] as i32;
            let bottom = gray.get_pixel(x, y + 1).0[0] as i32;
            let left = gray.get_pixel(x - 1, y).0[0] as i32;
            let right = gray.get_pixel(x + 1, y).0[0] as i32;

            let laplacian = (top + bottom + left + right - 4 * center).abs();
            
            sum += laplacian as f64;
            sum_sq += (laplacian as f64).powi(2);
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }

    let mean = sum / count as f64;
    let variance = (sum_sq / count as f64) - mean.powi(2);

    // Normalizálás: tipikus értékek 0-2000 körül vannak
    // Jól éles kép: ~500-1500 variancia
    let normalized = (variance / 1500.0).clamp(0.0, 1.0) as f32;
    
    normalized
}

// === Exposure Analysis (Histogram-based) ===

/// Expozíció elemzése hisztogram alapján
pub fn analyze_exposure(rgb: &RgbImage) -> ExposureAnalysis {
    let (width, height) = rgb.dimensions();
    let total_pixels = (width * height) as f64;
    
    if total_pixels == 0.0 {
        return ExposureAnalysis {
            overexposed_ratio: 0.0,
            underexposed_ratio: 0.0,
            mean_brightness: 128.0,
            is_properly_exposed: true,
            score: 0.5,
        };
    }

    let mut histogram = [0u64; 256];
    let mut brightness_sum: f64 = 0.0;

    // Hisztogram építése (luminance alapján)
    for pixel in rgb.pixels() {
        // ITU-R BT.601 luminance formula
        let luminance = (0.299 * pixel.0[0] as f64
            + 0.587 * pixel.0[1] as f64
            + 0.114 * pixel.0[2] as f64) as u8;
        
        histogram[luminance as usize] += 1;
        brightness_sum += luminance as f64;
    }

    let mean_brightness = (brightness_sum / total_pixels) as f32;

    // Túl/alulexponált pixelek számítása
    let overexposed: u64 = histogram[250..=255].iter().sum();
    let underexposed: u64 = histogram[0..=5].iter().sum();

    let overexposed_ratio = (overexposed as f64 / total_pixels) as f32;
    let underexposed_ratio = (underexposed as f64 / total_pixels) as f32;

    // Expozíció megfelelő, ha:
    // - Kevesebb mint 5% túlexponált
    // - Kevesebb mint 10% alulexponált
    // - Átlag közel 128-hoz (+-50)
    let is_properly_exposed = overexposed_ratio < 0.05
        && underexposed_ratio < 0.10
        && (78.0..178.0).contains(&mean_brightness);

    // Pontszám számítása
    // Ideális: 0% túlexponált, 0% alulexponált, 128 átlag
    let overexposed_penalty = (overexposed_ratio * 5.0).min(1.0);
    let underexposed_penalty = (underexposed_ratio * 3.0).min(1.0);
    let brightness_penalty = ((mean_brightness - 128.0).abs() / 128.0).min(1.0);

    let score = (1.0 - overexposed_penalty * 0.4 - underexposed_penalty * 0.3 - brightness_penalty * 0.3)
        .clamp(0.0, 1.0);

    ExposureAnalysis {
        overexposed_ratio,
        underexposed_ratio,
        mean_brightness,
        is_properly_exposed,
        score,
    }
}

// === Contrast Analysis ===

/// Kontraszt elemzése (standard deviation of luminance)
pub fn analyze_contrast(gray: &GrayImage) -> f32 {
    let (width, height) = gray.dimensions();
    let total_pixels = (width * height) as f64;

    if total_pixels == 0.0 {
        return 0.5;
    }

    let mut sum: f64 = 0.0;
    let mut sum_sq: f64 = 0.0;

    for pixel in gray.pixels() {
        let val = pixel.0[0] as f64;
        sum += val;
        sum_sq += val * val;
    }

    let mean = sum / total_pixels;
    let variance = (sum_sq / total_pixels) - mean.powi(2);
    let std_dev = variance.sqrt();

    // Normalizálás: tipikus std_dev 20-80 között van jó kontrasztú képeknél
    // Alacsony (<30): lapos, nincs kontraszt
    // Magas (>70): jó kontraszt
    let normalized = ((std_dev - 20.0) / 60.0).clamp(0.0, 1.0) as f32;

    normalized
}

// === Noise Estimation ===

/// Egyszerű zajbecslés (Laplacian Median Absolute Deviation)
/// 
/// A módszer a kép magas frekvenciás komponenseit vizsgálja.
/// Zajos képnél több a véletlenszerű magas frekvenciás jel.
pub fn estimate_noise(gray: &GrayImage) -> f32 {
    let (width, height) = gray.dimensions();

    if width < 3 || height < 3 {
        return 0.5;
    }

    let mut laplacian_values: Vec<i32> = Vec::with_capacity((width * height) as usize);

    // Laplacian értékek gyűjtése
    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let center = gray.get_pixel(x, y).0[0] as i32;
            let top = gray.get_pixel(x, y - 1).0[0] as i32;
            let bottom = gray.get_pixel(x, y + 1).0[0] as i32;
            let left = gray.get_pixel(x - 1, y).0[0] as i32;
            let right = gray.get_pixel(x + 1, y).0[0] as i32;

            let laplacian = (top + bottom + left + right - 4 * center).abs();
            laplacian_values.push(laplacian);
        }
    }

    if laplacian_values.is_empty() {
        return 0.5;
    }

    // Medián számítása
    laplacian_values.sort_unstable();
    let median = laplacian_values[laplacian_values.len() / 2];

    // Normalizálás: alacsony medián = tiszta kép, magas medián = zajos
    // Tipikus értékek: tiszta kép ~5-15, zajos kép ~30-60
    let noise_level = (median as f32) / 50.0;
    
    // Invertálás: magas pontszám = tiszta kép
    (1.0 - noise_level).clamp(0.0, 1.0)
}

// === Batch Analysis ===

/// Több kép elemzése párhuzamosan
pub fn analyze_batch(
    images: &[(String, DynamicImage)],
) -> Vec<(String, Result<QualityScore, VisionError>)> {
    use rayon::prelude::*;

    let analyzer = QualityAnalyzer::default();

    images
        .par_iter()
        .map(|(path, img)| {
            let result = analyzer.analyze(img);
            (path.clone(), result)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_exposure_analysis() {
        // Teljesen fekete kép
        let black = RgbImage::from_fn(100, 100, |_, _| Rgb([0, 0, 0]));
        let analysis = analyze_exposure(&black);
        assert!(analysis.underexposed_ratio > 0.9);
        assert!(analysis.score < 0.5);

        // Teljesen fehér kép
        let white = RgbImage::from_fn(100, 100, |_, _| Rgb([255, 255, 255]));
        let analysis = analyze_exposure(&white);
        assert!(analysis.overexposed_ratio > 0.9);
        assert!(analysis.score < 0.5);

        // Szürke kép (ideális)
        let gray = RgbImage::from_fn(100, 100, |_, _| Rgb([128, 128, 128]));
        let analysis = analyze_exposure(&gray);
        assert!(analysis.is_properly_exposed);
        assert!(analysis.score > 0.7);
    }
}
