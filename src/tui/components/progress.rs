//! Progress bar component for TUI
//!
//! This module provides minimal and detailed progress bar rendering.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Progress bar for TUI
pub struct ProgressBar {
    /// Current progress (0-100)
    pub progress: u16,
    /// Total value (for partial progress)
    pub total: u16,
    /// Message to display
    pub message: String,
    /// Show percentage
    pub show_percent: bool,
    /// Show detailed info
    pub detailed: bool,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            progress: 0,
            total: 100,
            message: String::new(),
            show_percent: true,
            detailed: false,
        }
    }

    /// Builder: set progress
    pub fn with_progress(mut self, progress: u16) -> Self {
        self.progress = progress.min(self.total);
        self
    }

    /// Builder: set total
    pub fn with_total(mut self, total: u16) -> Self {
        self.total = total;
        self
    }

    /// Builder: set message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Builder: set detailed mode
    pub fn with_detailed(mut self, detailed: bool) -> Self {
        self.detailed = detailed;
        self
    }

    /// Update progress
    pub fn set_progress(&mut self, progress: u16) {
        self.progress = progress.min(self.total);
    }

    /// Increment progress
    pub fn inc(&mut self, amount: u16) {
        self.progress = (self.progress + amount).min(self.total);
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.progress >= self.total
    }

    /// Get progress as percentage
    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        ((self.progress as f32 / self.total as f32) * 100.0) as u8
    }

    /// Draw the progress bar
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let percent = self.percent();
        let width = area.width.saturating_sub(2) as usize; // Account for borders
        let filled = (width * percent as usize) / 100;
        let empty = width.saturating_sub(filled);

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        let percent_text = if self.show_percent {
            format!(" {}%", percent)
        } else {
            String::new()
        };

        let text = if self.detailed {
            format!(
                "{}{}{}",
                bar,
                percent_text,
                if self.message.is_empty() {
                    String::new()
                } else {
                    format!(" - {}", self.message)
                }
            )
        } else {
            format!("{}{}", bar, percent_text)
        };

        let paragraph = Paragraph::new(Line::from(text))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    /// Draw a compact/minimal progress bar (no borders)
    pub fn draw_minimal(&self, f: &mut Frame, area: Rect) {
        let percent = self.percent();
        let width = area.width as usize;
        let filled = (width * percent as usize) / 100;
        let empty = width.saturating_sub(filled);

        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        let text = format!("{} {}%", bar, percent);

        let paragraph = Paragraph::new(Line::from(text))
            .style(Style::default().fg(Color::Green))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    /// Draw with message above (detailed view)
    pub fn draw_with_message(&self, f: &mut Frame, area: Rect) {
        if !self.message.is_empty() {
            let msg_area = Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: 3.min(area.height),
            };

            let msg_paragraph =
                Paragraph::new(self.message.as_str()).style(Style::default().fg(Color::Cyan));
            f.render_widget(msg_paragraph, msg_area);

            let progress_area = Rect {
                x: area.x,
                y: area.y + 3,
                width: area.width,
                height: area.height.saturating_sub(3),
            };
            self.draw(f, progress_area);
        } else {
            self.draw(f, area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_default() {
        let bar = ProgressBar::new();
        assert_eq!(bar.progress, 0);
        assert_eq!(bar.total, 100);
        assert_eq!(bar.percent(), 0);
    }

    #[test]
    fn test_progress_bar_percent() {
        let bar = ProgressBar::new().with_progress(50).with_total(100);
        assert_eq!(bar.percent(), 50);
    }

    #[test]
    fn test_progress_bar_inc() {
        let mut bar = ProgressBar::new().with_total(200);
        bar.inc(50);
        assert_eq!(bar.progress, 50);
        assert_eq!(bar.percent(), 25);

        bar.inc(150);
        assert_eq!(bar.progress, 200);
        assert!(bar.is_complete());
    }

    #[test]
    fn test_progress_bar_complete() {
        let bar = ProgressBar::new().with_progress(100);
        assert!(bar.is_complete());
    }
}
