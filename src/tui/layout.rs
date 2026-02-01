//! Layout engine for TUI
//!
//! This module handles adaptive layouts for different terminal sizes
//! and detects terminal capabilities (Unicode, color, size).

use crossterm::terminal;
use ratatui::layout::Rect;
use std::env;

/// Terminal capabilities and constraints
pub struct LayoutEngine {
    pub min_width: u16,
    pub min_height: u16,
}

/// Detected terminal capabilities
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    /// Unicode support (for box-drawing characters)
    pub unicode: bool,
    /// Color support (for styled output)
    pub color: bool,
    /// Terminal is too small for TUI
    pub too_small: bool,
    /// Current terminal width
    pub width: u16,
    /// Current terminal height
    pub height: u16,
}

impl TerminalCapabilities {
    /// Detect terminal capabilities from environment
    pub fn detect() -> Self {
        let (width, height) = Self::detect_size();
        let unicode = Self::detect_unicode();
        let color = Self::detect_color();
        let too_small = width < 60 || height < 20;

        Self {
            unicode,
            color,
            too_small,
            width,
            height,
        }
    }

    /// Detect terminal size using crossterm
    fn detect_size() -> (u16, u16) {
        match terminal::size() {
            Ok((w, h)) => (w, h),
            Err(_) => (80, 24), // Fallback default
        }
    }

    /// Detect Unicode support via environment variables
    fn detect_unicode() -> bool {
        // Check LANG/LC_ALL for UTF-8
        let has_utf8 = env::var("LANG")
            .or_else(|_| env::var("LC_ALL"))
            .map(|val| val.contains("UTF-8") || val.contains("utf-8"))
            .unwrap_or(false);

        // Also check for explicit UTF-8 flag
        let has_utf8_flag = env::var("LC_CTYPE")
            .map(|val| val.contains("UTF-8") || val.contains("utf-8"))
            .unwrap_or(false);

        // Check for dumb terminal (no Unicode)
        let is_dumb = env::var("TERM")
            .map(|val| val == "dumb" || val == "DUMB")
            .unwrap_or(false);

        (has_utf8 || has_utf8_flag) && !is_dumb
    }

    /// Detect color support
    fn detect_color() -> bool {
        // Check NO_COLOR standard (https://no-color.org/)
        if env::var("NO_COLOR").is_ok() {
            return false;
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

        // Check COLORTERM variable (set by many modern terminals)
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        let has_colorterm =
            !colorterm.is_empty() && colorterm != "0" && colorterm != "false" && colorterm != "no";

        has_color || has_colorterm
    }

    /// Check if terminal supports monochrome mode (NO_COLOR or dumb terminal)
    pub fn is_monochrome(&self) -> bool {
        !self.color
    }

    /// Get the appropriate minimum size based on terminal type
    pub fn minimum_size() -> (u16, u16) {
        (60, 20) // Minimum for TUI to be usable
    }

    /// Check if current size meets minimum requirements
    pub fn meets_minimum(&self) -> bool {
        !self.too_small
    }

    /// Get adaptive layout constraints based on current size
    pub fn layout_constraints(&self) -> LayoutConstraints {
        let breakpoints = LayoutConstraints::new();
        breakpoints.select_for(self.width)
    }
}

/// Layout constraints for different terminal widths
#[derive(Debug, Clone, Copy)]
pub struct LayoutConstraints {
    /// Width threshold
    pub width_threshold: u16,
    /// Whether to use compact layout
    pub compact: bool,
    /// Maximum content width
    pub max_content_width: u16,
    /// Whether to show labels (vs icons only)
    pub show_labels: bool,
}

impl LayoutConstraints {
    /// Create default layout constraints
    pub fn new() -> Self {
        Self {
            width_threshold: 80,
            compact: false,
            max_content_width: 76,
            show_labels: true,
        }
    }

    /// Select constraints based on terminal width
    pub fn select_for(&self, width: u16) -> LayoutConstraints {
        match width {
            w if w < 60 => LayoutConstraints {
                width_threshold: 40,
                compact: true,
                max_content_width: w.saturating_sub(4),
                show_labels: false,
            },
            w if w < 80 => LayoutConstraints {
                width_threshold: 60,
                compact: false,
                max_content_width: w.saturating_sub(4),
                show_labels: true,
            },
            w if w < 100 => LayoutConstraints {
                width_threshold: 80,
                compact: false,
                max_content_width: 76,
                show_labels: true,
            },
            _ => LayoutConstraints {
                width_threshold: 100,
                compact: false,
                max_content_width: 90,
                show_labels: true,
            },
        }
    }
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            min_width: 60,
            min_height: 20,
        }
    }

    pub fn with_minimum(mut self, width: u16, height: u16) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }

    pub fn is_terminal_too_small(&self, width: u16, height: u16) -> bool {
        width < self.min_width || height < self.min_height
    }

    /// Get minimum size as a tuple
    pub fn minimum_size(&self) -> (u16, u16) {
        (self.min_width, self.min_height)
    }

    /// Calculate if a resize is needed (for future use)
    pub fn needs_resize(&self, current: Rect) -> bool {
        current.width < self.min_width || current.height < self.min_height
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_engine_new() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.min_width, 60);
        assert_eq!(engine.min_height, 20);
    }

    #[test]
    fn test_is_terminal_too_small() {
        let engine = LayoutEngine::new();
        assert!(engine.is_terminal_too_small(40, 15));
        assert!(engine.is_terminal_too_small(80, 15));
        assert!(engine.is_terminal_too_small(40, 24));
        assert!(!engine.is_terminal_too_small(80, 24));
    }

    #[test]
    fn test_minimum_size() {
        let engine = LayoutEngine::new();
        assert_eq!(engine.minimum_size(), (60, 20));
    }

    #[test]
    fn test_layout_constraints_small() {
        let constraints = LayoutConstraints::new();
        let small = constraints.select_for(50);
        assert!(small.compact);
        assert!(!small.show_labels);
    }

    #[test]
    fn test_layout_constraints_medium() {
        let constraints = LayoutConstraints::new();
        let medium = constraints.select_for(70);
        assert!(!medium.compact);
        assert!(medium.show_labels);
    }

    #[test]
    fn test_layout_constraints_large() {
        let constraints = LayoutConstraints::new();
        let large = constraints.select_for(100);
        assert!(!large.compact);
        assert!(large.show_labels);
        assert_eq!(large.max_content_width, 90);
    }
}
