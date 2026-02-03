//! TUI Application state and main loop
//!
//! This module contains the main TUI application logic using ratatui.

use crate::config::Config;
use crate::dependency::DependencyState;
use crate::error::Result;
use crate::tui::download_manager::DownloadManager;
use crate::tui::layout::TerminalCapabilities;
use crate::tui::screens::{
    HelpScreen, PlaylistScreen, ProgressScreen, ScreenResult, SettingsScreen,
};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io;
use std::time::{Duration, Instant};

/// TUI screen states
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Welcome,
    Download,
    Playlist,
    Progress,
    Settings,
    Help,
    Summary,
}

/// Main application state
pub struct App {
    pub current_screen: Screen,
    pub should_quit: bool,
    pub config: Config,
    pub screen_data: ScreenData,
    /// Detected terminal capabilities
    pub capabilities: TerminalCapabilities,
    /// Persistent welcome screen (preserves loading state)
    pub welcome_screen: crate::tui::screens::welcome::WelcomeScreen,
    /// Persistent playlist screen (preserves state when navigating)
    pub playlist_screen: PlaylistScreen,
    /// Persistent progress screen (shows download status)
    pub progress_screen: ProgressScreen,
    /// Persistent settings screen (preserves edits)
    pub settings_screen: SettingsScreen,
    /// Persistent summary screen (preserves action selection)
    pub summary_screen: crate::tui::screens::summary::SummaryScreen,
    /// Persistent download screen (preserves metadata state)
    pub download_screen: crate::tui::screens::download::DownloadScreen,
    /// Download manager for async download operations
    pub download_manager: DownloadManager,
    /// Track rapid Esc presses for "Esc Esc Esc = welcome" behavior
    esc_press_count: u8,
    last_esc_time: Option<Instant>,
    /// Flag indicating terminal was resized
    pub resized: bool,
}

/// Data shared between screens
#[derive(Default)]
pub struct ScreenData {
    pub input_url: String,
    pub input_artist: String,
    pub input_album: String,
    pub download_status: String,
    pub error_message: Option<String>,
    pub last_download_result: Option<DownloadResult>,
    /// Dependency check result (None = not checked yet)
    pub dependency_status: Option<DependencyStatus>,
    /// True if artist/album were auto-detected from video metadata
    pub metadata_autodetected: bool,
}

/// Status of dependency check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStatus {
    /// All dependencies present
    Ok,
    /// Missing dependencies - downloads blocked
    MissingDependencies { ytdlp: bool, ffmpeg: bool },
}

