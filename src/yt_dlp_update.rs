//! yt-dlp auto-update functionality
//!
//! This module handles automatic updates of yt-dlp when downloads fail
//! or when the version is outdated.

use crate::error::{Result, YtcsError};
use colored::Colorize;
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Update configuration
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// Minimum interval between automatic update checks
    pub min_update_interval: Duration,
    /// Whether to always attempt update on failure
    pub force_on_failure: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            min_update_interval: Duration::from_secs(24 * 60 * 60), // 24 hours
            force_on_failure: true,
        }
    }
}

/// Get the path to the last update timestamp file
fn last_update_file() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|home| {
        home.join(".config")
            .join("ytcs")
            .join("last_ytdlp_update.txt")
    })
}

/// Get the timestamp of the last yt-dlp update attempt
pub fn get_last_update_time() -> Option<SystemTime> {
    let path = last_update_file()?;
    let content = std::fs::read_to_string(path).ok()?;
    let timestamp = content.trim().parse::<u64>().ok()?;
    Some(SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp))
}

/// Save the timestamp of this update attempt
fn save_update_time() -> Result<()> {
    let path = last_update_file()
        .ok_or_else(|| YtcsError::ConfigError("Cannot determine home directory".to_string()))?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| YtcsError::ConfigError(format!("Invalid system time: {}", e)))?
        .as_secs();

    std::fs::write(path, now.to_string())?;
    Ok(())
}

/// Check if an update should be attempted based on time elapsed
pub fn should_check_for_update(config: &UpdateConfig) -> bool {
    if let Some(last_time) = get_last_update_time()
        && let Ok(elapsed) = last_time.elapsed()
    {
        return elapsed >= config.min_update_interval;
    }
    true // No previous update time, should check
}

/// Get the current yt-dlp version
pub fn get_ytdlp_version() -> Option<String> {
    let output = Command::new("yt-dlp").arg("--version").output().ok()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(version)
    } else {
        None
    }
}

/// Update yt-dlp to the latest version
///
/// Returns Ok(true) if update was successful, Ok(false) if already up to date,
/// Err if update failed
pub fn update_ytdlp(show_output: bool) -> Result<bool> {
    if show_output {
        eprintln!("{}", "Updating yt-dlp to the latest version...".yellow());
    }

    // Try common update methods
    let methods = [
        // Method 1: pip (Python package manager)
        || {
            Command::new("python")
                .args(["-m", "pip", "install", "--upgrade", "yt-dlp"])
                .output()
        },
        // Method 2: python3
        || {
            Command::new("python3")
                .args(["-m", "pip", "install", "--upgrade", "yt-dlp"])
                .output()
        },
        // Method 3: yt-dlp --update
        || Command::new("yt-dlp").arg("--update").output(),
    ];

    let mut last_error = None;

    for method in methods {
        match method() {
            Ok(output) => {
                if output.status.success() {
                    save_update_time()?;

                    if let Some(new_version) = get_ytdlp_version() {
                        if show_output {
                            eprintln!(
                                "{}",
                                format!("✓ yt-dlp updated successfully (version {})", new_version)
                                    .green()
                            );
                        }
                    } else if show_output {
                        eprintln!("{}", "✓ yt-dlp updated successfully".green());
                    }

                    return Ok(true);
                }
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    // All methods failed
    Err(YtcsError::InstallError(format!(
        "Failed to update yt-dlp. Last error: {}",
        last_error
            .map(|e| e.to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    )))
}

/// Check if yt-dlp needs updating and optionally update
///
/// Returns Ok(true) if update was performed, Ok(false) if no update needed
pub fn check_and_update(config: &UpdateConfig, force: bool, show_output: bool) -> Result<bool> {
    if !force && !should_check_for_update(config) {
        return Ok(false);
    }

    // Check current version
    let old_version = get_ytdlp_version();

    if let Some(ref ver) = old_version
        && show_output
    {
        eprintln!("{}", format!("Current yt-dlp version: {}", ver).dimmed());
    }

    // Attempt update
    update_ytdlp(show_output)
}

/// Attempt to update yt-dlp after a download failure
///
/// This is called when yt-dlp fails to download, suggesting the version
/// may be outdated. Returns true if update was attempted.
pub fn attempt_update_on_failure(error: &YtcsError, config: &UpdateConfig) -> Result<bool> {
    // Only attempt update for yt-dlp related errors
    let is_ytdlp_error = matches!(error, YtcsError::DownloadError(_));

    if !is_ytdlp_error || !config.force_on_failure {
        return Ok(false);
    }

    // Don't spam updates - check interval
    if !should_check_for_update(config) {
        return Ok(false);
    }

    eprintln!();
    eprintln!(
        "{}",
        "Download failed. This may be due to an outdated yt-dlp version.".yellow()
    );
    eprintln!("{}", "Attempting to update yt-dlp...".dimmed());

    update_ytdlp(true)?;

    eprintln!();
    eprintln!("{}", "Please retry your download.".cyan());

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_config_default() {
        let config = UpdateConfig::default();
        assert_eq!(
            config.min_update_interval,
            Duration::from_secs(24 * 60 * 60)
        );
        assert!(config.force_on_failure);
    }

    #[test]
    fn test_should_check_when_no_previous_update() {
        // Clear any existing update file for testing
        if let Some(path) = last_update_file() {
            let _ = std::fs::remove_file(&path);
        }

        let config = UpdateConfig::default();
        assert!(should_check_for_update(&config));
    }
}
