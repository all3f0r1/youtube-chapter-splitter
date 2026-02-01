//! Persistent configuration management module
//!
//! This module handles application configuration stored in a TOML file.

use crate::error::{Result, YtcsError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Playlist detection behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlaylistBehavior {
    /// Ask the user what they want to do
    Ask,
    /// Always download only the single video (default for v1.0)
    #[default]
    VideoOnly,
    /// Always download the entire playlist
    PlaylistOnly,
}

/// TUI progress display mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProgressMode {
    /// Show only percentage bar
    Minimal,
    /// Show bar, percentage, speed, and ETA
    Detailed,
    /// Automatically choose based on terminal size
    #[default]
    Auto,
}

/// Automatic dependency installation behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AutoInstallBehavior {
    /// Always prompt before installing
    #[default]
    Prompt,
    /// Install automatically without asking
    Always,
    /// Never auto-install (manual only)
    Never,
}

/// TUI keyboard key bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    /// Key to quit the application
    #[serde(default = "default_key_quit")]
    pub quit: String,

    /// Key to show help screen
    #[serde(default = "default_key_help")]
    pub help: String,

    /// Key to confirm selection
    #[serde(default = "default_key_confirm")]
    pub confirm: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: "q".to_string(),
            help: "?".to_string(),
            confirm: "Enter".to_string(),
        }
    }
}

// Default functions for serde
fn default_key_quit() -> String {
    "q".to_string()
}

fn default_key_help() -> String {
    "?".to_string()
}

fn default_key_confirm() -> String {
    "Enter".to_string()
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default output directory (None = system Music directory)
    #[serde(default)]
    pub default_output_dir: Option<String>,

    /// Download album cover art
    #[serde(default = "default_download_cover")]
    pub download_cover: bool,

    /// Track filename format
    /// Available placeholders:
    /// - %n: track number (01, 02, etc.)
    /// - %t: track title
    /// - %a: artist
    /// - %A: album
    #[serde(default = "default_filename_format")]
    pub filename_format: String,

    /// Directory name format
    /// Available placeholders:
    /// - %a: artist
    /// - %A: album
    #[serde(default = "default_directory_format")]
    pub directory_format: String,

    /// Audio quality in kbps (128, 192, or 320)
    #[serde(default = "default_audio_quality")]
    pub audio_quality: u32,

    /// Overwrite existing files
    #[serde(default)]
    pub overwrite_existing: bool,

    /// Maximum retry attempts for downloads
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Create a playlist file (.m3u)
    #[serde(default)]
    pub create_playlist: bool,

    /// Playlist detection behavior (default: VideoOnly for v1.0)
    #[serde(default)]
    pub playlist_behavior: PlaylistBehavior,

    /// Browser to extract cookies from automatically
    /// Options: "chrome", "firefox", "safari", "edge", "chromium", "brave", "opera", "vivaldi"
    /// If None, uses cookies.txt file if it exists
    #[serde(default)]
    pub cookies_from_browser: Option<String>,

    /// Download timeout in seconds (0 = no timeout)
    #[serde(default = "default_download_timeout")]
    pub download_timeout: u64,

    // === TUI Settings (v1.0) ===
    /// TUI progress display mode
    #[serde(default = "default_tui_progress_mode")]
    pub tui_progress_mode: ProgressMode,

    /// TUI keyboard key bindings
    #[serde(default)]
    pub tui_key_bindings: KeyBindings,

    /// Automatic dependency installation behavior
    #[serde(default = "default_dependency_auto_install")]
    pub dependency_auto_install: AutoInstallBehavior,

    /// Auto-update yt-dlp on download failure
    #[serde(default = "default_ytdlp_auto_update")]
    pub ytdlp_auto_update: bool,

    /// Minimum days between auto-update attempts (0 = always check)
    #[serde(default = "default_ytdlp_update_interval")]
    pub ytdlp_update_interval_days: u64,
}

// Default value functions for serde
fn default_download_cover() -> bool {
    true
}

fn default_filename_format() -> String {
    "%n - %t".to_string()
}

fn default_directory_format() -> String {
    "%a - %A".to_string()
}

fn default_audio_quality() -> u32 {
    192
}

fn default_max_retries() -> u32 {
    3
}

fn default_download_timeout() -> u64 {
    300 // 5 minutes default
}

fn default_tui_progress_mode() -> ProgressMode {
    ProgressMode::Auto
}

fn default_dependency_auto_install() -> AutoInstallBehavior {
    AutoInstallBehavior::Prompt
}

fn default_ytdlp_auto_update() -> bool {
    true
}

