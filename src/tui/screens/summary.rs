//! Summary screen for the TUI
//!
//! Shows the result of download operations with actions to open folder
//! or download another video.

use crate::config::Config;
use crate::tui::app::{Screen, ScreenData};
use crate::tui::screens::ScreenResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::process::Command;

/// Action buttons in the summary screen
#[derive(Debug, Clone, Copy, PartialEq)]
enum SummaryAction {
    /// Open the output folder in file manager
    OpenFolder,
    /// Download another video (return to welcome)
    DownloadAnother,
    /// Quit the application
    Quit,
}

/// Summary screen
pub struct SummaryScreen {
    /// Currently selected action (for keyboard navigation)
    selected_action: SummaryAction,
}

impl SummaryScreen {
    pub fn new() -> Self {
        Self {
            selected_action: SummaryAction::OpenFolder,
        }
    }

    /// Open a folder in the system's file manager
    fn open_folder(path: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .args(["/select,", path])
                .spawn()
                .is_ok()
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(path).spawn().is_ok()
        }
        #[cfg(target_os = "linux")]
        {
            // Try xdg-open first, then common alternatives
            Command::new("xdg-open")
                .arg(path)
                .spawn()
                .or_else(|_| Command::new("nautilus").arg(path).spawn())
                .or_else(|_| Command::new("dolphin").arg(path).spawn())
                .or_else(|_| Command::new("thunar").arg(path).spawn())
                .is_ok()
        }
    }

    /// Draw an action button
    fn draw_action_button(
        &self,
        f: &mut Frame,
        area: Rect,
        label: &str,
        key: &str,
        is_selected: bool,
    ) {
        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::White)
        };

        let text = format!("[{}] {}", key, label);
        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(style)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    pub fn draw(&mut self, f: &mut Frame, data: &ScreenData, _config: &Config) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Actions row
                Constraint::Length(2), // Footer hints
            ])
            .split(size);

        // Title
        let title = Paragraph::new("Download Summary")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Content area
        let content = self.build_content(data);
        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, chunks[1]);

        // Actions row
        self.draw_actions(f, chunks[2], data);

        // Footer hints
        let footer_text = "↑↓: Select | Enter: Confirm | Esc: Welcome | Q: Quit";
        let footer = Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(120, 120, 120)))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[3]);
    }

    fn build_content(&self, data: &ScreenData) -> Vec<Line<'_>> {
        if let Some(ref result) = data.last_download_result {
            if result.success {
                vec![
                    Line::from(""),
                    Line::from("✓ Download completed successfully!").style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Line::from(""),
                    Line::from(format!("  Tracks: {}", result.tracks_count))
                        .style(Style::default().fg(Color::White)),
                    Line::from(format!("  Location: {}", result.output_path))
                        .style(Style::default().fg(Color::Cyan)),
                    Line::from(""),
                    Line::from("  Select an action below:"),
                    Line::from(""),
                ]
            } else {
                vec![
                    Line::from(""),
                    Line::from("✗ Download failed!")
                        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Line::from(""),
                    Line::from(format!(
                        "  Error: {}",
                        result.error.as_deref().unwrap_or("Unknown")
                    ))
                    .style(Style::default().fg(Color::Red)),
                    Line::from(""),
                    Line::from("  You can try again or check the error above."),
                    Line::from(""),
                ]
            }
        } else {
            vec![
                Line::from(""),
                Line::from("No download has been performed yet.")
                    .style(Style::default().fg(Color::Yellow)),
                Line::from(""),
                Line::from("  Start a download from the main menu."),
                Line::from(""),
            ]
        }
    }

    fn draw_actions(&self, f: &mut Frame, area: Rect, data: &ScreenData) {
        // Split actions into 3 columns
        let action_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ])
            .split(area);

        // Check if we have a successful download with output path
        let has_output_path = data_has_output_path(data);

        let (open_enabled, another_enabled, quit_enabled) = match self.selected_action {
            SummaryAction::OpenFolder => (true, false, false),
            SummaryAction::DownloadAnother => (false, true, false),
            SummaryAction::Quit => (false, false, true),
        };

        // Draw action buttons
        self.draw_action_button(
            f,
            action_chunks[0],
            "Open Folder",
            "O",
            open_enabled && has_output_path,
        );

        self.draw_action_button(
            f,
            action_chunks[1],
            "Download Another",
            "D",
            another_enabled,
        );

        self.draw_action_button(f, action_chunks[2], "Quit", "Q", quit_enabled);
    }

    pub fn handle_key(&mut self, key: KeyEvent, data: &mut ScreenData) -> ScreenResult {
        match key.code {
            KeyCode::Left => {
                self.selected_action = match self.selected_action {
                    SummaryAction::OpenFolder => SummaryAction::Quit,
                    SummaryAction::DownloadAnother => SummaryAction::OpenFolder,
                    SummaryAction::Quit => SummaryAction::DownloadAnother,
                };
                ScreenResult::Continue
            }
            KeyCode::Right => {
                self.selected_action = match self.selected_action {
                    SummaryAction::OpenFolder => SummaryAction::DownloadAnother,
                    SummaryAction::DownloadAnother => SummaryAction::Quit,
                    SummaryAction::Quit => SummaryAction::OpenFolder,
                };
                ScreenResult::Continue
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                self.selected_action = SummaryAction::OpenFolder;
                self.try_open_folder(data);
                ScreenResult::Continue
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Clear previous download data for a fresh start
                data.last_download_result = None;
                data.input_url.clear();
                data.input_artist.clear();
                data.input_album.clear();
                ScreenResult::NavigateTo(Screen::Welcome)
            }
            KeyCode::Enter => {
                match self.selected_action {
                    SummaryAction::OpenFolder => {
                        self.try_open_folder(data);
                        ScreenResult::Continue
                    }
                    SummaryAction::DownloadAnother => {
                        // Clear previous download data for a fresh start
                        data.last_download_result = None;
                        data.input_url.clear();
                        data.input_artist.clear();
                        data.input_album.clear();
                        ScreenResult::NavigateTo(Screen::Welcome)
                    }
                    SummaryAction::Quit => ScreenResult::Quit,
                }
            }
            KeyCode::Esc => ScreenResult::NavigateTo(Screen::Welcome),
            KeyCode::Char('q') | KeyCode::Char('Q') => ScreenResult::Quit,
            _ => ScreenResult::Continue,
        }
    }

    fn try_open_folder(&mut self, data: &ScreenData) {
        if let Some(ref result) = data.last_download_result
            && result.success
        {
            let path = &result.output_path;
            if Self::open_folder(path) {
                // Success - folder opened
            } else {
                // Failed - could show error in UI
            }
        }
    }
}

impl Default for SummaryScreen {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to check if data has a valid output path
fn data_has_output_path(data: &ScreenData) -> bool {
    data.last_download_result
        .as_ref()
        .map(|r| r.success && !r.output_path.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_screen_new() {
        let screen = SummaryScreen::new();
        assert_eq!(screen.selected_action, SummaryAction::OpenFolder);
    }

    #[test]
    fn test_action_navigation() {
        let mut screen = SummaryScreen::new();
        assert_eq!(screen.selected_action, SummaryAction::OpenFolder);

        // Navigate right
        screen.selected_action = match screen.selected_action {
            SummaryAction::OpenFolder => SummaryAction::DownloadAnother,
            _ => screen.selected_action,
        };
        assert_eq!(screen.selected_action, SummaryAction::DownloadAnother);

        // Navigate right again
        screen.selected_action = match screen.selected_action {
            SummaryAction::DownloadAnother => SummaryAction::Quit,
            _ => screen.selected_action,
        };
        assert_eq!(screen.selected_action, SummaryAction::Quit);
    }
}
