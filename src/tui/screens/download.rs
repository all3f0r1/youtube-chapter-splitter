//! Download screen for the TUI
//!
//! Allows users to input URLs and download options.
//! Supports auto-detection of artist/album from video metadata.

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
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Download screen
pub struct DownloadScreen {
    url_input: TextInput,
    artist_input: TextInput,
    album_input: TextInput,
    focused_field: FocusedField,
    download_state: DownloadState,
    /// Track if user has manually modified artist (to clear auto-detected flag)
    artist_modified: bool,
    /// Track if user has manually modified album (to clear auto-detected flag)
    album_modified: bool,
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
            artist_modified: false,
            album_modified: false,
        }
    }

    pub fn draw(&mut self, f: &mut Frame, data: &ScreenData, _config: &Config) {
        // Sync URL from screen data
        if self.url_input.value != data.input_url {
            self.url_input.value = data.input_url.clone();
        }

        // Sync artist/album from screen data if auto-detected and not modified by user
        if data.metadata_autodetected {
            // Only sync if the user hasn't manually modified the field
            if !self.artist_modified && self.artist_input.value != data.input_artist {
                self.artist_input.value = data.input_artist.clone();
            }
            if !self.album_modified && self.album_input.value != data.input_album {
                self.album_input.value = data.input_album.clone();
            }
        }

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

        // Update artist/album input titles and colors based on auto-detection
        let (artist_title, artist_color) = if data.metadata_autodetected
            && !self.artist_input.value.is_empty()
            && !self.artist_modified
        {
            ("Artist (auto-detected) ✓", Color::Rgb(100, 200, 100))
        } else if self.artist_modified {
            ("Artist (edited)", Color::Rgb(200, 200, 150))
        } else {
            ("Artist (optional)", Color::Gray)
        };

        let (album_title, album_color) = if data.metadata_autodetected
            && !self.album_input.value.is_empty()
            && !self.album_modified
        {
            ("Album (auto-detected) ✓", Color::Rgb(100, 200, 100))
        } else if self.album_modified {
            ("Album (edited)", Color::Rgb(200, 200, 150))
        } else {
            ("Album (optional)", Color::Gray)
        };

        self.artist_input.title = artist_title.to_string();
        self.album_input.title = album_title.to_string();

        // Draw inputs with custom title colors for auto-detected fields
        self.draw_input_with_title_color(
            f,
            content_chunks[2],
            &self.artist_input,
            artist_color,
        );
        self.draw_input_with_title_color(
            f,
            content_chunks[4],
            &self.album_input,
            album_color,
        );

        // URL input (default color)
        self.url_input.draw(f, content_chunks[0]);

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

    /// Draw a TextInput with a custom title color
    fn draw_input_with_title_color(
        &self,
        f: &mut Frame,
        area: Rect,
        input: &TextInput,
        title_color: Color,
    ) {
        let title_style = Style::default().fg(title_color);
        let title = Span::styled(&input.title, title_style);

        let block = Block::default()
            .borders(if input.focused {
                Borders::ALL
            } else {
                Borders::ALL
            })
            .border_style(if input.focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            })
            .title(title);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let input_text = if input.value.is_empty() {
            Span::styled(&input.placeholder, Style::default().fg(Color::DarkGray))
        } else {
            // Truncate if too long for display
            let display_value = if input.value.len() > (inner.width as usize).saturating_sub(2) {
                let start = input.value.len().saturating_sub((inner.width as usize).saturating_sub(4));
                format!("...{}", &input.value[start..])
            } else {
                input.value.clone()
            };
            Span::from(display_value)
        };

        let paragraph = Paragraph::new(input_text)
            .alignment(ratatui::layout::Alignment::Left);
        f.render_widget(paragraph, inner);
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
            // Track when user modifies artist/album fields
            if self.focused_field == FocusedField::Artist {
                // Mark as modified if user typed something (not just navigation)
                if matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete) {
                    self.artist_modified = true;
                }
            }
            if self.focused_field == FocusedField::Album {
                if matches!(key.code, KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete) {
                    self.album_modified = true;
                }
            }

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

    pub fn reset(&mut self) {
        self.url_input.clear();
        self.artist_input.clear();
        self.album_input.clear();
        self.focused_field = FocusedField::Url;
        self.download_state = DownloadState::Idle;
        self.artist_modified = false;
        self.album_modified = false;
    }
}

impl Default for DownloadScreen {
    fn default() -> Self {
        Self::new()
    }
}
