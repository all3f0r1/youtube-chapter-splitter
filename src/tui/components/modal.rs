//! Modal overlay component for TUI
//!
//! Provides dialog boxes for confirmations, alerts, and prompts.

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Modal dialog type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    Info,
    Warning,
    Error,
    Confirm,
    Input,
}

/// Modal state for user interaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalState {
    /// Waiting for user response
    Waiting,
    /// User confirmed (Yes/OK)
    Confirmed,
    /// User cancelled (No/Cancel)
    Cancelled,
    /// User entered text
    Input(String),
}

/// Modal overlay component
pub struct Modal {
    pub title: String,
    pub message: String,
    pub modal_type: ModalType,
    pub state: ModalState,
    pub buttons: Vec<String>,
    pub selected_button: usize,
}

impl Modal {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            modal_type: ModalType::Info,
            state: ModalState::Waiting,
            buttons: vec!["OK".to_string()],
            selected_button: 0,
        }
    }

    /// Set the modal type
    pub fn with_type(mut self, modal_type: ModalType) -> Self {
        self.modal_type = modal_type;
        self
    }

    /// Set buttons (e.g., ["Yes", "No"] or ["OK", "Cancel"])
    pub fn with_buttons(mut self, buttons: Vec<&str>) -> Self {
        self.buttons = buttons.iter().map(|s| s.to_string()).collect();
        if !self.buttons.is_empty() {
            self.selected_button = 0;
        }
        self
    }

    /// Create an info modal
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(ModalType::Info)
    }

    /// Create a warning modal
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(ModalType::Warning)
    }

    /// Create an error modal
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message).with_type(ModalType::Error)
    }

    /// Create a confirmation modal
    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(title, message)
            .with_type(ModalType::Confirm)
            .with_buttons(vec!["Yes", "No"])
    }

    /// Get the modal state
    pub fn state(&self) -> &ModalState {
        &self.state
    }

    /// Check if waiting for user response
    pub fn is_waiting(&self) -> bool {
        self.state == ModalState::Waiting
    }

    /// Handle key event for modal
    ///
    /// Returns true if the modal was closed
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        match key.code {
            crossterm::event::KeyCode::Enter => {
                if self.selected_button < self.buttons.len() {
                    if self.selected_button == 0 {
                        // First button (Yes/OK)
                        match self.modal_type {
                            ModalType::Confirm => {
                                self.state = ModalState::Confirmed;
                            }
                            _ => {
                                self.state = ModalState::Cancelled;
                            }
                        }
                    } else {
                        // Other buttons (No/Cancel)
                        self.state = ModalState::Cancelled;
                    }
                }
                true
            }
            crossterm::event::KeyCode::Esc => {
                self.state = ModalState::Cancelled;
                true
            }
            crossterm::event::KeyCode::Left => {
                if !self.buttons.is_empty() && self.selected_button > 0 {
                    self.selected_button -= 1;
                }
                false
            }
            crossterm::event::KeyCode::Right => {
                if self.selected_button + 1 < self.buttons.len() {
                    self.selected_button += 1;
                }
                false
            }
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Char('Q') => {
                self.state = ModalState::Cancelled;
                true
            }
            crossterm::event::KeyCode::Char('y') | crossterm::event::KeyCode::Char('Y') => {
                if self.modal_type == ModalType::Confirm {
                    self.state = ModalState::Confirmed;
                    true
                } else {
                    false
                }
            }
            crossterm::event::KeyCode::Char('n') | crossterm::event::KeyCode::Char('N') => {
                if self.modal_type == ModalType::Confirm {
                    self.state = ModalState::Cancelled;
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Draw the modal overlay
    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.area();

        // Calculate modal size (centered, with max width)
        let max_width = 60;
        let max_height = 20;

        let width = (size.width.min(max_width)).saturating_sub(4);
        let height = (size.height.min(max_height)).saturating_sub(4);

        let x = (size.width - width) / 2;
        let y = (size.height - height) / 2;

        let area = Rect {
            x,
            y,
            width,
            height,
        };

        // Clear area under modal
        f.render_widget(Clear, area);

        // Get border color based on type
        let border_color = match self.modal_type {
            ModalType::Info => Color::Rgb(100, 150, 200),
            ModalType::Warning => Color::Rgb(200, 150, 50),
            ModalType::Error => Color::Rgb(200, 50, 50),
            ModalType::Confirm => Color::Rgb(100, 200, 100),
            ModalType::Input => Color::Rgb(100, 150, 200),
        };

        // Split into title, content, and buttons
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Message
                Constraint::Length(3), // Buttons
            ])
            .split(area);

        // Title
        let title_style = Style::default()
            .fg(border_color)
            .add_modifier(Modifier::BOLD);
        let title = Paragraph::new(self.title.as_str())
            .alignment(Alignment::Center)
            .style(title_style);

        f.render_widget(
            title,
            Rect {
                x: chunks[0].x + 1,
                y: chunks[0].y + 1,
                width: chunks[0].width.saturating_sub(2),
                height: chunks[0].height.saturating_sub(2),
            },
        );

        // Message (with word wrapping)
        let message_paragraph = Paragraph::new(self.message.as_str())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });

        f.render_widget(
            message_paragraph,
            Rect {
                x: chunks[1].x + 1,
                y: chunks[1].y + 1,
                width: chunks[1].width.saturating_sub(2),
                height: chunks[1].height.saturating_sub(2),
            },
        );

        // Buttons
        if !self.buttons.is_empty() {
            self.draw_buttons(f, chunks[2], border_color);
        }
    }

    fn draw_buttons(&self, f: &mut Frame, area: Rect, border_color: Color) {
        let button_width = 10;
        let gap = 2;
        let total_width = self.buttons.len() as u16 * (button_width + gap) - gap;

        let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;

        for (i, button) in self.buttons.iter().enumerate() {
            let button_area = Rect {
                x: start_x + (i as u16 * (button_width + gap)),
                y: area.y + 1,
                width: button_width,
                height: area.height.saturating_sub(2),
            };

            let is_selected = i == self.selected_button;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(border_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White).bg(border_color)
            };

            let paragraph = Paragraph::new(button.as_str())
                .alignment(Alignment::Center)
                .style(style);

            f.render_widget(paragraph, button_area);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_new() {
        let modal = Modal::new("Title", "Message");
        assert_eq!(modal.title, "Title");
        assert_eq!(modal.message, "Message");
        assert_eq!(modal.buttons, vec!["OK"]);
    }

    #[test]
    fn test_modal_confirm() {
        let modal = Modal::confirm("Confirm", "Are you sure?");
        assert_eq!(modal.modal_type, ModalType::Confirm);
        assert_eq!(modal.buttons, vec!["Yes", "No"]);
        assert!(modal.is_waiting());
    }

    #[test]
    fn test_modal_confirm_yes() {
        let mut modal = Modal::confirm("Confirm", "Are you sure?");
        modal.handle_key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('y'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(modal.state, ModalState::Confirmed);
    }

    #[test]
    fn test_modal_confirm_no() {
        let mut modal = Modal::confirm("Confirm", "Are you sure?");
        modal.handle_key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Char('n'),
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(modal.state, ModalState::Cancelled);
    }

    #[test]
    fn test_modal_cancel() {
        let mut modal = Modal::confirm("Confirm", "Are you sure?");
        modal.handle_key(crossterm::event::KeyEvent::new(
            crossterm::event::KeyCode::Esc,
            crossterm::event::KeyModifiers::empty(),
        ));
        assert_eq!(modal.state, ModalState::Cancelled);
    }
}
