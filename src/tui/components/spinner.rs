//! Spinner component for loading states
//!
//! This module provides cycling animation for loading indicators.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Spinner type/style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerStyle {
    /// Classic vertical spinner
    Vertical,
    /// Dots animation
    Dots,
    /// Arrow rotation
    Arrow,
    /// Simple box drawing
    Box,
}

/// Spinner component for loading states
pub struct Spinner {
    pub frames: Vec<String>,
    pub current: usize,
    pub style: SpinnerStyle,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self::with_style(SpinnerStyle::Vertical)
    }

    pub fn with_style(style: SpinnerStyle) -> Self {
        let frames = match style {
            SpinnerStyle::Vertical => vec!["⏳", "│", "╵", "│"],
            SpinnerStyle::Dots => vec!["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            SpinnerStyle::Arrow => vec!["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
            SpinnerStyle::Box => vec!["░", "▒", "▓", "▒", "▓", "▒", "▓", "█"],
        };

        Self {
            frames: frames.iter().map(|s| s.to_string()).collect(),
            current: 0,
            style,
        }
    }

    /// Get the current frame
    pub fn current(&self) -> &str {
        &self.frames[self.current]
    }

    /// Advance to next frame
    pub fn tick(&mut self) -> &str {
        self.current = (self.current + 1) % self.frames.len();
        self.current()
    }

    /// Reset to first frame
    pub fn reset(&mut self) {
        self.current = 0;
    }

    /// Draw the spinner at a position
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let text = self.tick();

        let paragraph = Paragraph::new(Line::from(text))
            .style(Style::default().fg(Color::Cyan))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    /// Draw with a message
    pub fn draw_with_message(&mut self, f: &mut Frame, area: Rect, message: &str) {
        self.tick();

        let text = format!("{} {}", self.current(), message);

        let paragraph = Paragraph::new(Line::from(text))
            .style(Style::default().fg(Color::Cyan))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    /// Draw in a centered box
    pub fn draw_centered(&mut self, f: &mut Frame, area: Rect, message: &str) {
        self.tick();

        let lines = vec![
            Line::from(""),
            Line::from(format!("{} {}", self.current(), message)),
            Line::from(""),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Center)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_new() {
        let spinner = Spinner::new();
        assert_eq!(spinner.current(), "⏳");
    }

    #[test]
    fn test_spinner_tick() {
        let mut spinner = Spinner::new();
        spinner.tick();
        assert_eq!(spinner.current(), "│");

        for _ in 0..10 {
            spinner.tick();
        }
        // Should cycle
        assert!(spinner.current() == "⏳" || spinner.current() == "│");
    }

    #[test]
    fn test_spinner_dots() {
        let spinner = Spinner::with_style(SpinnerStyle::Dots);
        assert_eq!(spinner.current(), "⣾");
        assert_eq!(spinner.frames.len(), 8);
    }
}
