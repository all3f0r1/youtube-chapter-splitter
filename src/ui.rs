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
        "ytcs v0.10.3".dimmed()
    );
}

/// Afficher un en-tête de section
pub fn print_section_header(title: &str) {
    println!("{}", title.bright_cyan().bold());
    println!();
}

/// Afficher les informations de la vidéo
pub fn print_video_info(title: &str, duration: &str, tracks: usize) {
    // Nettoyer le titre des éléments inutiles
    let clean_title = clean_title(title);

    println!(
        "{} {}",
        "→".cyan().bold(),
        clean_title.bright_white().bold()
    );
    println!(
        "  {} {} {}",
        duration.dimmed(),
        "•".dimmed(),
        format!("{} tracks", tracks).dimmed()
    );
    println!();
}

/// Nettoyer le titre des éléments inutiles
fn clean_title(title: &str) -> String {
    let mut cleaned = title.to_string();

    // Patterns à supprimer
    let patterns = [
        "[Full Album]",
        "[FULL ALBUM]",
        "(Full Album)",
        "(FULL ALBUM)",
        "[Official Audio]",
        "[Official Video]",
        "(Official Audio)",
        "(Official Video)",
        "[HD]",
        "[4K]",
        "(HD)",
        "(4K)",
    ];

    for pattern in &patterns {
        cleaned = cleaned.replace(pattern, "");
    }

    // Nettoyer les espaces multiples et trim
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    cleaned
}

/// Afficher le statut de téléchargement
pub fn print_download_status(cover_status: Status, audio_status: Status) {
    println!("  {} Cover downloaded", status_icon(cover_status));
    println!("  {} Audio downloaded", status_icon(audio_status));
    println!();
}

/// Afficher le début du splitting
pub fn print_splitting_start() {
    println!("  {}\n", "Splitting tracks...".dimmed());
}

/// Afficher une piste
pub fn print_track(track: &TrackProgress) {
    println!(
        "  {} {:02} {} ({})",
        status_icon(track.status),
        track.number,
        track.title.bright_white(),
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
        assert_eq!(clean_title("Artist - Song [Full Album]"), "Artist - Song");
        assert_eq!(clean_title("Song (Official Audio) [HD]"), "Song");
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
