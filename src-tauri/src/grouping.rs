// VisionSelect AI - Image Grouping Module
// Intelligens csoportosítás időbélyeg és hasonlóság alapján
//
// A modul automatikusan felismeri a sorozatfelvételeket (burst mode),
// expozíció bracketeket és manuálisan létrehozott csoportokat.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::quality_analyzer::QualityScore;
use crate::types::ImageRecord;

// === Group Types ===

/// Kép csoport típusok
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupType {
    /// Sorozatfelvétel (burst mode)
    Burst,
    /// Expozíció bracket (HDR)
    Bracket,
    /// Time-lapse szekvencia
    TimeLapse,
    /// Felhasználó által létrehozott
    Manual,
    /// Egyedi képek (nem csoportosított)
    Single,
}

/// Kép csoport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGroup {
    /// Egyedi azonosító
    pub id: u64,
    /// Csoport neve (opcionális)
    pub name: Option<String>,
    /// Csoportba tartozó képek útvonalai
    pub images: Vec<PathBuf>,
    /// Csoport típusa
    pub group_type: GroupType,
    /// Legjobb kép (AI alapján)
    pub best_image: Option<PathBuf>,
    /// Első felvétel időpontja
    pub capture_start: Option<DateTime<Utc>>,
    /// Utolsó felvétel időpontja
    pub capture_end: Option<DateTime<Utc>>,
}

impl ImageGroup {
    pub fn new(id: u64, group_type: GroupType) -> Self {
        Self {
            id,
            name: None,
            images: Vec::new(),
            group_type,
            best_image: None,
            capture_start: None,
            capture_end: None,
        }
    }

    /// Kép hozzáadása a csoporthoz
    pub fn add_image(&mut self, path: PathBuf, capture_date: Option<DateTime<Utc>>) {
        self.images.push(path);

        if let Some(date) = capture_date {
            match self.capture_start {
                None => {
                    self.capture_start = Some(date);
                    self.capture_end = Some(date);
                }
                Some(start) => {
                    if date < start {
                        self.capture_start = Some(date);
                    }
                    if let Some(end) = self.capture_end {
                        if date > end {
                            self.capture_end = Some(date);
                        }
                    }
                }
            }
        }
    }

    /// Csoport időtartama
    pub fn duration(&self) -> Option<Duration> {
        match (self.capture_start, self.capture_end) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }

    /// Képek száma
    pub fn len(&self) -> usize {
        self.images.len()
    }

    /// Üres-e a csoport
    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }
}

/// Rangsorolt csoport (AI pontszámokkal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedGroup {
    /// Az eredeti csoport
    pub group: ImageGroup,
    /// Rangsorolt képek (legjobbtól a legrosszabbig)
    pub ranked_images: Vec<PathBuf>,
    /// Pontszámok képenként
    pub scores: HashMap<String, QualityScore>,
}

// === Grouping Algorithm ===

/// Csoportosítási konfiguráció
#[derive(Debug, Clone)]
pub struct GroupingConfig {
    /// Időbeli küszöb sorozatfelvételhez (másodperc)
    pub burst_threshold_secs: i64,
    /// Minimum képek száma egy csoporthoz
    pub min_group_size: usize,
    /// Maximum képek száma egy csoporthoz
    pub max_group_size: usize,
}

impl Default for GroupingConfig {
    fn default() -> Self {
        Self {
            burst_threshold_secs: 3,
            min_group_size: 2,
            max_group_size: 50,
        }
    }
}

/// Képek csoportosítása időbélyeg alapján
///
/// # Algoritmus
/// 1. Képek rendezése felvételi idő szerint
/// 2. Ha két egymást követő kép között < threshold másodperc van → egy csoportba
/// 3. Ha nincs capture_date, a kép önálló csoportba kerül
pub fn group_by_timestamp(images: &[ImageRecord], config: &GroupingConfig) -> Vec<ImageGroup> {
    if images.is_empty() {
        return Vec::new();
    }

    // Képek rendezése felvételi idő szerint
    let mut sorted_images: Vec<&ImageRecord> = images.iter().collect();
    sorted_images.sort_by(|a, b| {
        let date_a = a.exif.as_ref().and_then(|e| e.capture_date);
        let date_b = b.exif.as_ref().and_then(|e| e.capture_date);
        date_a.cmp(&date_b)
    });

    let mut groups: Vec<ImageGroup> = Vec::new();
    let mut current_group = ImageGroup::new(1, GroupType::Burst);
    let mut group_id: u64 = 1;

    for image in sorted_images {
        let capture_date = image.exif.as_ref().and_then(|e| e.capture_date);

        // Ha nincs dátum, egyedi csoportba kerül
        if capture_date.is_none() {
            // Befejezzük az aktuális csoportot ha van benne kép
            if !current_group.is_empty() {
                finalize_group(&mut current_group, config);
                groups.push(current_group);
                group_id += 1;
                current_group = ImageGroup::new(group_id, GroupType::Burst);
            }

            // Egyedi csoport létrehozása
            let mut single_group = ImageGroup::new(group_id, GroupType::Single);
            single_group.add_image(PathBuf::from(&image.file_path), None);
            groups.push(single_group);
            group_id += 1;
            current_group = ImageGroup::new(group_id, GroupType::Burst);
            continue;
        }

        let current_date = capture_date.unwrap();

        // Ellenőrizzük, hogy az aktuális csoportba tartozik-e
        if let Some(last_date) = current_group.capture_end {
            let diff = (current_date - last_date).num_seconds().abs();

            if diff <= config.burst_threshold_secs && current_group.len() < config.max_group_size {
                // Ugyanabba a csoportba tartozik
                current_group.add_image(PathBuf::from(&image.file_path), Some(current_date));
            } else {
                // Új csoport kezdése
                finalize_group(&mut current_group, config);
                groups.push(current_group);
                group_id += 1;

                current_group = ImageGroup::new(group_id, GroupType::Burst);
                current_group.add_image(PathBuf::from(&image.file_path), Some(current_date));
            }
        } else {
            // Első kép a csoportban
            current_group.add_image(PathBuf::from(&image.file_path), Some(current_date));
        }
    }

    // Utolsó csoport hozzáadása
    if !current_group.is_empty() {
        finalize_group(&mut current_group, config);
        groups.push(current_group);
    }

    groups
}

