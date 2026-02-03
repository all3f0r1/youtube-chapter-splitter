//! TUI-style presenter for non-interactive mode
//!
//! This module provides TUI-style box-drawing output for non-interactive mode.

use crate::error::Result;

/// TUI-style presenter for non-interactive downloads
pub struct TuiStylePresenter;

impl Default for TuiStylePresenter {
    fn default() -> Self {
        Self
    }
}

impl TuiStylePresenter {
    pub fn new() -> Self {
        Self
    }

    pub fn print_header(&self, title: &str, duration: &str, track_count: usize) {
        let top = "┌".to_string() + &"─".repeat(60) + "┐";
        let bottom = "└".to_string() + &"─".repeat(60) + "┘";

        println!("{}", top);
        println!("│{}│", center_text(title, 60));
        println!(
            "│{}│",
            center_text(&format!("{} • {} tracks", duration, track_count), 60)
        );
        println!("{}", bottom);
    }
}

fn center_text(text: &str, width: usize) -> String {
    let padding = width.saturating_sub(text.len()).saturating_sub(2) / 2;
    format!("{}{}{}", " ".repeat(padding), text, " ".repeat(padding))
}

/// Entry point for TUI (when compiled with tui feature)
#[cfg(feature = "tui")]
pub fn run_tui() -> Result<()> {
    use colored::Colorize;

    // Placeholder until TUI is implemented
    eprintln!("{}", "Interactive TUI not yet implemented".yellow());
    eprintln!();
    eprintln!("Use: ytcs <URL> for non-interactive download");
    eprintln!("Or: ytcs --cli <URL> for plain-text mode");

    // TODO: Launch interactive TUI here
    std::process::exit(1);
}
