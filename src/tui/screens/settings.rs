//! Settings screen for the TUI
//!
//! Allows users to configure application settings with inline editing.

use crate::config::{AutoInstallBehavior, Config, PlaylistBehavior};
use crate::tui::app::{Screen, ScreenData};
use crate::tui::components::input::TextInput;
use crate::tui::components::modal::{Modal, ModalState};
use crate::tui::screens::ScreenResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Setting item types
#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingType {
    Boolean,
    String,
    Integer,
    Enum(&'static [&'static str]),
    Path,
}

/// A single setting item
#[derive(Debug, Clone)]
struct SettingItem {
    key: String,
    name: String,
    description: String,
    setting_type: SettingType,
    value_string: String,
}

impl SettingItem {
    fn new(
        key: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        setting_type: SettingType,
        value_string: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            description: description.into(),
            setting_type,
            value_string: value_string.into(),
        }
    }
}

/// Settings screen state
#[derive(Debug, Clone, PartialEq)]
enum SettingsState {
    /// Browsing settings list
    Browsing,
    /// Editing a text value
    EditingText,
    /// Showing save confirmation (for future use)
    #[allow(dead_code)]
    ConfirmSave,
    /// Showing error message
    Error(String),
}

/// Settings screen
pub struct SettingsScreen {
    items: Vec<SettingItem>,
    selected: usize,
    offset: usize,
    state: SettingsState,
    text_input: TextInput,
    pending_changes: bool,
    config_snapshot: Option<Config>,
    modal: Option<Modal>,
}

