//! Playlist selection screen implementation
//!
//! Allows users to select videos from a playlist before downloading.

use crate::config::Config;
use crate::playlist::PlaylistInfo;
use crate::tui::app::{Screen, ScreenData};
use crate::tui::components::spinner::Spinner;
use crate::tui::screens::ScreenResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

/// State of the playlist screen
#[derive(Debug, Clone)]
enum PlaylistState {
    /// Loading playlist info
    Loading,
    /// Playlist loaded successfully
    Loaded { playlist: PlaylistInfo },
    /// Error loading playlist
    Error(String),
}

/// Playlist selection screen
pub struct PlaylistScreen {
    state: PlaylistState,
    /// Selected video indices (for multi-select)
    selected_videos: Vec<bool>,
    /// Currently focused index in the list
    focused_index: usize,
    /// Scroll offset
    offset: usize,
    /// Spinner for loading state
    spinner: Spinner,
}

impl PlaylistScreen {
    pub fn new() -> Self {
        Self {
            state: PlaylistState::Loading,
            selected_videos: Vec::new(),
            focused_index: 0,
            offset: 0,
            spinner: Spinner::new(),
        }
    }

    /// Load playlist from URL
    pub fn load_from_url(&mut self, url: &str, config: &Config) {
        self.state = PlaylistState::Loading;
        self.selected_videos.clear();
        self.focused_index = 0;
        self.offset = 0;

        // Try to load playlist info
        let cookies = config.cookies_from_browser.as_deref();
        let result = crate::playlist::get_playlist_info(url, cookies);

        match result {
            Ok(playlist) => {
                // Initialize selection vector (all selected by default)
                let count = playlist.videos.len();
                self.selected_videos = vec![true; count];
                self.state = PlaylistState::Loaded { playlist };
            }
            Err(e) => {
                self.state = PlaylistState::Error(e.to_string());
            }
        }
    }

    /// Get the number of selected videos
    fn selected_count(&self) -> usize {
        self.selected_videos.iter().filter(|&&s| s).count()
    }

    /// Toggle selection for focused video
    fn toggle_selection(&mut self) {
        if let Some(playlist) = self.get_playlist() {
            if self.focused_index < playlist.videos.len() {
                self.selected_videos[self.focused_index] =
                    !self.selected_videos[self.focused_index];
            }
        }
    }

    /// Select all videos
    fn select_all(&mut self) {
        for selection in &mut self.selected_videos {
            *selection = true;
        }
    }

    /// Deselect all videos
    fn deselect_all(&mut self) {
        for selection in &mut self.selected_videos {
            *selection = false;
        }
    }

    /// Get reference to playlist if loaded
    fn get_playlist(&self) -> Option<&PlaylistInfo> {
        match &self.state {
            PlaylistState::Loaded { playlist } => Some(playlist),
            _ => None,
        }
    }

