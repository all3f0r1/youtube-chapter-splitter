//! Module d'interface utilisateur minimaliste
//!
//! Design: Pragmatique • Direct • Classe
//! - Pas de bordures inutiles
//! - Info essentielle seulement
//! - Espacement naturel
//! - Statut clair avec ✓/✗

use colored::*;
use std::io::{self, Write};

/// Statut d'une opération
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// En attente
    Pending,
    /// En cours
    InProgress,
    /// Réussi (✓)
    Success,
    /// Échoué (✗)
    Failed,
}

/// Progression d'une piste
#[derive(Debug, Clone)]
pub struct TrackProgress {
    pub number: usize,
    pub title: String,
    pub status: Status,
    pub duration: String,
}

/// Icône de statut
fn status_icon(status: Status) -> ColoredString {
    match status {
        Status::Pending => " ".dimmed(),
        Status::InProgress => "⏳".yellow(),
        Status::Success => "✓".green(),
        Status::Failed => "✗".red(),
    }
}

/// Afficher l'en-tête minimal
pub fn print_header() {
    println!(
        "{}
",
        "ytcs v0.10.8".dimmed()
    );
}

/// Afficher un en-tête de section
pub fn print_section_header(title: &str) {
    println!("{}", title.bright_cyan().bold());
    println!();
}

/// Afficher les informations de la vidéo
pub fn print_video_info(
    title: &str,
    duration: &str,
    tracks: usize,
    from_description: bool,
    silence_detection: bool,
) {
    // Nettoyer le titre des éléments inutiles
    let clean_title = clean_title(title);

    println!(
        "{} {}",
        "→".cyan().bold(),
        clean_title.bright_white().bold()
    );

    // Affichage du nombre de tracks
    let tracks_display = if silence_detection {
        "? tracks → silence detection mode".to_string()
    } else if tracks > 0 {
        format!("{} tracks", tracks)
    } else if from_description {
        "checking description...".to_string()
    } else {
        "? tracks".to_string()
    };

    println!(
        "  {} {} {}",
        duration.dimmed(),
        "•".dimmed(),
        tracks_display.dimmed()
    );
    println!();
}

/// Nettoyer le titre pour qu'il corresponde au nom de dossier
pub fn clean_title(title: &str) -> String {
    use crate::utils;

    // Utiliser la même logique que clean_folder_name pour avoir un affichage cohérent
    utils::clean_folder_name(title)
}

/// Afficher le statut de téléchargement du cover
pub fn print_cover_status(cover_status: Status) {
    println!("  {} Cover downloaded", status_icon(cover_status));
}

/// Afficher une piste avec le format complet
pub fn print_track(track: &TrackProgress, artist: &str, album: &str, filename_format: &str) {
    // Construire le nom de fichier selon le format
    let formatted_name = filename_format
        .replace("{track}", &format!("{:02}", track.number))
        .replace("{title}", &track.title)
        .replace("{artist}", artist)
        .replace("{album}", album);

    println!(
        "  {} {} ({})",
        status_icon(track.status),
        formatted_name.bright_white(),
        track.duration.dimmed()
    );
    io::stdout().flush().ok();
}

/// Afficher un message de succès final
pub fn print_success(output_dir: &str) {
    println!();
    println!(
        "{} {} {}",
        "✓".green().bold(),
        "Done".green(),
        format!("→ {}", output_dir).bright_blue()
    );
    println!();
}

/// Afficher un message d'erreur
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Afficher un avertissement
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow(), message.yellow());
}

/// Afficher un message d'info
pub fn print_info(message: &str) {
    println!("  {}", message.dimmed());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
        // clean_title utilise maintenant clean_folder_name qui capitalise
        assert_eq!(clean_title("Artist - Song [Full Album]"), "Artist - Song");
        assert_eq!(clean_title("Normal Title"), "Normal Title");
    }

    #[test]
    fn test_status_icon() {
        let success = status_icon(Status::Success);
        let failed = status_icon(Status::Failed);
        assert!(!success.is_empty());
        assert!(!failed.is_empty());
    }
}