impl SettingsScreen {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
            offset: 0,
            state: SettingsState::Browsing,
            text_input: TextInput::new(),
            pending_changes: false,
            config_snapshot: None,
            modal: None,
        }
    }

    /// Initialize settings from config
    pub fn load_config(&mut self, config: &Config) {
        // Store snapshot for detecting changes
        self.config_snapshot = Some(config.clone());

        self.items = vec![
            SettingItem::new(
                "download_cover",
                "Download album cover",
                "Automatically download thumbnail as cover art",
                SettingType::Boolean,
                config.download_cover.to_string(),
            ),
            SettingItem::new(
                "audio_quality",
                "Audio quality",
                "MP3 bitrate in kbps (128, 192, or 320)",
                SettingType::Enum(&["128", "192", "320"]),
                config.audio_quality.to_string(),
            ),
            SettingItem::new(
                "filename_format",
                "Filename format",
                "Track naming: %n=number, %t=title, %a=artist, %A=album",
                SettingType::String,
                config.filename_format.clone(),
            ),
            SettingItem::new(
                "directory_format",
                "Directory format",
                "Album folder naming: %a=artist, %A=album",
                SettingType::String,
                config.directory_format.clone(),
            ),
            SettingItem::new(
                "default_output_dir",
                "Output directory",
                "Default location for downloads (empty = system Music)",
                SettingType::Path,
                config.default_output_dir.clone().unwrap_or_default(),
            ),
            SettingItem::new(
                "overwrite_existing",
                "Overwrite files",
                "Replace existing files without prompting",
                SettingType::Boolean,
                config.overwrite_existing.to_string(),
            ),
            SettingItem::new(
                "create_playlist",
                "Create playlist",
                "Generate .m3u playlist file",
                SettingType::Boolean,
                config.create_playlist.to_string(),
            ),
            SettingItem::new(
                "max_retries",
                "Max retries",
                "Maximum download retry attempts",
                SettingType::Integer,
                config.max_retries.to_string(),
            ),
            SettingItem::new(
                "playlist_behavior",
                "Playlist behavior",
                "What to do when a playlist URL is detected",
                SettingType::Enum(&["Ask", "Video Only", "Playlist Only"]),
                format!("{:?}", config.playlist_behavior)
                    .replace("PlaylistBehavior::", "")
                    .replace("VideoOnly", "Video Only"),
            ),
            SettingItem::new(
                "cookies_from_browser",
                "Browser cookies",
                "Auto-extract cookies from browser (empty = disabled)",
                SettingType::String,
                config.cookies_from_browser.clone().unwrap_or_default(),
            ),
            SettingItem::new(
                "ytdlp_auto_update",
                "Auto-update yt-dlp",
                "Automatically update yt-dlp on download failure",
                SettingType::Boolean,
                config.ytdlp_auto_update.to_string(),
            ),
            SettingItem::new(
                "ytdlp_update_interval_days",
                "Update check interval",
                "Days between yt-dlp update checks (0 = always)",
                SettingType::Integer,
                config.ytdlp_update_interval_days.to_string(),
            ),
            SettingItem::new(
                "dependency_auto_install",
                "Auto-install dependencies",
                "Behavior when dependencies are missing",
                SettingType::Enum(&["Prompt", "Always", "Never"]),
                format!("{:?}", config.dependency_auto_install)
                    .replace("AutoInstallBehavior::", ""),
            ),
        ];

        self.selected = 0;
        self.offset = 0;
        self.pending_changes = false;
    }

    /// Get the currently selected setting
    fn selected_item(&self) -> Option<&SettingItem> {
        self.items.get(self.selected)
    }

    /// Get the currently selected setting mutably
    fn selected_item_mut(&mut self) -> Option<&mut SettingItem> {
        self.items.get_mut(self.selected)
    }

    /// Toggle a boolean setting
    fn toggle_boolean(&mut self) {
        if let Some(item) = self.selected_item_mut()
            && item.setting_type == SettingType::Boolean
        {
            let new_val = !item.value_string.parse::<bool>().unwrap_or(false);
            item.value_string = new_val.to_string();
            self.pending_changes = true;
        }
    }

    /// Cycle to next enum value
    fn cycle_enum(&mut self, forward: bool) {
        if let Some(item) = self.selected_item_mut()
            && let SettingType::Enum(options) = item.setting_type
        {
            let current = &item.value_string;
            let idx = options.iter().position(|&s| s == current).unwrap_or(0);
            let len = options.len();
            let new_idx = if forward {
                (idx + 1) % len
            } else {
                (idx + len - 1) % len // Wrap correctly when going backward from 0
            };
            item.value_string = options[new_idx].to_string();
            self.pending_changes = true;
        }
    }

    /// Start editing a text setting
    fn start_editing(&mut self) {
        if let Some(item) = self.selected_item() {
            match item.setting_type {
                SettingType::String | SettingType::Path | SettingType::Integer => {
                    self.text_input = TextInput::new()
                        .with_value(item.value_string.clone())
                        .with_title(&item.name)
                        .with_focused(true);
                    self.state = SettingsState::EditingText;
                }
                SettingType::Boolean => {
                    self.toggle_boolean();
                }
                SettingType::Enum(_) => {
                    self.cycle_enum(true);
                }
            }
        }
    }

    /// Apply all changes to config
    fn apply_changes(&self, config: &mut Config) -> Result<(), String> {
        for item in &self.items {
            match item.key.as_str() {
                "download_cover" => {
                    config.download_cover = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid boolean value".to_string())?;
                }
                "audio_quality" => {
                    config.audio_quality = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid audio quality".to_string())?;
                }
                "filename_format" => {
                    config.filename_format = item.value_string.clone();
                }
                "directory_format" => {
                    config.directory_format = item.value_string.clone();
                }
                "default_output_dir" => {
                    config.default_output_dir = if item.value_string.is_empty() {
                        None
                    } else {
                        Some(item.value_string.clone())
                    };
                }
                "overwrite_existing" => {
                    config.overwrite_existing = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid boolean value".to_string())?;
                }
                "create_playlist" => {
                    config.create_playlist = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid boolean value".to_string())?;
                }
                "max_retries" => {
                    config.max_retries = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid number".to_string())?;
                }
                "playlist_behavior" => {
                    config.playlist_behavior = match item.value_string.as_str() {
                        "Ask" => PlaylistBehavior::Ask,
                        "Video Only" => PlaylistBehavior::VideoOnly,
                        "Playlist Only" => PlaylistBehavior::PlaylistOnly,
                        _ => return Err("Invalid playlist behavior".to_string()),
                    };
                }
                "cookies_from_browser" => {
                    config.cookies_from_browser = if item.value_string.is_empty() {
                        None
                    } else {
                        Some(item.value_string.clone())
                    };
                }
                "ytdlp_auto_update" => {
                    config.ytdlp_auto_update = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid boolean value".to_string())?;
                }
                "ytdlp_update_interval_days" => {
                    config.ytdlp_update_interval_days = item
                        .value_string
                        .parse()
                        .map_err(|_| "Invalid number".to_string())?;
                }
                "dependency_auto_install" => {
                    config.dependency_auto_install = match item.value_string.as_str() {
                        "Prompt" => AutoInstallBehavior::Prompt,
                        "Always" => AutoInstallBehavior::Always,
                        "Never" => AutoInstallBehavior::Never,
                        _ => return Err("Invalid auto-install behavior".to_string()),
                    };
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Save configuration to file
    fn save_config(config: &Config) -> Result<(), String> {
        config
            .save()
            .map_err(|e| format!("Failed to save config: {}", e))
    }

    /// Move selection up
    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    fn select_next(&mut self) {
        if self.selected < self.items.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn draw(&mut self, f: &mut Frame, _data: &ScreenData, config: &Config) {
        let size = f.area();

        // Load config if not loaded
        if self.items.is_empty() {
            self.load_config(config);
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(4), // Footer
            ])
            .split(size);

        // Draw title
        self.draw_title(f, chunks[0]);

        // Draw content
        if self.state == SettingsState::EditingText {
            self.draw_editing(f, chunks[1]);
        } else if self.modal.is_some() {
            // Draw the list in background
            self.draw_list(f, chunks[1]);
            // Draw modal on top
            if let Some(ref mut modal) = self.modal {
                modal.draw(f);
            }
        } else {
            self.draw_list(f, chunks[1]);
        }

        // Draw footer
        self.draw_footer(f, chunks[2]);
    }

    fn draw_title(&self, f: &mut Frame, area: Rect) {
        let title_text = if self.pending_changes {
            "Settings (*)"
        } else {
            "Settings"
        };

        let title = Paragraph::new(title_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, area);
    }

    fn draw_list(&mut self, f: &mut Frame, area: Rect) {
        let inner_height = area.height.saturating_sub(2) as usize;

        // Adjust offset to keep selection in view
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + inner_height {
            self.offset = self.selected - inner_height + 1;
        }

        // Build list items
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .skip(self.offset)
            .take(inner_height)
            .map(|(i, item)| {
                let is_selected = i == self.selected;
                let value_style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(180, 180, 180))
                };

                let name_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let value_indicator = match item.setting_type {
                    SettingType::Boolean => {
                        if item.value_string == "true" {
                            "[✓]"
                        } else {
                            "[ ]"
                        }
                    }
                    SettingType::Enum(_) => "[~]",
                    _ => "",
                };

                ListItem::new(Line::from(vec![
                    Span::raw(format!(" {:2} ", i + 1)),
                    Span::styled(&item.name, name_style),
                    Span::raw(" "),
                    Span::styled(value_indicator, Style::default().fg(Color::Yellow)),
                    if !value_indicator.is_empty() {
                        Span::raw(" ")
                    } else {
                        Span::raw(": ")
                    },
                    Span::styled(format_value_display(&item.value_string, 30), value_style),
                ]))
            })
            .collect();

        let list = List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(100, 150, 200))),
        );

        let mut state = ListState::default();
        if self.selected >= self.offset {
            state.select(Some(self.selected - self.offset));
        }

        f.render_stateful_widget(list, area, &mut state);
    }

    fn draw_editing(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Input
                Constraint::Min(0),    // Info
            ])
            .split(area);

        // Draw text input
        self.text_input.draw(f, chunks[0]);

        // Draw editing info
        if let Some(item) = self.selected_item() {
            let info = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                    Span::styled(&item.description, Style::default().fg(Color::White)),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from("Editing instructions:"),
                Line::from("  • Type to edit the value"),
                Line::from("  • Enter: Save changes"),
                Line::from("  • Esc: Cancel without saving"),
            ];

            let paragraph = Paragraph::new(info).wrap(Wrap { trim: true });
            f.render_widget(paragraph, chunks[1]);
        }
    }

    fn draw_footer(&self, f: &mut Frame, area: Rect) {
        let hints = match self.state {
            SettingsState::Browsing => {
                vec![
                    "↑↓: Navigate".to_string(),
                    "Enter: Edit".to_string(),
                    "S: Save".to_string(),
                    "R: Reset".to_string(),
                    "Esc: Back".to_string(),
                ]
            }
            SettingsState::EditingText => {
                vec![
                    "Type to edit".to_string(),
                    "Enter: Save".to_string(),
                    "Esc: Cancel".to_string(),
                ]
            }
            _ => vec!["Esc: Close".to_string()],
        };

        let hint_text = hints.join(" | ");

        let paragraph = Paragraph::new(hint_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(150, 150, 150)))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(paragraph, area);
    }

    pub fn handle_key(
        &mut self,
        key: KeyEvent,
        _data: &mut ScreenData,
        config: &mut Config,
    ) -> ScreenResult {
        match self.state {
            SettingsState::EditingText => self.handle_editing_key(key, config),
            SettingsState::Error(_) => {
                // Any key closes error
                self.state = SettingsState::Browsing;
                self.modal = None;
                ScreenResult::Continue
            }
            _ => self.handle_browsing_key(key, config),
        }
    }

    fn handle_browsing_key(&mut self, key: KeyEvent, config: &mut Config) -> ScreenResult {
        // Handle modal first if present
        if let Some(ref mut modal) = self.modal {
            if modal.handle_key(key) {
                // Modal was closed
                let modal_state = modal.state().clone();
                self.modal = None;
                match modal_state {
                    ModalState::Confirmed => {
                        // Save and quit
                        if let Err(e) = Self::save_config(config) {
                            self.state = SettingsState::Error(e);
                            self.modal =
                                Some(Modal::error("Save Failed", "Could not save configuration"));
                            return ScreenResult::Continue;
                        }
                        self.pending_changes = false;
                        self.config_snapshot = Some(config.clone());
                        return ScreenResult::Continue;
                    }
                    ModalState::Cancelled => {
                        self.state = SettingsState::Browsing;
                    }
                    _ => {}
                }
            }
            return ScreenResult::Continue;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                ScreenResult::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                ScreenResult::Continue
            }
            KeyCode::Enter => {
                self.start_editing();
                ScreenResult::Continue
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                // Reset to defaults
                self.modal = Some(Modal::confirm(
                    "Reset Settings",
                    "Reset all settings to default values? This will discard your changes.",
                ));
                ScreenResult::Continue
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                if self.pending_changes {
                    // Apply changes and save
                    match self.apply_changes(config) {
                        Ok(()) => match Self::save_config(config) {
                            Ok(()) => {
                                self.pending_changes = false;
                                self.config_snapshot = Some(config.clone());
                                self.modal = Some(Modal::info(
                                    "Settings Saved",
                                    "Configuration has been saved successfully.",
                                ));
                            }
                            Err(e) => {
                                self.modal = Some(Modal::error("Save Failed", &e));
                            }
                        },
                        Err(e) => {
                            self.modal = Some(Modal::error("Invalid Setting", &e));
                        }
                    }
                } else {
                    self.modal = Some(Modal::info(
                        "No Changes",
                        "No changes have been made to save.",
                    ));
                }
                ScreenResult::Continue
            }
            KeyCode::Esc => {
                if self.pending_changes {
                    self.modal = Some(Modal::confirm(
                        "Unsaved Changes",
                        "You have unsaved changes. Exit anyway?",
                    ));
                    ScreenResult::Continue
                } else {
                    ScreenResult::NavigateTo(Screen::Welcome)
                }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if self.pending_changes {
                    self.modal = Some(Modal::confirm(
                        "Unsaved Changes",
                        "You have unsaved changes. Quit anyway?",
                    ));
                    ScreenResult::Continue
                } else {
                    ScreenResult::Quit
                }
            }
            KeyCode::Left => {
                // Cycle enum backward
                if let Some(item) = self.selected_item() {
                    if matches!(item.setting_type, SettingType::Enum(_)) {
                        self.cycle_enum(false);
                        ScreenResult::Continue
                    } else {
                        ScreenResult::Continue
                    }
                } else {
                    ScreenResult::Continue
                }
            }
            KeyCode::Right => {
                // Cycle enum forward
                if let Some(item) = self.selected_item() {
                    if matches!(item.setting_type, SettingType::Enum(_)) {
                        self.cycle_enum(true);
                        ScreenResult::Continue
                    } else {
                        ScreenResult::Continue
                    }
                } else {
                    ScreenResult::Continue
                }
            }
            _ => ScreenResult::Continue,
        }
    }

    fn handle_editing_key(&mut self, key: KeyEvent, _config: &mut Config) -> ScreenResult {
        if self.text_input.handle_key_event(key) {
            // Input handled
            ScreenResult::Continue
        } else {
            match key.code {
                KeyCode::Enter => {
                    // Save the edited value - clone before borrowing
                    let new_value = self.text_input.value.clone();
                    if let Some(item) = self.selected_item_mut() {
                        item.value_string = new_value;
                        self.pending_changes = true;
                    }
                    self.state = SettingsState::Browsing;
                    ScreenResult::Continue
                }
                KeyCode::Esc => {
                    // Cancel editing
                    self.state = SettingsState::Browsing;
                    ScreenResult::Continue
                }
                _ => ScreenResult::Continue,
            }
        }
    }
}