    /// Get selected video URLs
    pub fn get_selected_urls(&self) -> Vec<String> {
        match &self.state {
            PlaylistState::Loaded { playlist } => playlist
                .videos
                .iter()
                .enumerate()
                .filter(|(i, _)| *i < self.selected_videos.len() && self.selected_videos[*i])
                .map(|(_, v)| v.url.clone())
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Format duration as MM:SS or HH:MM:SS
    fn format_duration(seconds: f64) -> String {
        let total_secs = seconds as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        let hours = mins / 60;
        let mins = mins % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }

    pub fn draw(&mut self, f: &mut Frame, _data: &ScreenData, _config: &Config) {
        let size = f.area();

        // Clone the state to avoid borrow checker issues
        let state = self.state.clone();
        match &state {
            PlaylistState::Loading => {
                self.draw_loading(f, size);
            }
            PlaylistState::Loaded { playlist } => {
                self.draw_loaded(f, size, playlist);
            }
            PlaylistState::Error(err) => {
                self.draw_error(f, size, err);
            }
        }
    }

    fn draw_loading(&mut self, f: &mut Frame, area: Rect) {
        // Draw centered loading message
        let lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from("Loading playlist...").style(Style::default().fg(Color::Cyan)),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);

        // Draw spinner at bottom center
        if area.height > 5 {
            let spinner_area = Rect {
                x: area.x + area.width / 2 - 2,
                y: area.y + area.height / 2,
                width: 4,
                height: 1,
            };
            self.spinner.draw(f, spinner_area);
        }
    }

    fn draw_loaded(&mut self, f: &mut Frame, area: Rect, playlist: &PlaylistInfo) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Video list
                Constraint::Length(3), // Footer with actions
            ])
            .split(area);

        // Header
        self.draw_header(f, chunks[0], playlist);

        // Video list
        self.draw_video_list(f, chunks[1], playlist);

        // Footer
        self.draw_footer(f, chunks[2], playlist);
    }

    fn draw_header(&self, f: &mut Frame, area: Rect, playlist: &PlaylistInfo) {
        let title = format!("Playlist: {}", playlist.title);
        let info = format!(
            "{} videos | {} selected",
            playlist.videos.len(),
            self.selected_count()
        );

        let lines = vec![
            Line::from(vec![Span::styled(
                title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                info,
                Style::default().fg(Color::Rgb(150, 150, 150)),
            )]),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    fn draw_video_list(&mut self, f: &mut Frame, area: Rect, playlist: &PlaylistInfo) {
        let inner_height = area.height.saturating_sub(2) as usize;

        // Adjust offset to keep focused item in view
        if self.focused_index < self.offset {
            self.offset = self.focused_index;
        } else if self.focused_index >= self.offset + inner_height {
            self.offset = self.focused_index - inner_height + 1;
        }

        // Build list items with checkboxes
        let list_items: Vec<ListItem> = playlist
            .videos
            .iter()
            .enumerate()
            .skip(self.offset)
            .take(inner_height)
            .map(|(i, video)| {
                let is_selected = i < self.selected_videos.len() && self.selected_videos[i];
                let is_focused = i == self.focused_index;

                let checkbox = if is_selected { "[✓]" } else { "[ ]" };
                let index = i + 1;
                let duration = Self::format_duration(video.duration);

                let style = if is_focused {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                // Truncate title if too long
                let title = if video.title.len() > 50 {
                    format!("{}...", &video.title[..47])
                } else {
                    video.title.clone()
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!(" {:3} ", index)),
                    Span::styled(checkbox, Style::default().fg(Color::Green)),
                    Span::raw(format!(" {} ", title)),
                    Span::styled(
                        format!("({})", duration),
                        Style::default().fg(Color::Rgb(120, 120, 120)),
                    ),
                ]))
                .style(style)
            })
            .collect();

        let list = List::new(list_items.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(100, 150, 200))),
        );

        let mut state = ListState::default();
        // Calculate the relative position within the visible window
        let visible_index = if self.focused_index >= self.offset {
            self.focused_index - self.offset
        } else {
            0
        };
        if visible_index < list_items.len() {
            state.select(Some(visible_index));
        }

        f.render_stateful_widget(list, area, &mut state);
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect, playlist: &PlaylistInfo) {
        let selected = self.selected_count();

        let actions = vec![
            "↑↓: Navigate".to_string(),
            "Space: Toggle".to_string(),
            "A: Select All".to_string(),
            "D: Deselect All".to_string(),
            format!("Enter: Download ({})", selected),
            "Esc: Cancel".to_string(),
        ];

        let text = actions.join("  |  ");

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(150, 150, 150)))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    fn draw_error(&self, f: &mut Frame, area: Rect, error: &str) {
        // Draw error modal-like appearance
        let modal_width = 60.min(area.width.saturating_sub(4));
        let modal_height = 10.min(area.height.saturating_sub(4));

        let x = (area.width - modal_width) / 2;
        let y = (area.height - modal_height) / 2;

        let modal_area = Rect {
            x,
            y,
            width: modal_width,
            height: modal_height,
        };

        // Clear area
        f.render_widget(Clear, modal_area);

        let lines = vec![
            Line::from(""),
            Line::from("Error Loading Playlist")
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Line::from(""),
            Line::from(error).style(Style::default().fg(Color::White)),
            Line::from(""),
            Line::from("Press Esc to go back")
                .style(Style::default().fg(Color::Rgb(150, 150, 150))),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            );

        f.render_widget(paragraph, modal_area);
    }

    pub fn handle_key(&mut self, key: KeyEvent, data: &mut ScreenData) -> ScreenResult {
        // Always allow quit
        if matches!(
            key.code,
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')
        ) {
            return ScreenResult::NavigateTo(Screen::Download);
        }

        // Check state and handle accordingly
        let is_loading = matches!(self.state, PlaylistState::Loading);
        let is_error = matches!(self.state, PlaylistState::Error(_));

        if is_loading {
            // Loading state - just wait
            ScreenResult::Continue
        } else if is_error {
            // Error state - go back
            ScreenResult::NavigateTo(Screen::Download)
        } else {
            // Loaded state - handle navigation
            // Extract playlist reference to avoid double borrow
            let playlist = match &self.state {
                PlaylistState::Loaded { playlist } => playlist.clone(),
                _ => return ScreenResult::Continue,
            };
            self.handle_loaded_key(key, &playlist, data)
        }
    }

    fn handle_loaded_key(
        &mut self,
        key: KeyEvent,
        playlist: &PlaylistInfo,
        data: &mut ScreenData,
    ) -> ScreenResult {
        let video_count = playlist.videos.len();
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                if self.focused_index < video_count.saturating_sub(1) {
                    self.focused_index += 1;
                }
                ScreenResult::Continue
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.focused_index > 0 {
                    self.focused_index -= 1;
                }
                ScreenResult::Continue
            }
            KeyCode::PageDown => {
                let page_size = 10;
                self.focused_index = (self.focused_index + page_size).min(video_count - 1);
                ScreenResult::Continue
            }
            KeyCode::PageUp => {
                let page_size = 10;
                self.focused_index = self.focused_index.saturating_sub(page_size);
                ScreenResult::Continue
            }
            KeyCode::Home => {
                self.focused_index = 0;
                ScreenResult::Continue
            }
            KeyCode::End => {
                self.focused_index = video_count.saturating_sub(1);
                ScreenResult::Continue
            }
            KeyCode::Char(' ') => {
                self.toggle_selection();
                ScreenResult::Continue
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.select_all();
                ScreenResult::Continue
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                self.deselect_all();
                ScreenResult::Continue
            }
            KeyCode::Enter => {
                if self.selected_count() > 0 {
                    // Store selected URLs and proceed
                    data.input_url = self.get_selected_urls().join("\n");
                    ScreenResult::NavigateTo(Screen::Summary)
                } else {
                    ScreenResult::Continue
                }
            }
            _ => ScreenResult::Continue,
        }
    }
}

impl Default for PlaylistScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(PlaylistScreen::format_duration(45.0), "00:45");
        assert_eq!(PlaylistScreen::format_duration(125.0), "02:05");
        assert_eq!(PlaylistScreen::format_duration(3665.0), "01:01:05");
    }

    #[test]
    fn test_new_playlist_screen() {
        let screen = PlaylistScreen::new();
        assert!(matches!(screen.state, PlaylistState::Loading));
        assert_eq!(screen.focused_index, 0);
        assert!(screen.selected_videos.is_empty());
    }

    #[test]
    fn test_selection_count() {
        let mut screen = PlaylistScreen::new();
        screen.selected_videos = vec![true, false, true, false, true];
        assert_eq!(screen.selected_count(), 3);
    }
}
