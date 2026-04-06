//! Persistent configuration management module
//!
//! This module handles application configuration stored in a TOML file.

use crate::error::{Result, YtcsError};
use crate::utils;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Output container / codec for yt-dlp extraction and per-chapter ffmpeg encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    #[default]
    Mp3,
    Opus,
    M4a,
}

impl AudioFormat {
    pub fn extension(self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Opus => "opus",
            AudioFormat::M4a => "m4a",
        }
    }

    pub fn yt_dlp_name(self) -> &'static str {
        self.extension()
    }
}

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

    /// Output format for downloaded audio and split tracks
    #[serde(default)]
    pub audio_format: AudioFormat,

    /// Overwrite existing files
    #[serde(default)]
    pub overwrite_existing: bool,

    /// Maximum retry attempts for downloads
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Create a playlist file (.m3u)
    #[serde(default)]
    pub create_playlist: bool,

    /// Adjust chapter boundaries using silence detection (extra ffmpeg pass)
    #[serde(default)]
    pub refine_chapters: bool,

    /// Search window (seconds, ±) for silence refinement around chapter edges
    #[serde(default = "default_refine_silence_window")]
    pub refine_silence_window: f64,

    /// Noise threshold in dB for silence refinement
    #[serde(default = "default_refine_noise_db")]
    pub refine_noise_db: f64,

    /// Minimum silence duration (seconds) for refinement pass
    #[serde(default = "default_refine_min_silence")]
    pub refine_min_silence: f64,

    /// Prefix album folder with `01-`, `02-`, … when processing multiple playlist entries
    #[serde(default)]
    pub playlist_prefix_index: bool,

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

fn default_refine_silence_window() -> f64 {
    5.0
}

fn default_refine_noise_db() -> f64 {
    -35.0
}

fn default_refine_min_silence() -> f64 {
    1.5
}

fn default_max_retries() -> u32 {
    3
}

fn default_download_timeout() -> u64 {
    300 // 5 minutes default
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
            audio_format: AudioFormat::Mp3,
            overwrite_existing: false,
            max_retries: 3,
            create_playlist: false,
            refine_chapters: false,
            refine_silence_window: 5.0,
            refine_noise_db: -35.0,
            refine_min_silence: 1.5,
            playlist_prefix_index: false,
            playlist_behavior: PlaylistBehavior::VideoOnly, // Changed from Ask for v1.0
            cookies_from_browser: None,
            download_timeout: 300,
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
        Self::format_filename_with_template(
            &self.filename_format,
            track_number,
            title,
            artist,
            album,
        )
    }

    /// Format a track filename using an explicit template (same placeholders as `filename_format`).
    pub fn format_filename_with_template(
        template: &str,
        track_number: usize,
        title: &str,
        artist: &str,
        album: &str,
    ) -> String {
        let safe_t = utils::sanitize_filesystem_chars(title);
        let safe_a = utils::sanitize_filesystem_chars(artist);
        let safe_al = utils::sanitize_filesystem_chars(album);
        template
            .replace("%n", &format!("{:02}", track_number))
            .replace("%t", &safe_t)
            .replace("%a", &safe_a)
            .replace("%A", &safe_al)
    }

    /// Format directory name according to template
    pub fn format_directory(&self, artist: &str, album: &str) -> String {
        let safe_a = utils::sanitize_filesystem_chars(artist);
        let safe_al = utils::sanitize_filesystem_chars(album);
        self.directory_format
            .replace("%a", &safe_a)
            .replace("%A", &safe_al)
    }
}

