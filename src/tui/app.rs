//! Main TUI application state and logic.

use crate::chapters::Chapter;
use crate::downloader::VideoInfo;
use crate::error::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::path::PathBuf;

/// Application modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Chapter selection mode
    ChapterSelection,
    /// Metadata editing mode
    MetadataEdit,
    /// Downloading mode
    Downloading,
    /// Splitting audio mode
    Splitting,
    /// Confirmation dialog
    Confirmation,
    /// Help screen
    Help,
}

/// Main application state
pub struct App {
    /// Current application mode
    pub mode: AppMode,
    
    /// Video information
    pub video_info: VideoInfo,
    
    /// List of chapters
    pub chapters: Vec<Chapter>,
    
    /// Selected chapters (true = selected)
    pub selected_chapters: Vec<bool>,
    
    /// Current cursor position in chapter list
    pub current_selection: usize,
    
    /// Artist name (editable)
    pub artist: String,
    
    /// Album name (editable)
    pub album: String,
    
    /// Output directory
    pub output_dir: PathBuf,
    
    /// Download progress (0.0 to 1.0)
    pub download_progress: f64,
    
    /// Split progress (0.0 to 1.0)
    pub split_progress: f64,
    
    /// Current track being processed
    pub current_track: Option<String>,
    
    /// Log messages
    pub logs: Vec<String>,
    
    /// Whether to quit the application
    pub should_quit: bool,
    
    /// Whether user confirmed to start
    pub confirmed: bool,
    
    /// Metadata edit field index (0 = artist, 1 = album, 2+ = track titles)
    pub edit_field_index: usize,
    
    /// Current edit buffer
    pub edit_buffer: String,
}

impl App {
    /// Create a new App instance
    pub fn new(
        video_info: VideoInfo,
        chapters: Vec<Chapter>,
        artist: String,
        album: String,
        output_dir: PathBuf,
    ) -> Self {
        let selected_chapters = vec![true; chapters.len()];
        
        Self {
            mode: AppMode::ChapterSelection,
            video_info,
            chapters,
            selected_chapters,
            current_selection: 0,
            artist,
            album,
            output_dir,
            download_progress: 0.0,
            split_progress: 0.0,
            current_track: None,
            logs: Vec::new(),
            should_quit: false,
            confirmed: false,
            edit_field_index: 0,
            edit_buffer: String::new(),
        }
    }
    
    /// Run the TUI application
    pub fn run(mut self) -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Main loop
        loop {
            terminal.draw(|f| crate::tui::ui::draw(f, &self))?;
            
            if self.should_quit {
                break;
            }
            
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key.code, key.modifiers);
                }
            }
        }
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        Ok(self)
    }
    
    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match self.mode {
            AppMode::ChapterSelection => self.handle_chapter_selection_keys(key, modifiers),
            AppMode::MetadataEdit => self.handle_metadata_edit_keys(key),
            AppMode::Confirmation => self.handle_confirmation_keys(key),
            AppMode::Help => self.handle_help_keys(key),
            _ => {}
        }
    }
    
    /// Handle keys in chapter selection mode
    fn handle_chapter_selection_keys(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_selection > 0 {
                    self.current_selection -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_selection < self.chapters.len().saturating_sub(1) {
                    self.current_selection += 1;
                }
            }
            KeyCode::Char(' ') => {
                if self.current_selection < self.selected_chapters.len() {
                    self.selected_chapters[self.current_selection] = 
                        !self.selected_chapters[self.current_selection];
                }
            }
            KeyCode::Char('a') => {
                // Select all
                self.selected_chapters.iter_mut().for_each(|s| *s = true);
            }
            KeyCode::Char('n') => {
                // Select none
                self.selected_chapters.iter_mut().for_each(|s| *s = false);
            }
            KeyCode::Char('i') => {
                // Invert selection
                self.selected_chapters.iter_mut().for_each(|s| *s = !*s);
            }
            KeyCode::Char('e') => {
                // Enter metadata edit mode
                self.mode = AppMode::MetadataEdit;
                self.edit_field_index = 0;
                self.edit_buffer = self.artist.clone();
            }
            KeyCode::Char('s') | KeyCode::Enter => {
                // Start download (show confirmation first)
                let selected_count = self.selected_chapters.iter().filter(|&&s| s).count();
                if selected_count > 0 {
                    self.mode = AppMode::Confirmation;
                } else {
                    self.add_log("⚠️  No chapters selected!".to_string());
                }
            }
            KeyCode::Char('?') | KeyCode::F(1) => {
                // Show help
                self.mode = AppMode::Help;
            }
            _ => {}
        }
    }
    
    /// Handle keys in metadata edit mode
    fn handle_metadata_edit_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                // Cancel editing
                self.mode = AppMode::ChapterSelection;
            }
            KeyCode::Enter => {
                // Save current field and move to next
                self.save_current_edit_field();
                self.edit_field_index += 1;
                
                if self.edit_field_index >= 2 + self.chapters.len() {
                    // Done editing
                    self.mode = AppMode::ChapterSelection;
                } else {
                    self.load_edit_field();
                }
            }
            KeyCode::Tab => {
                // Move to next field
                self.save_current_edit_field();
                self.edit_field_index = (self.edit_field_index + 1) % (2 + self.chapters.len());
                self.load_edit_field();
            }
            KeyCode::BackTab => {
                // Move to previous field
                self.save_current_edit_field();
                if self.edit_field_index == 0 {
                    self.edit_field_index = 1 + self.chapters.len();
                } else {
                    self.edit_field_index -= 1;
                }
                self.load_edit_field();
            }
            KeyCode::Backspace => {
                self.edit_buffer.pop();
            }
            KeyCode::Char(c) => {
                self.edit_buffer.push(c);
            }
            _ => {}
        }
    }
    
    /// Handle keys in confirmation mode
    fn handle_confirmation_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.confirmed = true;
                self.should_quit = true;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = AppMode::ChapterSelection;
            }
            _ => {}
        }
    }
    
    /// Handle keys in help mode
    fn handle_help_keys(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::F(1) => {
                self.mode = AppMode::ChapterSelection;
            }
            _ => {}
        }
    }
    
    /// Save the current edit field
    fn save_current_edit_field(&mut self) {
        match self.edit_field_index {
            0 => self.artist = self.edit_buffer.clone(),
            1 => self.album = self.edit_buffer.clone(),
            i if i >= 2 && i < 2 + self.chapters.len() => {
                let chapter_idx = i - 2;
                self.chapters[chapter_idx].title = self.edit_buffer.clone();
            }
            _ => {}
        }
    }
    
    /// Load the edit field into the buffer
    fn load_edit_field(&mut self) {
        self.edit_buffer = match self.edit_field_index {
            0 => self.artist.clone(),
            1 => self.album.clone(),
            i if i >= 2 && i < 2 + self.chapters.len() => {
                let chapter_idx = i - 2;
                self.chapters[chapter_idx].title.clone()
            }
            _ => String::new(),
        };
    }
    
    /// Add a log message
    pub fn add_log(&mut self, message: String) {
        self.logs.push(message);
        // Keep only last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
    
    /// Get selected chapters
    pub fn get_selected_chapters(&self) -> Vec<Chapter> {
        self.chapters
            .iter()
            .zip(self.selected_chapters.iter())
            .filter(|(_, &selected)| selected)
            .map(|(chapter, _)| chapter.clone())
            .collect()
    }
}