fn format_value_display(value: &str, max_len: usize) -> String {
    if value.len() > max_len {
        format!("{}...", &value[..max_len.saturating_sub(3)])
    } else if value.is_empty() {
        "(empty)".to_string()
    } else {
        value.to_string()
    }
}

impl Default for SettingsScreen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_screen_new() {
        let screen = SettingsScreen::new();
        assert!(screen.items.is_empty());
        assert_eq!(screen.selected, 0);
        assert!(!screen.pending_changes);
    }

    #[test]
    fn test_select_navigation() {
        let mut screen = SettingsScreen::new();
        screen.items = vec![
            SettingItem::new("1", "One", "", SettingType::Boolean, "false"),
            SettingItem::new("2", "Two", "", SettingType::Boolean, "true"),
        ];

        screen.select_next();
        assert_eq!(screen.selected, 1);

        screen.select_previous();
        assert_eq!(screen.selected, 0);
    }

    #[test]
    fn test_toggle_boolean() {
        let mut screen = SettingsScreen::new();
        screen.selected = 0; // Set selection before toggling
        screen.items = vec![SettingItem::new(
            "1",
            "One",
            "",
            SettingType::Boolean,
            "false",
        )];

        screen.toggle_boolean();
        assert_eq!(screen.items[0].value_string, "true");
        assert!(screen.pending_changes);

        screen.toggle_boolean();
        assert_eq!(screen.items[0].value_string, "false");
    }

    #[test]
    fn test_cycle_enum() {
        let mut screen = SettingsScreen::new();
        screen.selected = 0; // Set selection before cycling
        screen.items = vec![SettingItem::new(
            "1",
            "One",
            "",
            SettingType::Enum(&["128", "192", "320"]),
            "128",
        )];

        screen.cycle_enum(true);
        assert_eq!(screen.items[0].value_string, "192");

        screen.cycle_enum(true);
        assert_eq!(screen.items[0].value_string, "320");

        screen.cycle_enum(true);
        assert_eq!(screen.items[0].value_string, "128"); // Wrapped around

        screen.cycle_enum(false);
        assert_eq!(screen.items[0].value_string, "320"); // Wrapped back
    }

    #[test]
    fn test_format_value_display() {
        assert_eq!(format_value_display("hello", 10), "hello");
        assert_eq!(format_value_display("", 10), "(empty)");
        assert_eq!(
            format_value_display("this is a very long string", 15),
            "this is a ve..."
        );
    }
}