fn default_ytdlp_update_interval() -> u64 {
    1 // Check for updates daily at most
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_output_dir: None,
            download_cover: true,
            filename_format: "%n - %t".to_string(),
            directory_format: "%a - %A".to_string(),
            audio_quality: 192,
            overwrite_existing: false,
            max_retries: 3,
            create_playlist: false,
            playlist_behavior: PlaylistBehavior::VideoOnly, // Changed from Ask for v1.0
            cookies_from_browser: None,
            download_timeout: 300,
            tui_progress_mode: ProgressMode::Auto,
            tui_key_bindings: KeyBindings::default(),
            dependency_auto_install: AutoInstallBehavior::Prompt,
            ytdlp_auto_update: true,
            ytdlp_update_interval_days: 1,
        }
    }
}

impl Config {
    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| YtcsError::ConfigError("Could not find config directory".to_string()))?;

        let ytcs_config_dir = config_dir.join("ytcs");
        fs::create_dir_all(&ytcs_config_dir)?;

        Ok(ytcs_config_dir.join("config.toml"))
    }

    /// Load configuration from file
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Create default configuration
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| YtcsError::ConfigError(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| YtcsError::ConfigError(format!("Failed to serialize config: {}", e)))?;

        fs::write(&config_path, content)?;
        Ok(())
    }

    /// Get the default output directory
    pub fn get_output_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.default_output_dir {
            PathBuf::from(shellexpand::tilde(dir).to_string())
        } else {
            // Fallback: system Music directory or home
            if let Some(music_dir) = dirs::audio_dir() {
                music_dir
            } else {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
            }
        }
    }

    /// Format filename according to template
    pub fn format_filename(
        &self,
        track_number: usize,
        title: &str,
        artist: &str,
        album: &str,
    ) -> String {
        self.filename_format
            .replace("%n", &format!("{:02}", track_number))
            .replace("%t", title)
            .replace("%a", artist)
            .replace("%A", album)
    }

    /// Format directory name according to template
    pub fn format_directory(&self, artist: &str, album: &str) -> String {
        self.directory_format
            .replace("%a", artist)
            .replace("%A", album)
    }
}

/// Display current configuration
pub fn show_config() -> Result<()> {
    let config = Config::load()?;
    let config_path = Config::config_path()?;

    println!("Configuration file: {}", config_path.display());
    println!();
    println!("Current settings:");
    println!(
        "  default_output_dir     = {:?}",
        config
            .default_output_dir
            .unwrap_or_else(|| "Music directory (default)".to_string())
    );
    println!("  download_cover        = {}", config.download_cover);
    println!("  filename_format       = \"{}\"", config.filename_format);
    println!("  directory_format      = \"{}\"", config.directory_format);
    println!("  audio_quality         = {} kbps", config.audio_quality);
    println!("  overwrite_existing    = {}", config.overwrite_existing);
    println!("  max_retries           = {}", config.max_retries);
    println!("  create_playlist       = {}", config.create_playlist);
    println!("  playlist_behavior     = {:?}", config.playlist_behavior);
    println!(
        "  cookies_from_browser  = {:?}",
        config.cookies_from_browser.as_deref().unwrap_or("None")
    );
    println!();
    println!("TUI Settings:");
    println!("  tui_progress_mode     = {:?}", config.tui_progress_mode);
    println!(
        "  tui_key_bindings      = quit:'{}' help:'{}' confirm:'{}'",
        config.tui_key_bindings.quit, config.tui_key_bindings.help, config.tui_key_bindings.confirm
    );
    println!(
        "  dependency_auto_install = {:?}",
        config.dependency_auto_install
    );
    println!("  ytdlp_auto_update        = {}", config.ytdlp_auto_update);
    println!(
        "  ytdlp_update_interval_days = {}",
        config.ytdlp_update_interval_days
    );
    println!();
    println!("Available placeholders:");
    println!("  Filename: %n (track number), %t (title), %a (artist), %A (album)");
    println!("  Directory: %a (artist), %A (album)");

    Ok(())
}

