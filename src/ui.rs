//! Module d'interface utilisateur (TUI) moderne
//!
//! Ce module fournit une interface utilisateur en mode texte avec :
//! - Indicateurs visuels de statut (✓/✗/⏳/⏸️)
//! - Cadres et bordures adaptatifs
//! - Barres de progression
//! - Adaptation automatique à la largeur du terminal

use colored::*;
use std::io::{self, Write};

/// Statut d'une opération
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// En attente (⏸️)
    Pending,
    /// En cours (⏳)
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
    pub progress: u8, // 0-100
}

/// Obtenir la largeur du terminal (avec fallback à 80)
fn terminal_width() -> usize {
    term_size::dimensions()
        .map(|(w, _)| w)
        .unwrap_or(80)
        .max(60) // Minimum 60 caractères
        .min(120) // Maximum 120 caractères pour la lisibilité
}

/// Icône de statut avec couleur
fn status_icon(status: Status) -> ColoredString {
    match status {
        Status::Pending => "⏸️ ".dimmed(),
        Status::InProgress => "⏳".yellow(),
        Status::Success => "✓".green().bold(),
        Status::Failed => "✗".red().bold(),
    }
}

/// Tronquer une chaîne à une longueur maximale
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        format!("{:<width$}", s, width = max_len)
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

/// Créer une mini barre de progression
fn create_mini_progress_bar(percent: u8, width: usize) -> String {
    let filled = (percent as usize * width) / 100;
    let empty = width.saturating_sub(filled);
    format!(
        "[{}{}]",
        "█".repeat(filled).cyan(),
        "░".repeat(empty).dimmed()
    )
}

/// Afficher l'en-tête principal
pub fn print_header() {
    let width = terminal_width();
    let title = " YouTube Chapter Splitter v0.9.0 ";
    let padding = (width.saturating_sub(title.len()).saturating_sub(4)) / 2;
    
    println!("╔{}╗", "═".repeat(width.saturating_sub(2)));
    println!(
        "║{}{}{}║",
        " ".repeat(padding),
        title.bold(),
        " ".repeat(width.saturating_sub(padding + title.len() + 2))
    );
    println!("╚{}╝", "═".repeat(width.saturating_sub(2)));
    println!();
}

/// Afficher les informations de la vidéo
pub fn print_video_info(title: &str, duration: &str, tracks: usize) {
    println!("📹 Video: {}", title.bright_blue().bold());
    println!("⏱️  Duration: {}", duration.cyan());
    println!("🎵 Tracks: {}", tracks.to_string().yellow().bold());
    println!();
}

/// Afficher la section de téléchargement
pub fn print_download_section(cover_status: Status, audio_status: Status) {
    let width = terminal_width();
    let title = " Download Progress ";
    let title_padding = width.saturating_sub(title.len() + 3);
    
    println!("┌─{}{}", title.bold(), "─".repeat(title_padding));
    
    // Cover line
    let cover_text = format!("{} Cover art", status_icon(cover_status));
    let cover_plain_len = 10; // "⏸️  Cover art" sans couleurs
    let padding = width.saturating_sub(cover_plain_len + 5);
    println!("│ {}{} │", cover_text, " ".repeat(padding));
    
    // Audio line
    let audio_text = format!("{} Audio file", status_icon(audio_status));
    let audio_plain_len = 11; // "⏸️  Audio file" sans couleurs
    let padding = width.saturating_sub(audio_plain_len + 5);
    println!("│ {}{} │", audio_text, " ".repeat(padding));
    
    println!("└{}┘", "─".repeat(width.saturating_sub(2)));
    println!();
    
    // Flush pour afficher immédiatement
    io::stdout().flush().ok();
}

/// Afficher la section de découpage des pistes
pub fn print_track_section(tracks: &[TrackProgress]) {
    let width = terminal_width();
    let title = " Track Splitting ";
    let title_padding = width.saturating_sub(title.len() + 3);
    
    println!("┌─{}{}", title.bold(), "─".repeat(title_padding));
    
    for track in tracks {
        // Calculer les largeurs disponibles
        let prefix_len = 6; // "│ ⏸️  "
        let number_len = 5; // "01 - "
        let suffix_len = 2; // " │"
        let progress_bar_width = 10; // [████████░░]
        let percent_len = 5; // " 100%"
        
        let available_width = width
            .saturating_sub(prefix_len)
            .saturating_sub(number_len)
            .saturating_sub(progress_bar_width)
            .saturating_sub(percent_len)
            .saturating_sub(suffix_len)
            .saturating_sub(2); // Espaces
        
        let title_truncated = truncate(&track.title, available_width);
        let progress_bar = create_mini_progress_bar(track.progress, 8);
        
        println!(
            "│ {} {:02} - {} {} {:>3}% │",
            status_icon(track.status),
            track.number,
            title_truncated,
            progress_bar,
            track.progress
        );
    }
    
    println!("└{}┘", "─".repeat(width.saturating_sub(2)));
    println!();
    
    // Flush pour afficher immédiatement
    io::stdout().flush().ok();
}

/// Effacer les lignes précédentes (pour mise à jour en place)
pub fn clear_lines(count: usize) {
    for _ in 0..count {
        print!("\x1b[1A"); // Monter d'une ligne
        print!("\x1b[2K"); // Effacer la ligne
    }
    io::stdout().flush().ok();
}

/// Afficher un message de succès final
pub fn print_success(message: &str, output_dir: &str) {
    println!("{}", message.green().bold());
    println!("📁 Output: {}", output_dir.bright_blue());
    println!();
}

/// Afficher un message d'erreur
pub fn print_error(message: &str) {
    eprintln!("{}", format!("✗ Error: {}", message).red().bold());
}

/// Afficher un avertissement
pub fn print_warning(message: &str) {
    eprintln!("{}", format!("⚠ Warning: {}", message).yellow());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("Hello", 10), "Hello     ");
        assert_eq!(truncate("Hello World", 8), "Hello...");
        assert_eq!(truncate("Test", 4), "Test");
    }

    #[test]
    fn test_progress_bar() {
        let bar = create_mini_progress_bar(50, 8);
        assert!(bar.contains('['));
        assert!(bar.contains(']'));
    }

    #[test]
    fn test_status_icon() {
        let pending = status_icon(Status::Pending);
        let success = status_icon(Status::Success);
        assert!(!pending.is_empty());
        assert!(!success.is_empty());
    }
}
