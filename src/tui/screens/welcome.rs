//! Welcome screen for the TUI
//!
//! This is the first screen shown when launching the TUI.

use crate::config::Config;
use crate::tui::app::{Screen, ScreenData};
use crate::tui::components::spinner::Spinner;
use crate::tui::screens::ScreenResult;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Loading state for the welcome screen
#[derive(Debug, Clone, PartialEq)]
pub enum WelcomeState {
    /// Normal menu display
    Idle,
    /// Fetching video information
    FetchingVideoInfo,
    /// Loading playlist
    LoadingPlaylist,
    /// Checking for updates
    CheckingUpdates,
}

/// Welcome screen
pub struct WelcomeScreen {
    /// Current state
    state: WelcomeState,
    /// Spinner for loading animations
    spinner: Spinner,
    /// Current loading message
    loading_message: String,
}

impl WelcomeScreen {
    pub fn new() -> Self {
        Self {
            state: WelcomeState::Idle,
            spinner: Spinner::new(),
            loading_message: String::new(),
        }
    }

    /// Set the loading state
    pub fn set_loading(&mut self, state: WelcomeState, message: impl Into<String>) {
        self.loading_message = message.into();
        if state != WelcomeState::Idle {
            self.spinner.reset();
        }
        self.state = state;
    }

    /// Reset to idle state
    pub fn set_idle(&mut self) {
        self.state = WelcomeState::Idle;
        self.loading_message.clear();
    }

    /// Check if currently loading
    pub fn is_loading(&self) -> bool {
        self.state != WelcomeState::Idle
    }

    /// Get current state
    pub fn state(&self) -> &WelcomeState {
        &self.state
    }

    pub fn draw(&mut self, f: &mut Frame, _data: &ScreenData, _config: &Config) {
        let size = f.area();

        // Create layout with margins
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Top margin
                Constraint::Min(0),    // Content
                Constraint::Length(4), // Footer space (reserved by app)
            ])
            .split(size);

        match self.state {
            WelcomeState::Idle => {
                self.draw_idle(f, chunks[1]);
            }
            _ => {
                self.draw_loading(f, chunks[1]);
            }
        }
    }

    fn draw_idle(&self, f: &mut Frame, area: Rect) {
        let content_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Title
                Constraint::Length(3),  // Spacer
                Constraint::Length(15), // Menu
                Constraint::Min(0),     // Info
            ])
            .split(area);

        // Draw title
        self.draw_title(f, content_chunks[0]);

        // Draw menu
        self.draw_menu(f, content_chunks[2]);
    }

    fn draw_loading(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Loading content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Title
        let title = Paragraph::new("YouTube Chapter Splitter")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Loading content with spinner
        let spinner_text = self.spinner.current();
        let loading_lines = vec![
            Line::from(""),
            Line::from(format!("{} {}", spinner_text, self.loading_message))
                .style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from("Please wait...").style(Style::default().fg(Color::Rgb(150, 150, 150))),
            Line::from(""),
            Line::from("").style(Style::default().fg(Color::Rgb(100, 100, 100))),
        ];

        let paragraph = Paragraph::new(loading_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, chunks[1]);

        // Footer hint
        let footer = Paragraph::new("Press Esc to cancel")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(120, 120, 120)))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(footer, chunks[2]);

        // Tick the spinner
        self.spinner.tick();
    }

    fn draw_title(&self, f: &mut Frame, area: Rect) {
        let title = vec![
            Line::from("YouTube Chapter Splitter"),
            Line::from(""),
            Line::from("Download YouTube videos and split into MP3 tracks"),
        ];

        let paragraph = Paragraph::new(title)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    fn draw_menu(&self, f: &mut Frame, area: Rect) {
        let menu_width = 40;
        let menu_x = (area.width.saturating_sub(menu_width)) / 2;
        let menu_area = Rect {
            x: area.x + menu_x,
            y: area.y,
            width: menu_width.min(area.width),
            height: area.height,
        };

        let menu_text = vec![
            Line::from(""),
            Line::from("  [D] Download from URL"),
            Line::from(""),
            Line::from("  [S] Settings"),
            Line::from(""),
            Line::from("  [H] Help"),
            Line::from(""),
            Line::from(""),
            Line::from("  [Q] Quit"),
            Line::from(""),
        ];

        let paragraph = Paragraph::new(menu_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(100, 150, 200))),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, menu_area);
    }

    pub fn handle_key(&self, key: KeyEvent, _data: &mut ScreenData) -> ScreenResult {
        // Allow Esc to cancel loading
        if key.code == KeyCode::Esc && self.is_loading() {
            return ScreenResult::Continue; // Would need to actually cancel in real impl
        }

        // Don't allow navigation while loading
        if self.is_loading() {
            return ScreenResult::Continue;
        }

        match key.code {
            KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Enter => {
                ScreenResult::NavigateTo(Screen::Download)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => ScreenResult::NavigateTo(Screen::Settings),
            KeyCode::Char('h') | KeyCode::Char('H') | KeyCode::F(1) => {
                ScreenResult::NavigateTo(Screen::Help)
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => ScreenResult::Quit,
            _ => ScreenResult::Continue,
        }
    }
}

impl Default for WelcomeScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_welcome_screen_new() {
        let screen = WelcomeScreen::new();
        assert_eq!(screen.state(), &WelcomeState::Idle);
        assert!(!screen.is_loading());
    }

    #[test]
    fn test_welcome_screen_loading() {
        let mut screen = WelcomeScreen::new();
        screen.set_loading(WelcomeState::FetchingVideoInfo, "Fetching video info...");
        assert!(screen.is_loading());
        assert_eq!(screen.loading_message, "Fetching video info...");
    }

    #[test]
    fn test_welcome_screen_set_idle() {
        let mut screen = WelcomeScreen::new();
        screen.set_loading(WelcomeState::FetchingVideoInfo, "Loading...");
        screen.set_idle();
        assert!(!screen.is_loading());
    }
}
