//! Module pour gérer les barres de progression

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Créer une barre de progression pour le téléchargement
pub fn create_download_progress(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {spinner:.green}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Créer une barre de progression pour le traitement audio
pub fn create_audio_progress(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {spinner:.cyan}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Créer une barre de progression pour le splitting d'une piste
pub fn create_track_progress(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("  {msg} {spinner:.yellow}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
