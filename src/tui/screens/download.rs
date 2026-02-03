//! Download screen for the TUI
//!
//! Allows users to input URLs and download options.
//! Supports auto-detection of artist/album from video metadata.

use crate::config::Config;
use crate::downloader;
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
use std::time::Instant;

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
    /// Metadata fetching state
    metadata_state: MetadataState,
    /// Last URL change timestamp for debounce
    last_url_change: Option<Instant>,
    /// Debounce delay in milliseconds before fetching metadata
    metadata_debounce_ms: u64,
    /// Last URL for which metadata was fetched (to avoid re-fetching)
    last_fetched_url: Option<String>,
    /// Any error from metadata fetching
    metadata_error: Option<String>,
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

/// State of metadata auto-detection from URL
#[derive(Debug, Clone, PartialEq, Eq)]
enum MetadataState {
    /// No URL entered yet
    Idle,
    /// Fetching video metadata
    Loading,
    /// Metadata successfully detected
    Detected,
    /// Detection failed
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
            metadata_state: MetadataState::Idle,
            last_url_change: None,
            metadata_debounce_ms: 700, // 700ms debounce
            last_fetched_url: None,
            metadata_error: None,
        }
    }

    pub fn draw(&mut self, f: &mut Frame, data: &ScreenData, config: &Config) {
        // Sync URL from screen data
        if self.url_input.value != data.input_url {
            self.url_input.value = data.input_url.clone();
        }

        // Check if we should fetch metadata (debounce)
        if matches!(
            self.metadata_state,
            MetadataState::Idle | MetadataState::Error(_)
        ) && let Some(last_change) = self.last_url_change
        {
            let elapsed = last_change.elapsed().as_millis();
            if elapsed >= self.metadata_debounce_ms as u128 {
                let current_url = self.url_input.value.trim().to_string();
                // Check if URL looks valid and hasn't been fetched yet
                if self.is_valid_youtube_url(&current_url)
                    && self.last_fetched_url.as_ref() != Some(&current_url)
                {
                    // Extract cookies before mutable borrow to avoid borrow checker issue
                    let cookies_from_browser = config.cookies_from_browser.as_deref();
                    self.fetch_metadata(&current_url, cookies_from_browser);
                }
            }
        }

        // Sync artist/album from screen data if auto-detected and not modified by user
        if data.metadata_autodetected
            && !self.artist_modified
            && self.artist_input.value != data.input_artist
        {
            self.artist_input.value = data.input_artist.clone();
        }
        if data.metadata_autodetected
            && !self.album_modified
            && self.album_input.value != data.input_album
        {
            self.album_input.value = data.input_album.clone();
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
        let (artist_title, artist_color) = match &self.metadata_state {
            MetadataState::Loading => ("Artist (loading...)", Color::Rgb(200, 200, 100)),
            MetadataState::Detected if !self.artist_input.value.is_empty() => {
                ("Artist (auto-detected) ✓", Color::Rgb(100, 200, 100))
            }
            _ if self.artist_modified => ("Artist (edited)", Color::Rgb(200, 200, 150)),
            _ => ("Artist (optional)", Color::Gray),
        };

        let (album_title, album_color) = match &self.metadata_state {
            MetadataState::Loading => ("Album (loading...)", Color::Rgb(200, 200, 100)),
            MetadataState::Detected if !self.album_input.value.is_empty() => {
                ("Album (auto-detected) ✓", Color::Rgb(100, 200, 100))
            }
            _ if self.album_modified => ("Album (edited)", Color::Rgb(200, 200, 150)),
            _ => ("Album (optional)", Color::Gray),
        };

        self.artist_input.title = artist_title.to_string();
        self.album_input.title = album_title.to_string();

        // Draw inputs with custom title colors for auto-detected fields
        self.draw_input_with_title_color(f, content_chunks[2], &self.artist_input, artist_color);
        self.draw_input_with_title_color(f, content_chunks[4], &self.album_input, album_color);

        // URL input (default color)
        self.url_input.draw(f, content_chunks[0]);

        // Draw status/download info
        self.draw_status(f, content_chunks[6], data);

        // Draw download button or status
        self.draw_download_button(f, content_chunks[6]);
    }

    fn draw_status(&self, f: &mut Frame, area: Rect, _data: &ScreenData) {
        // Show metadata error if present
        if let Some(ref error) = self.metadata_error {
            let status_text = vec![
                Line::from(""),
                Line::from(""),
                Line::from("Looking for metadata...").style(Style::default().fg(Color::Yellow)),
                Line::from(""),
                Line::from("Metadata error:").style(Style::default().fg(Color::Red)),
                Line::from(format!("  {}", error)).style(Style::default().fg(Color::Red)),
            ];

            let paragraph = Paragraph::new(status_text)
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, area);
            return;
        }

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
            .borders(Borders::ALL)
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
                let start = input
                    .value
                    .len()
                    .saturating_sub((inner.width as usize).saturating_sub(4));
                format!("...{}", &input.value[start..])
            } else {
                input.value.clone()
            };
            Span::from(display_value)
        };

        let paragraph = Paragraph::new(input_text).alignment(ratatui::layout::Alignment::Left);
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
            // Track URL changes for metadata fetching debounce
            if self.focused_field == FocusedField::Url {
                // Check if URL actually changed (not just cursor movement)
                let url_changed = matches!(key.code, KeyCode::Char(_))
                    || matches!(key.code, KeyCode::Backspace)
                    || matches!(key.code, KeyCode::Delete);
                if url_changed {
                    // Reset metadata state and error when URL changes
                    self.metadata_state = MetadataState::Idle;
                    self.metadata_error = None;
                    self.last_url_change = Some(Instant::now());
                }
            }

            // Track when user modifies artist/album fields
            if self.focused_field == FocusedField::Artist
                && matches!(
                    key.code,
                    KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
                )
            {
                self.artist_modified = true;
            }
            if self.focused_field == FocusedField::Album
                && matches!(
                    key.code,
                    KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
                )
            {
                self.album_modified = true;
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
        self.metadata_state = MetadataState::Idle;
        self.last_url_change = None;
        self.last_fetched_url = None;
        self.metadata_error = None;
    }

    /// Check if the URL looks like a valid YouTube URL
    fn is_valid_youtube_url(&self, url: &str) -> bool {
        let trimmed = url.trim();
        trimmed.contains("youtube.com") || trimmed.contains("youtu.be")
    }

    /// Fetch metadata from YouTube URL
    fn fetch_metadata(&mut self, url: &str, cookies_from_browser: Option<&str>) {
        self.metadata_state = MetadataState::Loading;
        self.metadata_error = None;

        match downloader::get_video_info(url, cookies_from_browser) {
            Ok(video_info) => {
                let (artist, album) =
                    crate::utils::parse_artist_album(&video_info.title, &video_info.uploader);

                // Apply the detected metadata to the input fields
                self.artist_input.value = artist.clone();
                self.artist_input.cursor = artist.len();

                self.album_input.value = album.clone();
                self.album_input.cursor = album.len();

                self.metadata_state = MetadataState::Detected;
                self.last_fetched_url = Some(url.to_string());
            }
            Err(e) => {
                self.metadata_state = MetadataState::Error(e.to_string());
                self.metadata_error = Some(e.to_string());
            }
        }
    }
}

impl Default for DownloadScreen {
    fn default() -> Self {
        Self::new()
    }
}
