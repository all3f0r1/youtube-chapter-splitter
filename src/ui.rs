//! Minimalist user interface module
//!
//! Design: Pragmatic • Direct • Classy
//! - No unnecessary borders
//! - Essential info only
//! - Natural spacing
//! - Clear status with ✓/✗

use colored::*;
use std::io::{self, Write};

/// Print a blank line for explicit section separation
pub fn print_blank_line() {
    println!();
}

/// Output mode for UI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    /// Colored terminal output (default)
    Colored,
    /// Plain text output for --cli mode or piping
    Plain,
}

/// Operation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Pending
    Pending,
    /// In progress
    InProgress,
    /// Success (✓)
    Success,
    /// Failed (✗)
    Failed,
}

/// Track progress
#[derive(Debug, Clone)]
pub struct TrackProgress {
    pub number: usize,
    pub title: String,
    pub status: Status,
    pub duration: String,
}

/// Get status icon (plain or colored)
fn status_icon(status: Status) -> ColoredString {
    match status {
        Status::Pending => " ".dimmed(),
        Status::InProgress => "⏳".yellow(),
        Status::Success => "✓".green(),
        Status::Failed => "✗".red(),
    }
}

/// Plain text presenter for --cli mode
pub struct PlainTextPresenter {
    output_mode: OutputMode,
}

impl PlainTextPresenter {
    pub fn new() -> Self {
        Self {
            output_mode: OutputMode::Plain,
        }
    }

    pub fn with_output_mode(mut self, mode: OutputMode) -> Self {
        self.output_mode = mode;
        self
    }

    /// Print a message with ERROR prefix
    pub fn error(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => eprintln!("ERROR: {}", message),
            OutputMode::Colored => eprintln!("{} {}", "✗".red().bold(), message.red()),
        }
    }

    /// Print a message with WARNING prefix
    pub fn warning(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => eprintln!("WARNING: {}", message),
            OutputMode::Colored => eprintln!("{} {}", "⚠".yellow(), message.yellow()),
        }
    }

    /// Print info message
    pub fn info(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => println!("INFO: {}", message),
            OutputMode::Colored => println!("  {}", message.dimmed()),
        }
    }

    /// Print success message
    pub fn success(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => println!("OK: {}", message),
            OutputMode::Colored => println!("{} {}", "✓".green().bold(), message.green()),
        }
    }

    /// Print header
    pub fn header(&self) {
        match self.output_mode {
            OutputMode::Plain => println!("ytcs v{}", env!("CARGO_PKG_VERSION")),
            OutputMode::Colored => println!(
                "{}",
                format!("ytcs v{}", env!("CARGO_PKG_VERSION")).dimmed()
            ),
        }
    }

    /// Print section header
    pub fn section_header(&self, title: &str) {
        match self.output_mode {
            OutputMode::Plain => {
                println!("=== {} ===", title);
            }
            OutputMode::Colored => {
                println!("{}", title.bright_cyan().bold());
            }
        }
    }

    /// Print video info
    pub fn video_info(&self, title: &str, duration: &str, tracks: usize) {
        match self.output_mode {
            OutputMode::Plain => {
                println!("Title: {}", clean_title(title));
                println!("Duration: {}", duration);
                println!("Tracks: {}", tracks);
            }
            OutputMode::Colored => {
                print_video_info(title, duration, tracks, false, false);
            }
        }
    }

    /// Print progress for multiple URLs
    pub fn progress(&self, current: usize, total: usize, message: &str) {
        match self.output_mode {
            OutputMode::Plain => {
                println!("[{}/{}] {}", current, total, message);
            }
            OutputMode::Colored => {
                println!("  {}/{}: {}", current, total, message.dimmed());
            }
        }
    }
}

impl Default for PlainTextPresenter {
    fn default() -> Self {
        Self::new()
    }
}

/// Display minimal header
pub fn print_header() {
    println!(
        "{}
",
        format!("ytcs v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}

/// Display section header
pub fn print_section_header(title: &str) {
    println!("{}", title.bright_cyan().bold());
}

/// Display video information
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

/// Clean title to match folder name
pub fn clean_title(title: &str) -> String {
    use crate::utils;

    // Utiliser la même logique que clean_folder_name pour avoir un affichage cohérent
    utils::clean_folder_name(title)
}

/// Display cover download status
pub fn print_cover_status(cover_status: Status) {
    println!("  {} Cover downloaded", status_icon(cover_status));
}

/// Display track with full format
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

/// Display final success message
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

/// Display error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Display warning
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow(), message.yellow());
}

/// Display info message
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