/// Csoport véglegesítése és típus meghatározása
fn finalize_group(group: &mut ImageGroup, config: &GroupingConfig) {
    if group.len() < config.min_group_size {
        group.group_type = GroupType::Single;
    } else if group.len() >= 3 {
        // TODO: Bracket detection alapján ISO/exposure változás
        group.group_type = GroupType::Burst;
    }
}

/// Csoport rangsorolása AI pontszámok alapján
pub fn rank_group_images(
    group: &ImageGroup,
    scores: &HashMap<String, QualityScore>,
) -> RankedGroup {
    let mut image_scores: Vec<(&PathBuf, f32)> = group
        .images
        .iter()
        .map(|path| {
            let path_str = path.to_string_lossy().to_string();
            let score = scores.get(&path_str).map(|s| s.overall).unwrap_or(0.0);
            (path, score)
        })
        .collect();

    // Rendezés csökkenő sorrendben (legjobb előre)
    image_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let ranked_images: Vec<PathBuf> = image_scores.iter().map(|(p, _)| (*p).clone()).collect();

    let best_image = ranked_images.first().cloned();

    let mut result_group = group.clone();
    result_group.best_image = best_image;

    // HashMap konverzió String kulcsokkal
    let scores_map: HashMap<String, QualityScore> = scores
        .iter()
        .filter(|(path, _)| group.images.iter().any(|p| p.to_string_lossy() == **path))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    RankedGroup {
        group: result_group,
        ranked_images,
        scores: scores_map,
    }
}

/// Egyedi képek kiszűrése (nem csoportosított)
pub fn filter_single_images(groups: &[ImageGroup]) -> Vec<PathBuf> {
    groups
        .iter()
        .filter(|g| g.group_type == GroupType::Single)
        .flat_map(|g| g.images.clone())
        .collect()
}

/// Burst csoportok kiszűrése (> 1 kép)
pub fn filter_burst_groups(groups: &[ImageGroup]) -> Vec<&ImageGroup> {
    groups
        .iter()
        .filter(|g| g.group_type == GroupType::Burst && g.len() > 1)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExifData;

    fn create_test_image(path: &str, capture_date: Option<DateTime<Utc>>) -> ImageRecord {
        ImageRecord {
            id: None,
            file_path: path.to_string(),
            filename: path.to_string(),
            extension: "cr3".to_string(),
            file_size: 1000,
            file_hash: None,
            thumbnail_path: None,
            exif: Some(ExifData {
                capture_date,
                ..Default::default()
            }),
            created_at: None,
        }
    }

    #[test]
    fn test_grouping_burst() {
        let base_time = Utc::now();

        let images = vec![
            create_test_image("img1.cr3", Some(base_time)),
            create_test_image("img2.cr3", Some(base_time + Duration::seconds(1))),
            create_test_image("img3.cr3", Some(base_time + Duration::seconds(2))),
            // Gap of 10 seconds - new group
            create_test_image("img4.cr3", Some(base_time + Duration::seconds(12))),
            create_test_image("img5.cr3", Some(base_time + Duration::seconds(13))),
        ];

        let config = GroupingConfig::default();
        let groups = group_by_timestamp(&images, &config);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].images.len(), 3);
        assert_eq!(groups[1].images.len(), 2);
    }

    #[test]
    fn test_grouping_single() {
        let base_time = Utc::now();

        let images = vec![
            create_test_image("img1.cr3", Some(base_time)),
            // 60 seconds gap - separate groups
            create_test_image("img2.cr3", Some(base_time + Duration::seconds(60))),
        ];

        let config = GroupingConfig::default();
        let groups = group_by_timestamp(&images, &config);

        // Both should be single groups (less than min_group_size of 2)
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].group_type, GroupType::Single);
        assert_eq!(groups[1].group_type, GroupType::Single);
    }
}
