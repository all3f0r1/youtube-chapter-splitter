//! Download screen for the TUI
//!
//! Allows users to input URLs and download options.

use crate::config::Config;
use crate::playlist;
use crate::tui::app::{Screen, ScreenData};
use crate::tui::components::input::TextInput;
use crate::tui::screens::{PlaylistScreen, ScreenResult};
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Download screen
pub struct DownloadScreen {
    url_input: TextInput,
    artist_input: TextInput,
    album_input: TextInput,
    focused_field: FocusedField,
    download_state: DownloadState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedField {
    Url,
    Artist,
    Album,
    DownloadButton,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DownloadState {
    Idle,
    Downloading,
    Error(String),
}

impl DownloadScreen {
    pub fn new() -> Self {
        Self {
            url_input: TextInput::new()
                .with_placeholder("Enter YouTube URL here...")
                .with_title("URL")
                .with_focused(true),
            artist_input: TextInput::new()
                .with_placeholder("Leave empty to auto-detect")
                .with_title("Artist (optional)"),
            album_input: TextInput::new()
                .with_placeholder("Leave empty to auto-detect")
                .with_title("Album (optional)"),
            focused_field: FocusedField::Url,
            download_state: DownloadState::Idle,
        }
    }

    pub fn draw(&mut self, f: &mut Frame, data: &ScreenData, _config: &Config) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(4), // Footer space
            ])
            .split(size);

