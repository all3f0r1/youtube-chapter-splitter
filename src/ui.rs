//! Minimalist user interface module
//!
//! Design: Pragmatic • Direct • Classy
//! - Consistent tree structure (▶ ├─ └─)
//! - Aligned labels (Duration, Tracks, Artist, Album)
//! - Minimal blank lines, compact layout
//! - Progressive feedback during operations
//! - Clean typography with bold for sections only

use colored::*;
use std::io::{self, Write};

/// Source of metadata (artist/album)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataSource {
    /// Automatically detected from video title
    Detected,
    /// Forced by user via --artist or --album flag
    Forced,
    /// Default value (Unknown Artist)
    Default,
}

impl MetadataSource {
    /// Returns the display label for this source
    pub fn label(self) -> &'static str {
        match self {
            MetadataSource::Detected => "detected",
            MetadataSource::Forced => "user-forced",
            MetadataSource::Default => "default",
        }
    }
}

// ============================================================================
// Plain values - no colors, just styling
// ============================================================================

/// Print a blank line for explicit section separation
pub fn print_blank_line() {
    println!();
}

/// Display minimal header (version only)
pub fn print_header() {
    println!(
        "{}",
        format!("ytcs v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
    println!();
}

/// Display section header with tree arrow (bold only, no color)
pub fn print_section_header(title: &str) {
    println!("{} {}", "▶".bold(), title.bold());
}

/// Display a tree item line with aligned label (non-last)
pub fn print_tree_item(label: &str, value: &str) {
    println!("  ├─ {:<10} {}", label.bright_black().bold(), value);
}

/// Display the last tree item line with aligned label
pub fn print_tree_item_last(label: &str, value: &str) {
    println!("  └─ {:<10} {}", label.bright_black().bold(), value);
}

/// Display a tree item with extra info (non-last)
pub fn print_tree_item_with_extra(label: &str, value: &str, extra: &str) {
    println!(
        "  ├─ {:<10} {} ({})",
        label.bright_black().bold(),
        value,
        extra.dimmed()
    );
}

/// Display the last tree item with extra info
pub fn print_tree_item_last_with_extra(label: &str, value: &str, extra: &str) {
    println!(
        "  └─ {:<10} {} ({})",
        label.bright_black().bold(),
        value,
        extra.dimmed()
    );
}

/// Display video metadata in unified tree style
pub fn print_video_metadata_tree(
    _title: &str,
    duration: &str,
    tracks: usize,
    artist: &str,
    album: &str,
    artist_source: MetadataSource,
    album_source: MetadataSource,
) {
    // Main title line (using artist - album as display)
    let display_title = format!("{} - {}", artist, album);
    println!("{} {}", "▶".bold(), display_title.bold());

    // Tree structure with plain values (no blank line before)
    print_tree_item("Duration", duration);
    print_tree_item("Tracks", &tracks.to_string());
    print_tree_item_with_extra("Artist", artist, artist_source.label());
    print_tree_item_last_with_extra("Album", album, album_source.label());
    println!();
}

/// Prompt for both artist and album in a compact tree style
pub fn prompt_metadata(
    video_title: &str,
    default_artist: &str,
    default_album: &str,
) -> (String, String) {
    let _clean_title = video_title; // Used for display, stored to avoid warning

    println!("{} {}", "⚠".bold(), "Metadata required".bold());
    println!("  └─ Video {:<28}", format!("\"{}\"", video_title).dimmed());
    println!();
    println!("  ├─ Artist (required, or press Enter for \"unknown\"):",);
    print!("  └─> ");
    io::stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    let artist = if input.trim().is_empty() {
        default_artist.to_string()
    } else {
        input.trim().to_string()
    };

    println!("  ├─ Album (required, or press Enter for \"unknown\"):",);
    print!("  └─> ");
    io::stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    let album = if input.trim().is_empty() {
        default_album.to_string()
    } else {
        input.trim().to_string()
    };

    println!();

    (artist, album)
}

/// Print a single track during splitting (progressive output)
/// Aligns durations vertically by padding the track number
pub fn print_track_progress(index: usize, total: usize, title: &str, duration: &str) {
    let is_last = index == total;
    let prefix = if is_last { "  └─" } else { "  ├─" };

    // Calculate width for track number alignment
    let total_width = total.to_string().len();
    let index_str = format!("{:>width$}", index, width = total_width);

    println!("{} {} {} [{}]", prefix.dimmed(), index_str, title, duration);
    io::stdout().flush().ok();
}

/// Display artwork download section in tree style
pub fn print_artwork_section(filename: &str) {
    println!();
    print_section_header("Downloading artwork");
    if !filename.is_empty() {
        // Remove extra spaces: "Saved /path" instead of "Saved     /path"
        println!("  └─ Saved {}", filename);
    } else {
        print_tree_item_last("Status", "not available");
    }
}

/// Display audio download section header
pub fn print_audio_section_header() {
    println!();
    print_section_header("Downloading audio");
}

/// Display audio download complete (no blank line before)
pub fn print_audio_complete(filename: &str) {
    // No blank line before, just print the saved path
    println!("  └─ Saved {}", filename);
}

/// Display splitting section header
pub fn print_splitting_section_header(tracks_count: usize) {
    println!();
    print_section_header(&format!("Splitting into {} tracks", tracks_count));
}

/// Display splitting completion message
pub fn print_splitting_complete() {
    println!("{} Splitting achieved", "✓".bold());
}

/// Display final success with output directory
pub fn print_final_result(output_dir: &std::path::Path) {
    println!();
    println!("{} {}", "✓".bold(), output_dir.display());
    println!();
}

/// Display error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Display warning
pub fn print_warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow().bold(), message.yellow());
}

