//! Extraction de chapitres depuis la description d'une vidéo YouTube.
//!
//! Ce module détecte et parse les timestamps dans la description des vidéos
//! pour créer des chapitres lorsque les métadonnées YouTube n'en contiennent pas.

use crate::chapters::{parse_timestamp, Chapter};
use crate::error::{Result, YtcsError};
use crate::utils;
use regex::Regex;

/// Parse les chapitres depuis la description d'une vidéo.
///
/// Détecte les lignes contenant un timestamp suivi d'un titre.
/// Format attendu: [HH:MM:SS] - Titre ou HH:MM:SS - Titre
///
/// # Arguments
///
/// * `description` - La description de la vidéo
/// * `video_duration` - La durée totale de la vidéo en secondes
///
/// # Returns
///
/// Un vecteur de chapitres extraits, ou une erreur si aucun chapitre n'est trouvé
///
/// # Errors
///
/// Retourne une erreur si aucun chapitre valide n'est trouvé dans la description
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::chapters_from_description::parse_chapters_from_description;
///
/// let description = r#"
/// [00:00:00] - Introduction
/// [00:05:30] - Main Topic
/// [00:15:45] - Conclusion
/// "#;
///
/// let chapters = parse_chapters_from_description(description, 1200.0).unwrap();
/// assert_eq!(chapters.len(), 3);
/// ```
pub fn parse_chapters_from_description(
    description: &str,
    video_duration: f64,
) -> Result<Vec<Chapter>> {
    log::info!("Attempting to parse chapters from description");
    log::debug!("Video duration: {:.2}s", video_duration);
    log::debug!("Description length: {} characters", description.len());

    // Regex pour détecter: timestamp optionnel entre crochets + séparateur + titre
    // Formats supportés:
    // [00:00:00] - Title
    // [00:00] - Title
    // 00:00:00 - Title
    // 00:00 - Title
    // 00:00:00 Title
    // 1 - Title (0:00)
    // 2 - Title (4:24)
    let re = Regex::new(r"(?m)^\s*\[?(\d{1,2}:\d{2}(?::\d{2})?)\]?\s*[-–—:]?\s*(.+?)\s*$")
        .map_err(|e| YtcsError::ChapterError(format!("Regex error: {}", e)))?;

    // Regex pour le format avec numéro de piste au début: "1 - Title (0:00)"
    let re_track_format =
        Regex::new(r"(?m)^\s*(\d+)\s*[-–—]\s*(.+?)\s*\((\d{1,2}:\d{2}(?::\d{2})?)\)\s*$")
            .map_err(|e| YtcsError::ChapterError(format!("Regex error: {}", e)))?;

    let mut chapters_data: Vec<(f64, String)> = Vec::new();

    // Essayer d'abord le format avec numéro de piste: "1 - Title (0:00)"
    for cap in re_track_format.captures_iter(description) {
        if let (Some(_track_num_match), Some(title_match), Some(timestamp_match)) =
            (cap.get(1), cap.get(2), cap.get(3))
        {
            let timestamp_str = timestamp_match.as_str();
            let title = title_match.as_str().trim();

            // Ignorer les lignes vides ou trop courtes
            if title.is_empty() || title.len() < 2 {
                continue;
            }

            // Parser le timestamp
            if let Ok(start_time) = parse_timestamp(timestamp_str) {
                // Vérifier que le timestamp est dans la durée de la vidéo
                if start_time < video_duration {
                    // Nettoyer le titre
                    let clean_title = utils::sanitize_title(title);
                    if !clean_title.is_empty() {
                        chapters_data.push((start_time, clean_title));
                    }
                }
            }
        }
    }

    // Si aucun chapitre trouvé avec le format piste, essayer le format classique
    if chapters_data.is_empty() {
        for cap in re.captures_iter(description) {
            if let (Some(timestamp_match), Some(title_match)) = (cap.get(1), cap.get(2)) {
                let timestamp_str = timestamp_match.as_str();
                let title = title_match.as_str().trim();

                // Ignorer les lignes vides ou trop courtes
                if title.is_empty() || title.len() < 2 {
                    continue;
                }

                // Parser le timestamp
                if let Ok(start_time) = parse_timestamp(timestamp_str) {
                    // Vérifier que le timestamp est dans la durée de la vidéo
                    if start_time < video_duration {
                        // Nettoyer le titre
                        let clean_title = utils::sanitize_title(title);
                        if !clean_title.is_empty() {
                            chapters_data.push((start_time, clean_title));
                        }
                    }
                }
            }
        }
    }

    // Vérifier qu'on a trouvé au moins 2 chapitres
    if chapters_data.len() < 2 {
        return Err(YtcsError::ChapterError(
            "Not enough chapters found in description (need at least 2)".to_string(),
        ));
    }

    // Trier par timestamp
    chapters_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Créer les chapitres avec end_time
    let mut chapters = Vec::new();
    for i in 0..chapters_data.len() {
        let (start_time, title) = &chapters_data[i];
        let end_time = if i + 1 < chapters_data.len() {
            chapters_data[i + 1].0
        } else {
            video_duration
        };

        // Vérifier que le chapitre a une durée valide (au moins 1 seconde)
        if end_time > *start_time + 1.0 {
            chapters.push(Chapter::new(title.clone(), *start_time, end_time));
        }
    }

    if chapters.is_empty() {
        log::warn!("No valid chapters found in description");
        return Err(YtcsError::ChapterError(
            "No valid chapters found in description".to_string(),
        ));
    }

    log::info!(
        "Successfully parsed {} chapters from description",
        chapters.len()
    );
    Ok(chapters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chapters_with_brackets() {
        let description = r#"
[00:00:00] - The Observer's Paradox
[00:07:39] - The Keeper of the Keys
[00:12:49] - Chronos Awaits
[00:18:59] - Beneath a Veil of Stars
[00:26:34] - Quantum Echoes in the Dust
"#;
        let chapters = parse_chapters_from_description(description, 1800.0).unwrap();
        assert_eq!(chapters.len(), 5);
        assert_eq!(chapters[0].title, "The Observer's Paradox");
        assert_eq!(chapters[0].start_time, 0.0);
        assert_eq!(chapters[1].start_time, 459.0); // 7:39
    }

    #[test]
    fn test_parse_chapters_without_brackets() {
        let description = r#"
00:00 - Introduction
05:30 - Main Topic
15:45 - Conclusion
"#;
        let chapters = parse_chapters_from_description(description, 1200.0).unwrap();
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "Introduction");
        assert_eq!(chapters[1].start_time, 330.0); // 5:30
    }

    #[test]
    fn test_parse_chapters_mixed_format() {
        let description = r#"
Tracklist:
[0:00] Track One
[5:30] Track Two
10:15 Track Three
"#;
        let chapters = parse_chapters_from_description(description, 900.0).unwrap();
        assert_eq!(chapters.len(), 3);
    }

    #[test]
    fn test_parse_chapters_insufficient() {
        let description = "[00:00] Only One Track";
        let result = parse_chapters_from_description(description, 300.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_chapters_no_chapters() {
        let description = "This is a video description without any timestamps.";
        let result = parse_chapters_from_description(description, 300.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_chapters_track_number_format() {
        let description = r#"
1 - The Cornerstone of Some Dream (0:00)
2 - Architects of Inner Time (Part I) (4:24)
3 - The Ritual of the Octagonal Chamber (11:01)
4 - Colors at the Bottom of the Gesture (Instrumental) (17:52)
5 - The Ballad of the Hourglass Man (22:23)
6 - Mirror Against the Firmament (Suite in Three Parts) (26:43)
7 - The Navigation of Rational Ice (31:28)
8 - The Guardian of the Shadow Papyri (35:24)
9 - The Cycle of Chalk and Fine Sand (40:29)
10 - Song for the Submerged Mountains (44:11)
11 - The Filters of Chronos (48:35)
12 - Architects of Inner Time (Part II: The Awakening) (51:42)
"#;
        let chapters = parse_chapters_from_description(description, 3600.0).unwrap();
        assert_eq!(chapters.len(), 12);
        assert_eq!(chapters[0].title, "The Cornerstone of Some Dream");
        assert_eq!(chapters[0].start_time, 0.0);
        assert_eq!(chapters[1].title, "Architects of Inner Time (Part I)");
        assert_eq!(chapters[1].start_time, 264.0); // 4:24
        assert_eq!(
            chapters[11].title,
            "Architects of Inner Time (Part II_ The Awakening)"
        );
        assert_eq!(chapters[11].start_time, 3102.0); // 51:42
    }
}