        // Title
        let title = Paragraph::new("Download from YouTube")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Content area
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // URL input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Artist input
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Album input
                Constraint::Length(2), // Spacer
                Constraint::Min(0),    // Status/Instructions
            ])
            .split(chunks[1]);

        // Update focus states
        self.url_input.focused = self.focused_field == FocusedField::Url;
        self.artist_input.focused = self.focused_field == FocusedField::Artist;
        self.album_input.focused = self.focused_field == FocusedField::Album;

        // Draw inputs
        self.url_input.draw(f, content_chunks[0]);
        self.artist_input.draw(f, content_chunks[2]);
        self.album_input.draw(f, content_chunks[4]);

        // Draw status/download info
        self.draw_status(f, content_chunks[6], data);

        // Draw download button or status
        self.draw_download_button(f, content_chunks[6]);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect, _data: &ScreenData) {
        let status_text = match &self.download_state {
            DownloadState::Idle => {
                vec![
                    Line::from(""),
                    Line::from("Instructions:"),
                    Line::from("  • Enter a YouTube URL to download"),
                    Line::from("  • Optionally specify artist/album"),
                    Line::from("  • Press Enter on URL to start download"),
                    Line::from("  • Press Tab to move between fields"),
                    Line::from(""),
                    Line::from("Supported:"),
                    Line::from("  • Individual videos: youtube.com/watch?v=..."),
                    Line::from("  • Short links: youtu.be/..."),
                    Line::from("  • Playlists: (select videos before downloading)"),
                ]
            }
            DownloadState::Downloading => {
                vec![
                    Line::from(""),
                    Line::from("Downloading...").style(Style::default().fg(Color::Yellow)),
                    Line::from("(Download progress coming soon)"),
                ]
            }
            DownloadState::Error(err) => {
                vec![
                    Line::from(""),
                    Line::from("Download failed:").style(Style::default().fg(Color::Red)),
                    Line::from(format!("  {}", err)),
                ]
            }
        };

        let paragraph = Paragraph::new(status_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    fn draw_download_button(&self, f: &mut Frame, area: Rect) {
        let button_text = if self.download_state == DownloadState::Idle {
            if self.focused_field == FocusedField::DownloadButton && !self.url_input.is_empty() {
                "[Download Now]"
            } else {
                ""
            }
        } else {
            ""
        };

        if !button_text.is_empty() {
            let button_area = Rect {
                x: area.x + area.width.saturating_sub(16),
                y: area.y,
                width: 14.min(area.width),
                height: 1,
            };

            let paragraph = Paragraph::new(button_text)
                .alignment(Alignment::Center)
                .style(
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(ratatui::style::Modifier::BOLD),
                );
            f.render_widget(paragraph, button_area);
        }
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        data: &mut ScreenData,
        _playlist_screen: &mut PlaylistScreen,
    ) -> ScreenResult {
        // Check if an input field should handle this key
        let input_handled = match self.focused_field {
            FocusedField::Url => self.url_input.handle_key_event(key),
            FocusedField::Artist => self.artist_input.handle_key_event(key),
            FocusedField::Album => self.album_input.handle_key_event(key),
            FocusedField::DownloadButton => false,
        };

        if input_handled {
            // Clear any error state when user modifies input
            if matches!(self.download_state, DownloadState::Error(_)) {
                self.download_state = DownloadState::Idle;
            }
            // Sync URL to screen data
            data.input_url = self.url_input.value.clone();
            data.input_artist = self.artist_input.value.clone();
            data.input_album = self.album_input.value.clone();
            return ScreenResult::Continue;
        }

        match key.code {
            KeyCode::Tab => {
                // Cycle focus
                self.focused_field = match self.focused_field {
                    FocusedField::Url => FocusedField::Artist,
                    FocusedField::Artist => FocusedField::Album,
                    FocusedField::Album => FocusedField::Url,
                    FocusedField::DownloadButton => FocusedField::Url,
                };
                ScreenResult::Continue
            }
            KeyCode::BackTab => {
                // Reverse cycle focus
                self.focused_field = match self.focused_field {
                    FocusedField::Url => FocusedField::Album,
                    FocusedField::Artist => FocusedField::Url,
                    FocusedField::Album => FocusedField::Artist,
                    FocusedField::DownloadButton => FocusedField::Album,
                };
                ScreenResult::Continue
            }
            KeyCode::Enter => {
                if self.focused_field == FocusedField::Url && !self.url_input.is_empty() {
                    let url = self.url_input.value.trim();

                    // Validate URL is a YouTube URL
                    if !url.contains("youtube.com") && !url.contains("youtu.be") {
                        self.download_state = DownloadState::Error(
                            "Invalid YouTube URL. Use youtube.com or youtu.be links.".to_string(),
                        );
                        return ScreenResult::Continue;
                    }

                    // Check if this is a playlist URL
                    if playlist::is_playlist_url(url).is_some() {
                        // Load playlist and navigate to playlist screen
                        data.input_url = url.to_string();
                        data.input_artist = self.artist_input.value.clone();
                        data.input_album = self.album_input.value.clone();
                        // Note: playlist_screen will be loaded in app.rs after navigation
                        return ScreenResult::NavigateTo(Screen::Playlist);
                    }

                    // Single video - start download
                    self.start_download(data);
                    ScreenResult::NavigateTo(Screen::Progress)
                } else if self.focused_field == FocusedField::DownloadButton {
                    self.start_download(data);
                    ScreenResult::NavigateTo(Screen::Progress)
                } else {
                    // Move to next field
                    self.focused_field = FocusedField::Url;
                    ScreenResult::Continue
                }
            }
            KeyCode::Esc => {
                // Clear and go back
                self.reset();
                ScreenResult::NavigateTo(Screen::Welcome)
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => ScreenResult::Quit,
            _ => ScreenResult::Continue,
        }
    }

    fn start_download(&mut self, data: &mut ScreenData) {
        // Update screen data with current input values
        data.input_url = self.url_input.value.trim().to_string();
        data.input_artist = self.artist_input.value.trim().to_string();
        data.input_album = self.album_input.value.trim().to_string();

        // Mark as downloading - actual download will be handled by download_manager
        self.download_state = DownloadState::Downloading;
    }

    fn reset(&mut self) {
        self.url_input.clear();
        self.artist_input.clear();
        self.album_input.clear();
        self.focused_field = FocusedField::Url;
        self.download_state = DownloadState::Idle;
    }
}

impl Default for DownloadScreen {
    fn default() -> Self {
        Self::new()
    }
}