/// Display info message
pub fn print_info(message: &str) {
    println!("  {}", message.dimmed());
}

/// Clean title to match folder name
pub fn clean_title(title: &str) -> String {
    use crate::utils;
    utils::clean_folder_name(title)
}

// ============================================================================
// Legacy compatibility - kept for other modules that still use these
// ============================================================================

/// Output mode for UI (legacy)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    Colored,
    Plain,
}

/// Operation status (legacy)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Pending,
    InProgress,
    Success,
    Failed,
}

/// Track progress (legacy)
#[derive(Debug, Clone)]
pub struct TrackProgress {
    pub number: usize,
    pub title: String,
    pub status: Status,
    pub duration: String,
}

/// Plain text presenter for --cli mode (legacy)
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

    pub fn error(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => eprintln!("ERROR: {}", message),
            OutputMode::Colored => eprintln!("{} {}", "✗".red().bold(), message.red()),
        }
    }

    pub fn warning(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => eprintln!("WARNING: {}", message),
            OutputMode::Colored => eprintln!("{} {}", "⚠".yellow(), message.yellow()),
        }
    }

    pub fn info(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => println!("INFO: {}", message),
            OutputMode::Colored => println!("  {}", message.dimmed()),
        }
    }

    pub fn success(&self, message: &str) {
        match self.output_mode {
            OutputMode::Plain => println!("OK: {}", message),
            OutputMode::Colored => println!("{} {}", "✓".green().bold(), message.green()),
        }
    }

    pub fn header(&self) {
        match self.output_mode {
            OutputMode::Plain => println!("ytcs v{}", env!("CARGO_PKG_VERSION")),
            OutputMode::Colored => println!(
                "{}",
                format!("ytcs v{}", env!("CARGO_PKG_VERSION")).dimmed()
            ),
        }
    }

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

    pub fn video_info(&self, title: &str, duration: &str, tracks: usize) {
        match self.output_mode {
            OutputMode::Plain => {
                println!("Title: {}", clean_title(title));
                println!("Duration: {}", duration);
                println!("Tracks: {}", tracks);
            }
            OutputMode::Colored => {
                // Use the new tree style
                print_video_metadata_tree(
                    title,
                    duration,
                    tracks,
                    "Unknown",
                    "Unknown",
                    MetadataSource::Default,
                    MetadataSource::Default,
                );
            }
        }
    }

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

fn status_icon(status: Status) -> ColoredString {
    match status {
        Status::Pending => " ".dimmed(),
        Status::InProgress => "⏳".yellow(),
        Status::Success => "✓".green(),
        Status::Failed => "✗".red(),
    }
}

pub fn print_video_info(
    title: &str,
    duration: &str,
    tracks: usize,
    from_description: bool,
    silence_detection: bool,
) {
    let clean_title = clean_title(title);

    println!(
        "{} {}",
        "▶".bright_cyan().bold(),
        clean_title.bright_white().bold()
    );

    let tracks_display = if silence_detection {
        "? tracks (silence detection)".to_string()
    } else if tracks > 0 {
        format!("{} tracks", tracks)
    } else if from_description {
        "checking description...".to_string()
    } else {
        "? tracks".to_string()
    };

    println!(
        "  {} {}",
        duration.dimmed(),
        format!("• {}", tracks_display).dimmed()
    );
    println!();
}

pub fn print_cover_status(cover_status: Status) {
    println!("  {} Cover downloaded", status_icon(cover_status));
}

pub fn print_track(track: &TrackProgress, artist: &str, album: &str, filename_format: &str) {
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

pub fn print_success(output_dir: &str) {
    println!();
    println!("{} {}", "✓".green().bold(), output_dir.bright_blue());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
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

    #[test]
    fn test_metadata_source_labels() {
        assert_eq!(MetadataSource::Detected.label(), "detected");
        assert_eq!(MetadataSource::Forced.label(), "user-forced");
        assert_eq!(MetadataSource::Default.label(), "default");
    }

    #[test]
    fn test_track_progress_alignment() {
        // Test that track progress outputs correctly formatted lines
        // This is mainly to ensure the formatting string is valid
        let _ = std::io::stdout().flush();
        // The function should not panic
        print_track_progress(1, 12, "Test Track", "3:45");
        print_track_progress(12, 12, "Last Track", "5:00");
    }
}
