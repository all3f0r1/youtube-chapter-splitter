//! Color mode and accessibility utilities for TUI
//!
//! This module provides color mode detection and accessible styling.

use std::env;
use ratatui::style::{Color, Modifier, Style};

/// Color mode for TUI rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    /// Full color support
    Color,
    /// Monochrome mode (NO_COLOR or terminal doesn't support color)
    Monochrome,
}

impl ColorMode {
    /// Detect color mode from environment
    pub fn detect() -> Self {
        // Check NO_COLOR standard (https://no-color.org/)
        if env::var("NO_COLOR").is_ok() {
            return ColorMode::Monochrome;
        }

        // Check TERM for color support
        let term = env::var("TERM").unwrap_or_default();
        let has_color = !term.contains("dumb")
            && !term.contains("DUMB")
            && (term.contains("color")
                || term.contains("ansi")
                || term.contains("xterm")
                || term.contains("screen")
                || term.contains("tmux")
                || term.contains("vt100")
                || term.contains("vt220"));

        // Check COLORTERM variable
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        let has_colorterm = !colorterm.is_empty()
            && colorterm != "0"
            && colorterm != "false"
            && colorterm != "no";

        if has_color || has_colorterm {
            ColorMode::Color
        } else {
            ColorMode::Monochrome
        }
    }

    /// Check if running in monochrome mode
    pub fn is_monochrome(&self) -> bool {
        matches!(self, ColorMode::Monochrome)
    }
}

impl Default for ColorMode {
    fn default() -> Self {
        Self::detect()
    }
}

/// Accessible styling utilities
pub struct AccessibleStyle {
    color_mode: ColorMode,
}

impl AccessibleStyle {
    pub fn new() -> Self {
        Self {
            color_mode: ColorMode::detect(),
        }
    }

    pub fn with_color_mode(mut self, mode: ColorMode) -> Self {
        self.color_mode = mode;
        self
    }

    /// Get primary color (or gray for monochrome)
    pub fn primary_color(&self) -> Color {
        match self.color_mode {
            ColorMode::Color => Color::Cyan,
            ColorMode::Monochrome => Color::Gray,
        }
    }

    /// Get success color
    pub fn success_color(&self) -> Color {
        match self.color_mode {
            ColorMode::Color => Color::Green,
            ColorMode::Monochrome => Color::White,
        }
    }

    /// Get error color
    pub fn error_color(&self) -> Color {
        match self.color_mode {
            ColorMode::Color => Color::Red,
            ColorMode::Monochrome => Color::White,
        }
    }

    /// Get warning color
    pub fn warning_color(&self) -> Color {
        match self.color_mode {
            ColorMode::Color => Color::Yellow,
            ColorMode::Monochrome => Color::White,
        }
    }

    /// Get muted/secondary color
    pub fn muted_color(&self) -> Color {
        match self.color_mode {
            ColorMode::Color => Color::Rgb(100, 100, 100),
            ColorMode::Monochrome => Color::DarkGray,
        }
    }

    /// Create a primary style with dual coding (color + modifier)
    pub fn primary_style(&self) -> Style {
        let mut style = Style::default().fg(self.primary_color());
        if self.color_mode.is_monochrome() {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Create a success style with dual coding
    pub fn success_style(&self) -> Style {
        let mut style = Style::default().fg(self.success_color());
        if self.color_mode.is_monochrome() {
            style = style.add_modifier(Modifier::BOLD);
        } else {
            style = style.add_modifier(Modifier::ITALIC);
        }
        style
    }

    /// Create an error style with dual coding
    pub fn error_style(&self) -> Style {
        let mut style = Style::default().fg(self.error_color());
        if self.color_mode.is_monochrome() {
            style = style.add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED);
        } else {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Create a warning style with dual coding
    pub fn warning_style(&self) -> Style {
        let mut style = Style::default().fg(self.warning_color());
        if self.color_mode.is_monochrome() {
            style = style.add_modifier(Modifier::BOLD);
        }
        style
    }

    /// Create a focused style (for selection)
    pub fn focus_style(&self) -> Style {
        let bg = if self.color_mode.is_monochrome() {
            Color::DarkGray
        } else {
            Color::Rgb(50, 50, 100)
        };
        Style::default().bg(bg).add_modifier(Modifier::BOLD)
    }
}

impl Default for AccessibleStyle {
    fn default() -> Self {
        Self::new()
    }
}

/// Accessibility symbols for dual coding
pub struct Symbols;

impl Symbols {
    /// Check mark for success
    pub fn check() -> &'static str {
        "✓"
    }

    /// Cross mark for error
    pub fn cross() -> &'static str {
        "✗"
    }

    /// Warning triangle
    pub fn warning() -> &'static str {
        "⚠"
    }

    /// Info icon
    pub fn info() -> &'static str {
        "ℹ"
    }

    /// Right arrow/pointer for selection
    pub fn pointer() -> &'static str {
        "►"
    }

    /// Double pointer for active item
    pub fn pointer_double() -> &'static str {
        "▶"
    }

    /// Horizontal ellipsis for "more"
    pub fn ellipsis() -> &'static str {
        "…"
    }

    /// Fallback ASCII symbols for terminals without Unicode
    pub fn check_ascii() -> &'static str {
        "[x]"
    }

    pub fn cross_ascii() -> &'static str {
        "[ ]"
    }

    pub fn pointer_ascii() -> &'static str {
        ">"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_detect() {
        let mode = ColorMode::detect();
        // Just verify it doesn't panic
        match mode {
            ColorMode::Color => {}
            ColorMode::Monochrome => {}
        }
    }

    #[test]
    fn test_accessible_style_new() {
        let style = AccessibleStyle::new();
        // Verify primary color is valid
        let _ = style.primary_color();
    }

    #[test]
    fn test_symbols() {
        assert_eq!(Symbols::check(), "✓");
        assert_eq!(Symbols::cross(), "✗");
        assert_eq!(Symbols::warning(), "⚠");
        assert_eq!(Symbols::pointer(), "►");
    }
}