#[derive(Default, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub tracks_count: usize,
    pub output_path: String,
    pub error: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load().unwrap_or_default();
        let capabilities = TerminalCapabilities::detect();

        // Check dependencies on startup
        let dep_state = DependencyState::check_all();
        let dependency_status = if dep_state.all_present() {
            Some(DependencyStatus::Ok)
        } else {
            Some(DependencyStatus::MissingDependencies {
                ytdlp: !dep_state.ytdlp.installed,
                ffmpeg: !dep_state.ffmpeg.installed,
            })
        };

        let mut screen_data = ScreenData::default();
        screen_data.dependency_status = dependency_status;

        Ok(Self {
            current_screen: Screen::Welcome,
            should_quit: false,
            config: config.clone(),
            screen_data,
            capabilities,
            welcome_screen: crate::tui::screens::welcome::WelcomeScreen::new(),
            playlist_screen: PlaylistScreen::new(),
            progress_screen: ProgressScreen::new(),
            settings_screen: SettingsScreen::new(),
            summary_screen: crate::tui::screens::summary::SummaryScreen::new(),
            download_screen: crate::tui::screens::download::DownloadScreen::new(),
            download_manager: DownloadManager::new(config),
            esc_press_count: 0,
            last_esc_time: None,
            resized: false,
        })
    }

    /// Reset Esc press tracking (call after screen navigation)
    fn reset_esc_tracking(&mut self) {
        self.esc_press_count = 0;
        self.last_esc_time = None;
    }

    /// Handle rapid Esc presses for "Esc Esc Esc = welcome" behavior
    fn handle_esc_press(&mut self) -> bool {
        let now = Instant::now();
        let rapid_threshold = Duration::from_millis(800);

        // Check if this Esc is rapid (within threshold of last Esc)
        let is_rapid = self
            .last_esc_time
            .map(|last| now.duration_since(last) < rapid_threshold)
            .unwrap_or(false);

        if is_rapid {
            self.esc_press_count += 1;
        } else {
            self.esc_press_count = 1;
        }
        self.last_esc_time = Some(now);

        // Three rapid Esc presses return to welcome
        if self.esc_press_count >= 3 {
            self.reset_esc_tracking();
            self.current_screen = Screen::Welcome;
            return true; // Handled as rapid-esc
        }

        false // Normal Esc behavior
    }

    /// Main run loop for the TUI
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run the main loop
        let res = self.run_main_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        // Propagate any errors
        res?;

        Ok(())
    }

    fn run_main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<()> {
        let tick_rate = Duration::from_millis(250);

        loop {
            // Draw the current screen
            terminal.draw(|f| self.draw(f))?;

            // Handle timeout for polling
            if crossterm::event::poll(tick_rate)? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key),
                    Event::Resize(_, _) => {
                        self.resized = true;
                        // Re-detect terminal capabilities on resize
                        self.capabilities = TerminalCapabilities::detect();
                    }
                    // Ignore other events silently (mouse, paste, focus, etc.)
                    _ => {}
                }
            }

            // Clear resize flag after handling
            if self.resized {
                self.resized = false;
            }

            // Update any pending downloads
            self.update();

            if self.should_quit {
                return Ok(());
            }
        }
    }

    fn draw(&mut self, f: &mut Frame) {
        let size = f.area();

        // Ensure minimum terminal size
        if size.width < 60 || size.height < 20 {
            self.draw_too_small(f);
            return;
        }

        match self.current_screen {
            Screen::Welcome => {
                self.welcome_screen.draw(f, &self.screen_data, &self.config);
            }
            Screen::Download => {
                self.download_screen.draw(f, &self.screen_data, &self.config);
            }
            Screen::Playlist => {
                self.playlist_screen
                    .draw(f, &self.screen_data, &self.config);
            }
            Screen::Progress => {
                // Update progress from download manager before drawing
                self.progress_screen.update_from_manager(
                    self.download_manager.overall_percent(),
                    self.download_manager.completed_count(),
                    self.download_manager.tasks().len(),
                    self.download_manager.failed_count(),
                );
                self.progress_screen
                    .draw(f, &self.screen_data, &self.config);
            }
            Screen::Settings => {
                self.settings_screen
                    .draw(f, &self.screen_data, &self.config);
            }
            Screen::Help => {
                let mut screen = HelpScreen::new();
                screen.draw(f, &self.screen_data, &self.config);
            }
            Screen::Summary => {
                self.summary_screen.draw(f, &self.screen_data, &self.config);
            }
        }

        // Draw global footer with key hints
        self.draw_footer(f);
    }

    fn draw_footer(&self, f: &mut Frame) {
        let size = f.area();
        let footer_height = 3;

        let footer = Rect {
            x: 0,
            y: size.height.saturating_sub(footer_height),
            width: size.width,
            height: footer_height,
        };

        let hints = match self.current_screen {
            Screen::Welcome => vec!["Enter: Start", "S: Settings", "H: Help", "Q: Quit"],
            Screen::Download => vec!["Enter: Download", "Esc: Back", "Q: Quit"],
            Screen::Playlist => vec![
                "↑↓: Navigate",
                "Space: Toggle",
                "A: All",
                "D: None",
                "Esc: Back",
            ],
            Screen::Progress => vec!["Esc: Cancel (when done)", "Enter: Results", "Q: Quit"],
            Screen::Settings => vec!["↑↓: Navigate", "Enter: Edit", "Esc: Back"],
            Screen::Help => vec!["↑↓: Scroll", "Esc: Back", "Q: Quit"],
            Screen::Summary => vec!["Enter: Continue", "Q: Quit"],
        };

        let hint_text = hints.join(" | ");
        let paragraph = Paragraph::new(hint_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(100, 100, 100)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded),
            );

        f.render_widget(paragraph, footer);
    }

    fn draw_too_small(&self, f: &mut Frame) {
        let area = f.area();
        let paragraph = Paragraph::new(vec![
            Line::from(Span::styled(
                "Terminal too small",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Minimum size: 60x20"),
            Line::from(""),
            Line::from(format!("Current: {}x{}", area.width, area.height)),
            Line::from(""),
            Line::from("Press Q to quit"),
        ])
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }

    fn handle_key_event(&mut self, key: KeyEvent) {
        // Handle Ctrl+C for quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        // Track Esc presses for rapid-Esc-to-welcome behavior
        let is_rapid_esc = key.code == KeyCode::Esc && self.handle_esc_press();
        if is_rapid_esc {
            return; // Already handled by rapid-Esc
        }

        // Reset Esc tracking on any other key
        if key.code != KeyCode::Esc {
            self.reset_esc_tracking();
        }

        let previous_screen = self.current_screen.clone();

        // Route key event to current screen
        let result = match self.current_screen {
            Screen::Welcome => self.welcome_screen.handle_key(key, &mut self.screen_data),
            Screen::Download => {
                self.download_screen
                    .handle_key(key, &mut self.screen_data, &mut self.playlist_screen)
            }
            Screen::Playlist => self.playlist_screen.handle_key(key, &mut self.screen_data),
            Screen::Progress => self.progress_screen.handle_key(key, &mut self.screen_data),
            Screen::Settings => {
                self.settings_screen
                    .handle_key(key, &mut self.screen_data, &mut self.config)
            }
            Screen::Help => {
                let mut screen = HelpScreen::new();
                screen.handle_key(key, &mut self.screen_data)
            }
            Screen::Summary => self.summary_screen.handle_key(key, &mut self.screen_data),
        };

        // Handle screen result
        match result {
            ScreenResult::Continue => {}
            ScreenResult::NavigateTo(screen) => {
                self.current_screen = screen;
                self.reset_esc_tracking();
            }
            ScreenResult::Quit => {
                self.should_quit = true;
            }
        }

        // Special handling for specific screen transitions
        if previous_screen != self.current_screen {
            match (&previous_screen, &self.current_screen) {
                (Screen::Download, Screen::Welcome) => {
                    // Reset metadata auto-detection state when returning to welcome
                    self.screen_data.metadata_autodetected = false;
                    self.download_screen.reset();
                }
                (Screen::Download, Screen::Playlist) => {
                    self.playlist_screen
                        .load_from_url(&self.screen_data.input_url, &self.config);
                }
                (Screen::Download, Screen::Progress) => {
                    // Check dependencies before allowing download
                    if let Some(DependencyStatus::MissingDependencies { .. }) =
                        self.screen_data.dependency_status
                    {
                        // Show error and don't proceed
                        self.screen_data.error_message = Some(
                            "Missing dependencies. Please install yt-dlp and ffmpeg.".to_string(),
                        );
                        self.current_screen = Screen::Download;
                        return;
                    }

                    let url = self.screen_data.input_url.trim().to_string();
                    if !url.is_empty() {
                        let artist = if self.screen_data.input_artist.is_empty() {
                            None
                        } else {
                            Some(self.screen_data.input_artist.clone())
                        };
                        let album = if self.screen_data.input_album.is_empty() {
                            None
                        } else {
                            Some(self.screen_data.input_album.clone())
                        };
                        self.download_manager.add_url(url, artist, album);
                        self.download_manager.start();
                    }
                }
                (Screen::Playlist, Screen::Progress) => {
                    // Check dependencies before allowing download
                    if let Some(DependencyStatus::MissingDependencies { .. }) =
                        self.screen_data.dependency_status
                    {
                        // Show error and don't proceed
                        self.screen_data.error_message = Some(
                            "Missing dependencies. Please install yt-dlp and ffmpeg.".to_string(),
                        );
                        self.current_screen = Screen::Playlist;
                        return;
                    }

                    let urls: Vec<String> = self
                        .screen_data
                        .input_url
                        .split('\n')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    if !urls.is_empty() {
                        self.download_manager.add_playlist_urls(urls);
                        self.download_manager.start();
                    }
                }
                _ => {}
            }
        }
    }

    fn update(&mut self) {
        // Process downloads if the manager is active
        if self.download_manager.is_active() {
            if let Err(e) = self.download_manager.process_next() {
                // Store error in screen data for TUI display
                self.screen_data.error_message = Some(format!("Download error: {}", e));
            }

            // Check if all downloads are complete
            if self.download_manager.pending_count() == 0 && !self.download_manager.is_active() {
                // Store final download result in screen data
                let tasks = self.download_manager.tasks();
                if !tasks.is_empty() {
                    let last_task = &tasks[tasks.len() - 1];
                    if let Some(ref result) = last_task.result {
                        self.screen_data.last_download_result =
                            Some(crate::tui::app::DownloadResult {
                                success: result.success,
                                tracks_count: result.tracks_count,
                                output_path: result.output_path.clone(),
                                error: result.error.clone(),
                            });
                    }
                }
                // All downloads done - navigate to summary
                self.current_screen = Screen::Summary;
            }
        }
    }
}