/// Print current configuration (read-only).
pub fn print_config_summary() -> Result<()> {
    let config = Config::load()?;
    let config_path = Config::config_path()?;

    println!("Configuration file: {}", config_path.display());
    println!();
    println!("Current settings:");
    println!(
        "  default_output_dir          = {:?}",
        config
            .default_output_dir
            .clone()
            .unwrap_or_else(|| "(system Music directory)".to_string())
    );
    println!("  download_cover        = {}", config.download_cover);
    println!(
        "  filename_format             = \"{}\"",
        config.filename_format
    );
    println!(
        "  directory_format            = \"{}\"",
        config.directory_format
    );
    println!(
        "  audio_quality               = {} kbps",
        config.audio_quality
    );
    println!("  audio_format                = {:?}", config.audio_format);
    println!(
        "  overwrite_existing          = {}",
        config.overwrite_existing
    );
    println!("  max_retries                 = {}", config.max_retries);
    println!("  create_playlist             = {}", config.create_playlist);
    println!("  refine_chapters             = {}", config.refine_chapters);
    println!(
        "  refine_silence_window       = {} s",
        config.refine_silence_window
    );
    println!("  refine_noise_db             = {}", config.refine_noise_db);
    println!(
        "  refine_min_silence          = {} s",
        config.refine_min_silence
    );
    println!(
        "  playlist_prefix_index       = {}",
        config.playlist_prefix_index
    );
    println!(
        "  playlist_behavior           = {:?}",
        config.playlist_behavior
    );
    println!(
        "  cookies_from_browser        = {:?}",
        config.cookies_from_browser.as_deref().unwrap_or("(none)")
    );
    println!(
        "  download_timeout            = {} s (0 = no timeout)",
        config.download_timeout
    );
    println!(
        "  dependency_auto_install     = {:?}",
        config.dependency_auto_install
    );
    println!(
        "  ytdlp_auto_update           = {}",
        config.ytdlp_auto_update
    );
    println!(
        "  ytdlp_update_interval_days  = {}",
        config.ytdlp_update_interval_days
    );
    println!();
    println!("Placeholders: filename %n %t %a %A — directory %a %A");

    Ok(())
}

fn read_line_trimmed() -> String {
    let mut line = String::new();
    io::stdin().read_line(&mut line).ok();
    line.trim().to_string()
}

fn prompt_line(label: &str, help: &str, display_default: &str) -> String {
    println!("{}", label);
    println!("  {}", help);
    print!("  [default: {}] > ", display_default);
    io::stdout().flush().ok();
    read_line_trimmed()
}

fn parse_bool_input(s: &str, current: bool) -> Result<bool> {
    if s.is_empty() {
        return Ok(current);
    }
    match s.to_lowercase().as_str() {
        "y" | "yes" | "true" | "1" => Ok(true),
        "n" | "no" | "false" | "0" => Ok(false),
        _ => Err(YtcsError::ConfigError(
            "Enter y/n, or leave empty to keep the current value".to_string(),
        )),
    }
}

fn parse_audio_quality(s: &str, current: u32) -> Result<u32> {
    if s.is_empty() {
        return Ok(current);
    }
    let q: u32 = s.parse().map_err(|_| {
        YtcsError::ConfigError("Audio quality must be 128, 192, or 320".to_string())
    })?;
    if q != 128 && q != 192 && q != 320 {
        return Err(YtcsError::ConfigError(
            "Audio quality must be 128, 192, or 320 kbps".to_string(),
        ));
    }
    Ok(q)
}

