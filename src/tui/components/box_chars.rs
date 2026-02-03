//! Box-drawing characters for TUI borders and separators
//!
//! This module provides Unicode and ASCII box-drawing characters
//! that automatically adapt to terminal capabilities.

/// Box-drawing characters for TUI rendering
pub struct BoxChars {
    /// Use Unicode box-drawing characters
    use_unicode: bool,
}

impl BoxChars {
    /// Create new BoxChars based on Unicode support
    pub fn new(use_unicode: bool) -> Self {
        Self { use_unicode }
    }

    /// Create from terminal capabilities
    pub fn from_capabilities(has_unicode: bool) -> Self {
        Self::new(has_unicode)
    }

    /// Get top-left corner
    pub fn top_left(&self) -> char {
        if self.use_unicode { '┌' } else { '+' }
    }

    /// Get top-right corner
    pub fn top_right(&self) -> char {
        if self.use_unicode { '┐' } else { '+' }
    }

    /// Get bottom-left corner
    pub fn bottom_left(&self) -> char {
        if self.use_unicode { '└' } else { '+' }
    }

    /// Get bottom-right corner
    pub fn bottom_right(&self) -> char {
        if self.use_unicode { '┘' } else { '+' }
    }

    /// Get horizontal line
    pub fn horizontal(&self) -> char {
        if self.use_unicode { '─' } else { '-' }
    }

    /// Get vertical line
    pub fn vertical(&self) -> char {
        if self.use_unicode { '│' } else { '|' }
    }

    /// Get top T-junction
    pub fn top_tee(&self) -> char {
        if self.use_unicode { '┬' } else { '+' }
    }

    /// Get bottom T-junction
    pub fn bottom_tee(&self) -> char {
        if self.use_unicode { '┴' } else { '+' }
    }

    /// Get left T-junction
    pub fn left_tee(&self) -> char {
        if self.use_unicode { '├' } else { '+' }
    }

    /// Get right T-junction
    pub fn right_tee(&self) -> char {
        if self.use_unicode { '┤' } else { '+' }
    }

    /// Get cross junction
    pub fn cross(&self) -> char {
        if self.use_unicode { '┼' } else { '+' }
    }

    /// Get double-line top-left corner
    pub fn double_top_left(&self) -> char {
        if self.use_unicode { '╔' } else { '+' }
    }

    /// Get double-line top-right corner
    pub fn double_top_right(&self) -> char {
        if self.use_unicode { '╗' } else { '+' }
    }

    /// Get double-line bottom-left corner
    pub fn double_bottom_left(&self) -> char {
        if self.use_unicode { '╚' } else { '+' }
    }

    /// Get double-line bottom-right corner
    pub fn double_bottom_right(&self) -> char {
        if self.use_unicode { '╝' } else { '+' }
    }

    /// Get double-line horizontal
    pub fn double_horizontal(&self) -> char {
        if self.use_unicode { '═' } else { '=' }
    }

    /// Get double-line vertical
    pub fn double_vertical(&self) -> char {
        if self.use_unicode { '║' } else { '|' }
    }

    /// Get a horizontal line of specified width
    pub fn h_line(&self, width: usize) -> String {
        std::iter::repeat_n(self.horizontal(), width).collect()
    }

    /// Get a vertical line of specified height
    pub fn v_line(&self, height: usize) -> String {
        std::iter::repeat_n(self.vertical(), height).collect()
    }

    /// Create a box frame with title
    pub fn frame_with_title(&self, width: usize, title: Option<&str>) -> String {
        let tl = self.top_left();
        let tr = self.top_right();
        let bl = self.bottom_left();
        let br = self.bottom_right();
        let h_line = self.h_line(width.saturating_sub(2));
        let v = self.vertical();

        let inner_width = width.saturating_sub(2);

        let top = if let Some(t) = title {
            let title_len = t.len().min(inner_width);
            let padding = inner_width.saturating_sub(title_len);
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!(
                "{}{}{}{}{}{}",
                tl,
                self.h_line(left_pad),
                t,
                self.h_line(right_pad),
                tr,
                v
            )
        } else {
            format!("{}{}{}", tl, h_line, tr)
        };

        let bottom = format!("{}{}{}", bl, h_line, br);

        format!("{}\n{}\n{}", top, v, bottom)
    }

    /// Create a section separator
    pub fn separator(&self, width: usize) -> String {
        let cross = self.cross();
        let half_width = width.saturating_sub(2) / 2;
        format!(
            "{}{}{}{}",
            cross,
            self.h_line(half_width),
            cross,
            self.h_line(width.saturating_sub(2) - half_width)
        )
    }
}

impl Default for BoxChars {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Pre-defined border styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderStyle {
    /// Single line (light)
    Single,
    /// Double line
    Double,
    /// Rounded corners
    Rounded,
    /// Plain ASCII
    Plain,
}

impl BorderStyle {
    /// Get the appropriate ratatui BorderType
    pub fn to_border_type(self) -> ratatui::widgets::BorderType {
        match self {
            BorderStyle::Single => ratatui::widgets::BorderType::Plain,
            BorderStyle::Double => ratatui::widgets::BorderType::Double,
            BorderStyle::Rounded => ratatui::widgets::BorderType::Rounded,
            BorderStyle::Plain => ratatui::widgets::BorderType::Plain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_chars_unicode() {
        let chars = BoxChars::new(true);
        assert_eq!(chars.top_left(), '┌');
        assert_eq!(chars.top_right(), '┐');
        assert_eq!(chars.bottom_left(), '└');
        assert_eq!(chars.bottom_right(), '┘');
        assert_eq!(chars.horizontal(), '─');
        assert_eq!(chars.vertical(), '│');
    }

    #[test]
    fn test_box_chars_ascii() {
        let chars = BoxChars::new(false);
        assert_eq!(chars.top_left(), '+');
        assert_eq!(chars.top_right(), '+');
        assert_eq!(chars.bottom_left(), '+');
        assert_eq!(chars.bottom_right(), '+');
        assert_eq!(chars.horizontal(), '-');
        assert_eq!(chars.vertical(), '|');
    }

    #[test]
    fn test_box_chars_double() {
        let chars = BoxChars::new(true);
        assert_eq!(chars.double_top_left(), '╔');
        assert_eq!(chars.double_top_right(), '╗');
        assert_eq!(chars.double_bottom_left(), '╚');
        assert_eq!(chars.double_bottom_right(), '╝');
        assert_eq!(chars.double_horizontal(), '═');
        assert_eq!(chars.double_vertical(), '║');
    }

    #[test]
    fn test_box_chars_h_line() {
        let chars = BoxChars::new(true);
        assert_eq!(chars.h_line(5), "─────");
    }

    #[test]
    fn test_border_style_to_border_type() {
        assert_eq!(
            BorderStyle::Single.to_border_type(),
            ratatui::widgets::BorderType::Plain
        );
        assert_eq!(
            BorderStyle::Double.to_border_type(),
            ratatui::widgets::BorderType::Double
        );
        assert_eq!(
            BorderStyle::Rounded.to_border_type(),
            ratatui::widgets::BorderType::Rounded
        );
    }
}
