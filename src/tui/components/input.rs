//! Text input component for TUI
//!
//! Provides a single-line text input field with cursor management.

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Single-line text input component
pub struct TextInput {
    /// Current input value
    pub value: String,
    /// Cursor position (in bytes, not characters)
    pub cursor: usize,
    /// Placeholder text shown when value is empty
    pub placeholder: String,
    /// Title/label for the input
    pub title: String,
    /// Whether the input is focused
    pub focused: bool,
    /// Maximum length (0 = unlimited)
    pub max_length: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    /// Create a new empty text input
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            placeholder: String::new(),
            title: String::new(),
            focused: false,
            max_length: 0,
        }
    }

    /// Builder: set the initial value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.cursor = value.len();
        self.value = value;
        self
    }

    /// Builder: set the placeholder text
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Builder: set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: set focused state
    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Builder: set max length
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }

    /// Clear the input value
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Check if the input is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Draw the text input component
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let input_text = if self.value.is_empty() {
            Span::styled(
                &self.placeholder,
                Style::default().fg(Color::Rgb(128, 128, 128)),
            )
        } else {
            Span::raw(&self.value)
        };

        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Rgb(100, 100, 100))
        };

        let title = if self.title.is_empty() {
            "".to_string()
        } else {
            self.title.clone()
        };

        let paragraph = Paragraph::new(Line::from(input_text))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(title),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);

        // Show cursor if focused
        if self.focused {
            self.draw_cursor(f, area);
        }
    }

    /// Draw the cursor at the correct position
    fn draw_cursor(&self, f: &mut Frame, area: Rect) {
        // Calculate cursor position
        let text = if self.value.is_empty() {
            &self.placeholder
        } else {
            &self.value
        };

        // Simple cursor positioning - assumes single line
        let cursor_x = area.x + 1 + self.cursor_position_in_display(text) as u16;
        let cursor_y = area.y + 1;

        // Ensure cursor is within bounds
        if cursor_x < area.x + area.width.saturating_sub(1) {
            // Set cursor position (hidden in ratatui, but terminal will show it)
            f.set_cursor_position(ratatui::layout::Position::new(cursor_x, cursor_y));
        }
    }

    /// Calculate the visual cursor position (handles multibyte characters)
    fn cursor_position_in_display(&self, text: &str) -> usize {
        let mut byte_pos = 0;
        let mut char_pos = 0;

        for ch in text.chars() {
            if byte_pos >= self.cursor {
                break;
            }
            byte_pos += ch.len_utf8();
            char_pos += 1;
        }

        char_pos
    }

    /// Handle a key event for the input
    ///
    /// Returns true if the event was handled
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                // Check max length
                if self.max_length > 0 && self.value.len() >= self.max_length {
                    return true;
                }

                self.value.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                true
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    // Find the previous character boundary
                    let before_cursor = &self.value[..self.cursor];
                    let prev_char_len = before_cursor
                        .chars()
                        .last()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);

                    let new_cursor = self.cursor - prev_char_len;
                    self.value.remove(new_cursor);
                    self.cursor = new_cursor;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    let before_cursor = &self.value[..self.cursor];
                    let prev_char_len = before_cursor
                        .chars()
                        .last()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    self.cursor -= prev_char_len;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor < self.value.len() {
                    if let Some(ch) = self.value[self.cursor..].chars().next() {
                        self.cursor += ch.len_utf8();
                    }
                }
                true
            }
            KeyCode::Home => {
                self.cursor = 0;
                true
            }
            KeyCode::End => {
                self.cursor = self.value.len();
                true
            }
            _ => false,
        }
    }
}

/// Multiline text input component (for notes, descriptions, etc.)
pub struct TextArea {
    /// Current lines of text
    pub lines: Vec<String>,
    /// Cursor line
    pub cursor_line: usize,
    /// Cursor column (in bytes)
    pub cursor_col: usize,
    /// Placeholder text
    pub placeholder: String,
    /// Whether focused
    pub focused: bool,
    /// Scroll offset
    pub scroll: usize,
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            placeholder: String::new(),
            focused: false,
            scroll: 0,
        }
    }

    /// Get the full text content
    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// Set text content
    pub fn set_text(&mut self, text: &str) {
        self.lines = text.lines().map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor_line = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines.last().map(|l| l.len()).unwrap_or(0);
    }

    /// Clear the text area
    pub fn clear(&mut self) {
        self.lines.clear();
        self.lines.push(String::new());
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll = 0;
    }

    /// Draw the text area
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let display_text = if self.lines.iter().all(|l| l.is_empty()) {
            vec![Line::from(Span::styled(
                &self.placeholder,
                Style::default().fg(Color::Rgb(128, 128, 128)),
            ))]
        } else {
            self.lines.iter().map(|l| Line::raw(l.as_str())).collect()
        };

        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::Rgb(100, 100, 100))
        };

        let paragraph = Paragraph::new(display_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    /// Handle key events
    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                if c == '\n' {
                    self.lines.insert(self.cursor_line + 1, String::new());
                    let rest = self.lines[self.cursor_line].split_off(self.cursor_col);
                    self.lines[self.cursor_line + 1] = rest;
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                } else {
                    self.lines[self.cursor_line].insert(self.cursor_col, c);
                    self.cursor_col += c.len_utf8();
                }
                true
            }
            KeyCode::Backspace => {
                if self.cursor_col > 0 {
                    let prev_char_len = self.lines[self.cursor_line]
                        .chars()
                        .rev()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    self.cursor_col -= prev_char_len;
                    self.lines[self.cursor_line].remove(self.cursor_col);
                } else if self.cursor_line > 0 {
                    // Merge with previous line
                    let prev_len = self.lines[self.cursor_line - 1].len();
                    let current_line = self.lines[self.cursor_line].clone();
                    self.lines[self.cursor_line - 1].push_str(&current_line);
                    self.lines.remove(self.cursor_line);
                    self.cursor_line -= 1;
                    self.cursor_col = prev_len;
                }
                true
            }
            KeyCode::Enter => {
                self.lines.insert(self.cursor_line + 1, String::new());
                let rest = self.lines[self.cursor_line].split_off(self.cursor_col);
                self.lines[self.cursor_line + 1] = rest;
                self.cursor_line += 1;
                self.cursor_col = 0;
                true
            }
            KeyCode::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                }
                true
            }
            KeyCode::Down => {
                if self.cursor_line < self.lines.len().saturating_sub(1) {
                    self.cursor_line += 1;
                    self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_col < self.lines[self.cursor_line].len() {
                    self.cursor_col += 1;
                }
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_default() {
        let input = TextInput::new();
        assert!(input.is_empty());
        assert_eq!(input.cursor, 0);
    }

    #[test]
    fn test_text_input_builder() {
        let input = TextInput::new()
            .with_value("test")
            .with_placeholder("Enter text")
            .with_focused(true);

        assert_eq!(input.value, "test");
        assert_eq!(input.cursor, 4);
        assert!(input.focused);
    }

    #[test]
    fn test_text_input_clear() {
        let mut input = TextInput::new().with_value("test");
        input.clear();
        assert!(input.is_empty());
        assert_eq!(input.cursor, 0);
    }
}
