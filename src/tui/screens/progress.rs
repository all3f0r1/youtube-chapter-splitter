//! Download progress screen for the TUI
//!
//! Shows real-time progress of download operations.

use crate::config::Config;
use crate::tui::app::{Screen, ScreenData};
use crate::tui::components::progress::ProgressBar;
use crate::tui::screens::ScreenResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Progress display mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressMode {
    /// Minimal mode: just progress bar + percentage
    Minimal,
    /// Detailed mode: progress bar + speed + ETA + individual files
    Detailed,
}

/// Progress screen for showing download status
pub struct ProgressScreen {
    /// Current progress percentage
    progress: u8,
    /// Current status message
    status_message: String,
    /// Number of completed downloads
    completed: usize,
    /// Total number of downloads
    total: usize,
    /// Number of failed downloads
    failed: usize,
    /// Current display mode
    mode: ProgressMode,
}

impl ProgressScreen {
    pub fn new() -> Self {
        Self {
            progress: 0,
            status_message: String::new(),
            completed: 0,
            total: 0,
            failed: 0,
            mode: ProgressMode::Detailed,
        }
    }

    /// Toggle between minimal and detailed progress mode
    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            ProgressMode::Minimal => ProgressMode::Detailed,
            ProgressMode::Detailed => ProgressMode::Minimal,
        };
    }

    /// Set the progress mode
    pub fn set_mode(&mut self, mode: ProgressMode) {
        self.mode = mode;
    }

    /// Get current progress mode
    pub fn mode(&self) -> ProgressMode {
        self.mode
    }

    /// Update progress from download manager state
    pub fn update_from_manager(
        &mut self,
        percent: u8,
        completed: usize,
        total: usize,
        failed: usize,
    ) {
        self.progress = percent;
        self.completed = completed;
        self.total = total;
        self.failed = failed;

        if total > 0 {
            let current = completed + failed + 1;
            if current <= total {
                self.status_message = format!("Downloading {}/{}", current, total);
            } else if completed + failed >= total {
                self.status_message = if failed > 0 {
                    format!("Completed with {} errors", failed)
                } else {
                    "All downloads complete!".to_string()
                };
            }
        }
    }

    /// Set the status message directly
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    /// Check if all downloads are complete
    pub fn is_complete(&self) -> bool {
        self.total > 0 && (self.completed + self.failed) >= self.total
    }

    /// Draw the progress screen
    pub fn draw(&mut self, f: &mut Frame, _data: &ScreenData, _config: &Config) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Overall status
                Constraint::Length(5), // Progress bar
                Constraint::Min(0),    // Details/Log
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Title
        self.draw_title(f, chunks[0]);

        // Overall status
        self.draw_status(f, chunks[1]);

        // Progress bar
        self.draw_progress_bar(f, chunks[2]);

        // Details
        self.draw_details(f, chunks[3]);

        // Footer
        self.draw_footer(f, chunks[4]);
    }

    fn draw_title(&self, f: &mut Frame, area: Rect) {
        let title = Paragraph::new("Download Progress")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect) {
        let status_text = if self.total > 0 {
            format!(
                "Progress: {} / {} downloads complete | {} errors",
                self.completed, self.total, self.failed
            )
        } else {
            "Initializing...".to_string()
        };

        let paragraph = Paragraph::new(status_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    fn draw_progress_bar(&mut self, f: &mut Frame, area: Rect) {
        let detailed = self.mode == ProgressMode::Detailed;
        let bar = ProgressBar::new()
            .with_progress(self.progress as u16)
            .with_total(100)
            .with_message(self.status_message.clone())
            .with_detailed(detailed);

        bar.draw(f, area);
    }

    fn draw_details(&self, f: &mut Frame, area: Rect) {
        if self.mode == ProgressMode::Minimal {
            // Minimal mode - compact display
            let lines = vec![
                Line::from(""),
                Line::from(self.status_message.as_str()).style(Style::default().fg(Color::Cyan)),
            ];
            let paragraph = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, area);
        } else {
            // Detailed mode - full status
            let lines = vec![
                Line::from(""),
                Line::from("Download Status:").style(Style::default().fg(Color::Cyan)),
                Line::from(""),
                if self.completed > 0 {
                    Line::from(format!("  ✓ {} completed", self.completed))
                        .style(Style::default().fg(Color::Green))
                } else {
                    Line::from("")
                },
                if self.failed > 0 {
                    Line::from(format!("  ✗ {} failed", self.failed))
                        .style(Style::default().fg(Color::Red))
                } else {
                    Line::from("")
                },
                Line::from(""),
                Line::from(format!(
                    "Mode: {} | Press P to toggle",
                    if self.mode == ProgressMode::Detailed {
                        "Detailed"
                    } else {
                        "Minimal"
                    }
                ))
                .style(Style::default().fg(Color::Rgb(120, 120, 120))),
            ];

            let paragraph = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, area);
        }
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let footer_text = if self.is_complete() {
            format!(
                "Enter: Results | Esc: Back | P: Mode ({})",
                if self.mode == ProgressMode::Detailed {
                    "Detailed"
                } else {
                    "Minimal"
                }
            )
        } else {
            format!(
                "Esc: Cancel | P: Mode ({})",
                if self.mode == ProgressMode::Detailed {
                    "Detailed"
                } else {
                    "Minimal"
                }
            )
        };

        let paragraph = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(150, 150, 150)))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    pub fn handle_key(&mut self, key: KeyEvent, _data: &mut ScreenData) -> ScreenResult {
        match key.code {
            KeyCode::Esc => {
                // Allow cancel only if complete
                if self.is_complete() {
                    ScreenResult::NavigateTo(Screen::Welcome)
                } else {
                    ScreenResult::Continue
                }
            }
            KeyCode::Enter => {
                if self.is_complete() {
                    ScreenResult::NavigateTo(Screen::Summary)
                } else {
                    ScreenResult::Continue
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.toggle_mode();
                ScreenResult::Continue
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if self.is_complete() {
                    ScreenResult::Quit
                } else {
                    ScreenResult::Continue
                }
            }
            _ => ScreenResult::Continue,
        }
    }
}

impl Default for ProgressScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_screen_new() {
        let screen = ProgressScreen::new();
        assert_eq!(screen.progress, 0);
        assert_eq!(screen.completed, 0);
        assert_eq!(screen.total, 0);
        assert!(!screen.is_complete());
    }

    #[test]
    fn test_progress_screen_update() {
        let mut screen = ProgressScreen::new();
        screen.update_from_manager(50, 1, 2, 0);
        assert_eq!(screen.progress, 50);
        assert_eq!(screen.completed, 1);
        assert_eq!(screen.total, 2);
        assert!(!screen.is_complete());
    }

    #[test]
    fn test_progress_screen_complete() {
        let mut screen = ProgressScreen::new();
        screen.update_from_manager(100, 5, 5, 0);
        assert_eq!(screen.progress, 100);
        assert_eq!(screen.completed, 5);
        assert_eq!(screen.total, 5);
        assert!(screen.is_complete());
    }

    #[test]
    fn test_progress_screen_with_failures() {
        let mut screen = ProgressScreen::new();
        screen.update_from_manager(100, 3, 5, 2);
        assert_eq!(screen.completed, 3);
        assert_eq!(screen.failed, 2);
        assert!(screen.is_complete());
    }
}
