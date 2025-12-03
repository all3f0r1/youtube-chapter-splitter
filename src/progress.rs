//! Module pour gérer les barres de progression

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Constante pour le taux de rafraîchissement des barres de progression
const PROGRESS_TICK_RATE_MS: u64 = 100;

/// Type de barre de progression
#[derive(Debug, Clone, Copy)]
pub enum ProgressType {
    /// Barre de progression pour le téléchargement
    Download,
    /// Barre de progression pour le traitement audio
    Audio,
    /// Barre de progression pour le splitting d'une piste
    Track,
}

/// Créer une barre de progression selon le type spécifié
///
/// # Arguments
///
/// * `message` - Message à afficher
/// * `progress_type` - Type de barre de progression
///
/// # Returns
///
/// Une barre de progression configurée
///
/// # Examples
///
/// ```no_run
/// use youtube_chapter_splitter::progress::{create_progress, ProgressType};
///
/// let pb = create_progress("Downloading...", ProgressType::Download);
/// // ... faire quelque chose ...
/// pb.finish_and_clear();
/// ```
pub fn create_progress(message: &str, progress_type: ProgressType) -> ProgressBar {
    let pb = ProgressBar::new_spinner();

    let template = match progress_type {
        ProgressType::Download => "{msg} {spinner:.green}",
        ProgressType::Audio => "{msg} {spinner:.cyan}",
        ProgressType::Track => "  {msg} {spinner:.yellow}",
    };

    pb.set_style(ProgressStyle::default_spinner().template(template).unwrap());
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(PROGRESS_TICK_RATE_MS));
    pb
}

/// Créer une barre de progression pour le téléchargement
///
/// # Arguments
///
/// * `message` - Message à afficher
///
/// # Returns
///
/// Une barre de progression configurée pour le téléchargement
pub fn create_download_progress(message: &str) -> ProgressBar {
    create_progress(message, ProgressType::Download)
}

/// Créer une barre de progression pour le traitement audio
///
/// # Arguments
///
/// * `message` - Message à afficher
///
/// # Returns
///
/// Une barre de progression configurée pour le traitement audio
pub fn create_audio_progress(message: &str) -> ProgressBar {
    create_progress(message, ProgressType::Audio)
}

/// Créer une barre de progression pour le splitting d'une piste
///
/// # Arguments
///
/// * `message` - Message à afficher
///
/// # Returns
///
/// Une barre de progression configurée pour le splitting
pub fn create_track_progress(message: &str) -> ProgressBar {
    create_progress(message, ProgressType::Track)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_download_progress() {
        let pb = create_download_progress("Testing download");
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_create_audio_progress() {
        let pb = create_audio_progress("Testing audio");
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_create_track_progress() {
        let pb = create_track_progress("Testing track");
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_create_progress_with_type() {
        let pb = create_progress("Testing", ProgressType::Download);
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }
}