/// Set a configuration value
pub fn set_config(key: &str, value: &str) -> Result<()> {
    let mut config = Config::load()?;

    match key {
        "default_output_dir" => {
            config.default_output_dir = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            println!("✓ Set default_output_dir to: {}", value);
        }
        "download_cover" => {
            config.download_cover = value.parse::<bool>().map_err(|_| {
                YtcsError::ConfigError("Value must be 'true' or 'false'".to_string())
            })?;
            println!("✓ Set download_cover to: {}", config.download_cover);
        }
        "filename_format" => {
            config.filename_format = value.to_string();
            println!("✓ Set filename_format to: \"{}\"", value);
        }
        "directory_format" => {
            config.directory_format = value.to_string();
            println!("✓ Set directory_format to: \"{}\"", value);
        }
        "audio_quality" => {
            let quality = value.parse::<u32>().map_err(|_| {
                YtcsError::ConfigError("Value must be a number (128, 192, or 320)".to_string())
            })?;
            if quality != 128 && quality != 192 && quality != 320 {
                return Err(YtcsError::ConfigError(
                    "Audio quality must be 128, 192, or 320 kbps".to_string(),
                ));
            }
            config.audio_quality = quality;
            println!("✓ Set audio_quality to: {} kbps", quality);
        }
        "overwrite_existing" => {
            config.overwrite_existing = value.parse::<bool>().map_err(|_| {
                YtcsError::ConfigError("Value must be 'true' or 'false'".to_string())
            })?;
            println!("✓ Set overwrite_existing to: {}", config.overwrite_existing);
        }
        "max_retries" => {
            config.max_retries = value.parse::<u32>().map_err(|_| {
                YtcsError::ConfigError("Value must be a positive number".to_string())
            })?;
            println!("✓ Set max_retries to: {}", config.max_retries);
        }
        "create_playlist" => {
            config.create_playlist = value.parse::<bool>().map_err(|_| {
                YtcsError::ConfigError("Value must be 'true' or 'false'".to_string())
            })?;
            println!("✓ Set create_playlist to: {}", config.create_playlist);
        }
        "playlist_behavior" => {
            let behavior = match value {
                "ask" => PlaylistBehavior::Ask,
                "video_only" => PlaylistBehavior::VideoOnly,
                "playlist_only" => PlaylistBehavior::PlaylistOnly,
                _ => {
                    return Err(YtcsError::ConfigError(
                        "Value must be 'ask', 'video_only', or 'playlist_only'".to_string(),
                    ));
                }
            };
            config.playlist_behavior = behavior;
            println!("✓ Set playlist_behavior to: {:?}", config.playlist_behavior);
        }
        "cookies_from_browser" => {
            let valid_browsers = [
                "chrome", "firefox", "safari", "edge", "chromium", "brave", "opera", "vivaldi",
            ];
            if value.is_empty() {
                config.cookies_from_browser = None;
                println!("✓ Disabled cookies_from_browser");
            } else if valid_browsers.contains(&value.to_lowercase().as_str()) {
                config.cookies_from_browser = Some(value.to_lowercase());
                println!("✓ Set cookies_from_browser to: {}", value);
            } else {
                return Err(YtcsError::ConfigError(format!(
                    "Invalid browser '{}'. Valid options: {}",
                    value,
                    valid_browsers.join(", ")
                )));
            }
        }
        "tui_progress_mode" => {
            let mode = match value {
                "minimal" => ProgressMode::Minimal,
                "detailed" => ProgressMode::Detailed,
                "auto" => ProgressMode::Auto,
                _ => {
                    return Err(YtcsError::ConfigError(
                        "Value must be 'minimal', 'detailed', or 'auto'".to_string(),
                    ));
                }
            };
            println!("✓ Set tui_progress_mode to: {:?}", mode);
            config.tui_progress_mode = mode;
        }
        "dependency_auto_install" => {
            let behavior = match value {
                "prompt" => AutoInstallBehavior::Prompt,
                "always" => AutoInstallBehavior::Always,
                "never" => AutoInstallBehavior::Never,
                _ => {
                    return Err(YtcsError::ConfigError(
                        "Value must be 'prompt', 'always', or 'never'".to_string(),
                    ));
                }
            };
            println!("✓ Set dependency_auto_install to: {:?}", behavior);
            config.dependency_auto_install = behavior;
        }
        "ytdlp_auto_update" => {
            config.ytdlp_auto_update = value.parse::<bool>().map_err(|_| {
                YtcsError::ConfigError("Value must be 'true' or 'false'".to_string())
            })?;
            println!("✓ Set ytdlp_auto_update to: {}", config.ytdlp_auto_update);
        }
        "ytdlp_update_interval_days" => {
            config.ytdlp_update_interval_days = value.parse::<u64>().map_err(|_| {
                YtcsError::ConfigError("Value must be a positive number (days)".to_string())
            })?;
            println!(
                "✓ Set ytdlp_update_interval_days to: {}",
                config.ytdlp_update_interval_days
            );
        }
        _ => {
            return Err(YtcsError::ConfigError(format!(
                "Unknown config key: {}",
                key
            )));
        }
    }

    config.save()?;
    println!(
        "✓ Configuration saved to: {}",
        Config::config_path()?.display()
    );

    Ok(())
}

/// Reset configuration to defaults
pub fn reset_config() -> Result<()> {
    let config = Config::default();
    config.save()?;
    println!("✓ Configuration reset to defaults");
    println!(
        "✓ Configuration saved to: {}",
        Config::config_path()?.display()
    );
    Ok(())
}