/// Interactive wizard: each setting shows its current value; Enter keeps it.
pub fn run_interactive_config_wizard() -> Result<()> {
    let mut config = Config::load()?;
    let path = Config::config_path()?;

    println!();
    println!("ytcs — configuration wizard");
    println!("Config file: {}", path.display());
    println!("Press Enter at any prompt to keep the current value.");
    println!();

    // default_output_dir
    let dir_disp = config
        .default_output_dir
        .clone()
        .unwrap_or_else(|| "(none — use system Music folder)".to_string());
    let input = prompt_line(
        "Default output directory",
        "Path for albums, or 'none' to use the system Music folder.",
        &dir_disp,
    );
    if input.is_empty() {
        // keep
    } else if input.eq_ignore_ascii_case("none") || input == "-" {
        config.default_output_dir = None;
    } else {
        config.default_output_dir = Some(input);
    }

    let dc = config.download_cover;
    let input = prompt_line(
        "Download cover art",
        "y/n — fetch and embed album thumbnail when available.",
        &format!("{}", dc),
    );
    config.download_cover = parse_bool_input(&input, dc)?;

    let ff = config.filename_format.clone();
    let input = prompt_line(
        "Track filename format",
        "Placeholders: %n track number, %t title, %a artist, %A album.",
        &ff,
    );
    if !input.is_empty() {
        config.filename_format = input;
    }

    let df = config.directory_format.clone();
    let input = prompt_line(
        "Album folder name format",
        "Placeholders: %a artist, %A album.",
        &df,
    );
    if !input.is_empty() {
        config.directory_format = input;
    }

    let aq = config.audio_quality;
    let input = prompt_line(
        "MP3 bitrate (kbps)",
        "Allowed: 128, 192, or 320.",
        &format!("{}", aq),
    );
    config.audio_quality = parse_audio_quality(&input, aq)?;

    println!("Audio output format");
    println!("  1 = mp3 (default)  2 = opus  3 = m4a");
    print!("  [default: {:?}] > ", config.audio_format);
    io::stdout().flush().ok();
    let af_in = read_line_trimmed();
    if !af_in.is_empty() {
        config.audio_format = match af_in.as_str() {
            "1" => AudioFormat::Mp3,
            "2" => AudioFormat::Opus,
            "3" => AudioFormat::M4a,
            _ => {
                return Err(YtcsError::ConfigError(
                    "Enter 1, 2, or 3 (or leave empty to keep)".to_string(),
                ));
            }
        };
    }

    let oe = config.overwrite_existing;
    let input = prompt_line(
        "Overwrite existing files",
        "y/n — replace existing track MP3s in the output folder when re-running.",
        &format!("{}", oe),
    );
    config.overwrite_existing = parse_bool_input(&input, oe)?;

    let mr = config.max_retries;
    let input = prompt_line(
        "Download retries",
        "Number of retries for yt-dlp network operations.",
        &format!("{}", mr),
    );
    if !input.is_empty() {
        config.max_retries = input.parse().map_err(|_| {
            YtcsError::ConfigError("max_retries must be a positive integer".to_string())
        })?;
    }

    let cp = config.create_playlist;
    let input = prompt_line(
        "Create .m3u playlist file",
        "y/n — write playlist.m3u in the album folder after splitting.",
        &format!("{}", cp),
    );
    config.create_playlist = parse_bool_input(&input, cp)?;

    let rc = config.refine_chapters;
    let input = prompt_line(
        "Refine chapter boundaries with silence detection",
        "y/n — extra ffmpeg pass to snap cuts to quiet gaps (YouTube timestamps can be slightly off).",
        &format!("{}", rc),
    );
    config.refine_chapters = parse_bool_input(&input, rc)?;

    let rw = config.refine_silence_window;
    let input = prompt_line(
        "Refine silence search window (seconds)",
        "± seconds around each chapter edge to look for a quiet cut.",
        &format!("{}", rw),
    );
    if !input.is_empty() {
        config.refine_silence_window = input.parse().map_err(|_| {
            YtcsError::ConfigError("refine_silence_window must be a number".to_string())
        })?;
    }

    let rnd = config.refine_noise_db;
    let input = prompt_line(
        "Refine silence noise threshold (dB)",
        "Typical: -30 to -50 (more negative = stricter silence).",
        &format!("{}", rnd),
    );
    if !input.is_empty() {
        config.refine_noise_db = input
            .parse()
            .map_err(|_| YtcsError::ConfigError("refine_noise_db must be a number".to_string()))?;
    }

    let rms = config.refine_min_silence;
    let input = prompt_line(
        "Refine minimum silence duration (seconds)",
        "Minimum length of a gap to treat as silence for refinement.",
        &format!("{}", rms),
    );
    if !input.is_empty() {
        config.refine_min_silence = input.parse().map_err(|_| {
            YtcsError::ConfigError("refine_min_silence must be a number".to_string())
        })?;
    }

    println!("Playlist behavior when a playlist URL is used");
    println!("  1 = ask  2 = video_only (default)  3 = playlist_only");
    print!("  [default: {:?}] > ", config.playlist_behavior);
    io::stdout().flush().ok();
    let pb_in = read_line_trimmed();
    if !pb_in.is_empty() {
        config.playlist_behavior = match pb_in.as_str() {
            "1" => PlaylistBehavior::Ask,
            "2" => PlaylistBehavior::VideoOnly,
            "3" => PlaylistBehavior::PlaylistOnly,
            _ => {
                return Err(YtcsError::ConfigError(
                    "Enter 1, 2, or 3 (or leave empty to keep)".to_string(),
                ));
            }
        };
    }

    let ppi = config.playlist_prefix_index;
    let input = prompt_line(
        "Prefix folder with playlist index (01-, 02-, …)",
        "y/n — when downloading multiple videos from a playlist, avoid folder name clashes.",
        &format!("{}", ppi),
    );
    config.playlist_prefix_index = parse_bool_input(&input, ppi)?;

    let cb_disp = config
        .cookies_from_browser
        .clone()
        .unwrap_or_else(|| "(none)".to_string());
    let input = prompt_line(
        "Cookies from browser",
        "chrome, firefox, edge, … — or 'none' to disable (see also ~/.config/ytcs/cookies.txt).",
        &cb_disp,
    );
    if !input.is_empty() {
        let valid = [
            "chrome", "firefox", "safari", "edge", "chromium", "brave", "opera", "vivaldi",
        ];
        if input.eq_ignore_ascii_case("none") || input == "-" {
            config.cookies_from_browser = None;
        } else if valid.contains(&input.to_lowercase().as_str()) {
            config.cookies_from_browser = Some(input.to_lowercase());
        } else {
            return Err(YtcsError::ConfigError(format!(
                "Unknown browser. Use one of: {}",
                valid.join(", ")
            )));
        }
    }

    let dt = config.download_timeout;
    let input = prompt_line(
        "Download timeout (seconds)",
        "0 = no socket timeout for yt-dlp.",
        &format!("{}", dt),
    );
    if !input.is_empty() {
        config.download_timeout = input.parse().map_err(|_| {
            YtcsError::ConfigError("download_timeout must be a non-negative integer".to_string())
        })?;
    }

    println!("Dependency auto-install (yt-dlp / ffmpeg)");
    println!("  1 = prompt  2 = always  3 = never");
    print!("  [default: {:?}] > ", config.dependency_auto_install);
    io::stdout().flush().ok();
    let dep_in = read_line_trimmed();
    if !dep_in.is_empty() {
        config.dependency_auto_install = match dep_in.as_str() {
            "1" => AutoInstallBehavior::Prompt,
            "2" => AutoInstallBehavior::Always,
            "3" => AutoInstallBehavior::Never,
            _ => {
                return Err(YtcsError::ConfigError(
                    "Enter 1, 2, or 3 (or leave empty to keep)".to_string(),
                ));
            }
        };
    }

    let yu = config.ytdlp_auto_update;
    let input = prompt_line(
        "Auto-update yt-dlp on download failure",
        "y/n — offer to update yt-dlp when a failure looks like an outdated extractor.",
        &format!("{}", yu),
    );
    config.ytdlp_auto_update = parse_bool_input(&input, yu)?;

    let yid = config.ytdlp_update_interval_days;
    let input = prompt_line(
        "Minimum days between yt-dlp update checks",
        "0 = no minimum interval (reserved for future use).",
        &format!("{}", yid),
    );
    if !input.is_empty() {
        config.ytdlp_update_interval_days = input.parse().map_err(|_| {
            YtcsError::ConfigError(
                "ytdlp_update_interval_days must be a non-negative integer".to_string(),
            )
        })?;
    }

    config.save()?;
    println!();
    println!("✓ Configuration saved to {}", path.display());
    Ok(())
}
