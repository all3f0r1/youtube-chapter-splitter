//! Gestion des chapitres de vidéos YouTube.
//!
//! Ce module fournit les structures et fonctions pour manipuler les chapitres
//! extraits des vidéos YouTube.

use crate::error::{Result, YtcsError};
use crate::utils;
use serde::{Deserialize, Serialize};

/// Représente un chapitre d'une vidéo YouTube.
///
/// Un chapitre est défini par un titre et une plage temporelle (début et fin).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
}

impl Chapter {
    /// Crée un nouveau chapitre.
    ///
    /// # Arguments
    ///
    /// * `title` - Le titre du chapitre
    /// * `start_time` - Le temps de début en secondes (doit être >= 0)
    /// * `end_time` - Le temps de fin en secondes (doit être > start_time)
    ///
    /// # Panics
    ///
    /// Panique si start_time < 0 ou si end_time <= start_time
    pub fn new(title: String, start_time: f64, end_time: f64) -> Self {
        assert!(
            start_time >= 0.0,
            "start_time must be >= 0, got {}",
            start_time
        );
        assert!(
            end_time > start_time,
            "end_time ({}) must be > start_time ({})",
            end_time,
            start_time
        );

        Self {
            title,
            start_time,
            end_time,
        }
    }

    /// Calcule la durée du chapitre en secondes.
    ///
    /// # Returns
    ///
    /// La durée du chapitre (end_time - start_time)
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Nettoie le titre du chapitre pour l'utiliser comme nom de fichier.
    ///
    /// Délègue le traitement à [`utils::sanitize_title`].
    ///
    /// # Returns
    ///
    /// Un titre nettoyé, sûr pour une utilisation comme nom de fichier
    pub fn sanitize_title(&self) -> String {
        utils::sanitize_title(&self.title)
    }
}

/// Parses chapters from yt-dlp JSON output.
///
/// Extrait les chapitres depuis les métadonnées JSON d'une vidéo YouTube.
///
/// # Arguments
///
/// * `json_str` - La chaîne JSON contenant les métadonnées de la vidéo
///
/// # Returns
///
/// Un vecteur de chapitres extraits, ou une erreur si le parsing échoue
///
/// # Errors
///
/// Returns an error if :
/// - Le JSON est mal formaté
/// - Le champ "chapters" est absent
/// - Les champs start_time ou end_time sont invalides
pub fn parse_chapters_from_json(json_str: &str) -> Result<Vec<Chapter>> {
    let data: serde_json::Value = serde_json::from_str(json_str)?;

    let chapters_array = data["chapters"]
        .as_array()
        .ok_or_else(|| YtcsError::ChapterError("No chapters found".to_string()))?;

    let mut chapters = Vec::new();
    for (i, chapter) in chapters_array.iter().enumerate() {
        let title = chapter["title"]
            .as_str()
            .unwrap_or(&format!("Track {}", i + 1))
            .to_string();

        let start_time = chapter["start_time"]
            .as_f64()
            .ok_or_else(|| YtcsError::ChapterError("Invalid start_time".to_string()))?;

        let end_time = chapter["end_time"]
            .as_f64()
            .ok_or_else(|| YtcsError::ChapterError("Invalid end_time".to_string()))?;

        chapters.push(Chapter::new(title, start_time, end_time));
    }

    Ok(chapters)
}

/// Parses a timestamp in format HH:MM:SS, MM:SS ou SS.
///
/// # Arguments
///
/// * `timestamp` - Le timestamp à parser (ex: "1:23:45", "5:30", "42")
///
/// # Returns
///
/// Le nombre de secondes correspondant au timestamp
///
/// # Errors
///
/// Returns an error if the timestamp format is invalid
pub fn parse_timestamp(timestamp: &str) -> Result<f64> {
    let parts: Vec<&str> = timestamp.split(':').collect();

    let seconds = match parts.len() {
        1 => parts[0]
            .parse::<f64>()
            .map_err(|_| YtcsError::ChapterError("Invalid timestamp format".to_string()))?,
        2 => {
            let minutes = parts[0]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid minutes".to_string()))?;
            let seconds = parts[1]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid seconds".to_string()))?;
            minutes * 60.0 + seconds
        }
        3 => {
            let hours = parts[0]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid hours".to_string()))?;
            let minutes = parts[1]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid minutes".to_string()))?;
            let seconds = parts[2]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid seconds".to_string()))?;
            hours * 3600.0 + minutes * 60.0 + seconds
        }
        _ => {
            return Err(YtcsError::ChapterError(
                "Invalid timestamp format".to_string(),
            ));
        }
    };

    Ok(seconds)
}

/// Formate un nombre de secondes en timestamp HH:MM:SS ou MM:SS.
///
/// # Arguments
///
/// * `seconds` - Le nombre de secondes à formater
///
/// # Returns
///
/// Un timestamp formaté (HH:MM:SS si >= 1h, sinon MM:SS)
pub fn format_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}
