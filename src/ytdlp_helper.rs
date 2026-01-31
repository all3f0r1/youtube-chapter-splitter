//! Helper functions for yt-dlp version management and auto-update.
//!
//! This module provides utilities to:
//! - Check the current yt-dlp version
//! - Detect if yt-dlp is outdated (older than 90 days)
//! - Auto-update yt-dlp when download fails due to version issues

use crate::error::{Result, YtcsError};
use colored::Colorize;
use regex::Regex;
use std::io::Write;
use std::process::Command;

/// Information about yt-dlp version.
#[derive(Debug, Clone)]
pub struct YtdlpVersionInfo {
    /// Version string (e.g., "2025.10.22")
    pub version: String,
    /// Whether the version is outdated (>90 days old)
    pub is_outdated: bool,
    /// Days since release
    pub days_since_release: Option<i64>,
}

/// Checks if yt-dlp is installed and returns version information.
pub fn get_ytdlp_version() -> Option<YtdlpVersionInfo> {
    let output = Command::new("yt-dlp").arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Parse version to extract date (format: YYYY.MM.DD)
    let re = Regex::new(r"(\d{4})\.(\d{2})\.(\d{2})").ok()?;
    let caps = re.captures(&version_str)?;

    let year: i32 = caps.get(1)?.as_str().parse().ok()?;
    let month: u8 = caps.get(2)?.as_str().parse().ok()?;
    let day: u8 = caps.get(3)?.as_str().parse().ok()?;

    // Calculate days since release
    let release_date =
        time::Date::from_calendar_date(year, time::Month::try_from(month).ok()?, day).ok()?;
    let today = time::OffsetDateTime::now_utc().date();
    let duration = today - release_date;
    let days_since_release = duration.whole_days();

    let is_outdated = days_since_release > 90;

    Some(YtdlpVersionInfo {
        version: version_str,
        is_outdated,
        days_since_release: Some(days_since_release),
    })
}

/// Detects if a download error is likely due to outdated yt-dlp.
///
/// Checks stderr for patterns like:
/// - HTTP Error 403: Forbidden
/// - "is older than 90 days" warning
pub fn is_outdated_error(stderr: &str) -> bool {
    let stderr_lower = stderr.to_lowercase();

    // Check for version warning
    if stderr_lower.contains("older than 90 days") {
        return true;
    }

    // Check for HTTP 403 errors (often caused by outdated yt-dlp)
    if stderr_lower.contains("http error 403") || stderr_lower.contains("forbidden") {
        // Also check if yt-dlp is actually outdated
        if let Some(version_info) = get_ytdlp_version() {
            return version_info.is_outdated;
        }
    }

    false
}

/// Prompts user to update yt-dlp and performs the update if accepted.
///
/// Returns `true` if update was performed, `false` otherwise.
pub fn prompt_and_update_ytdlp() -> Result<bool> {
    // Check current version
    let version_info = get_ytdlp_version();

    let message = if let Some(info) = version_info {
        if info.is_outdated {
            format!(
                "{}",
                format!(
                    "Your yt-dlp version ({} / {} days old) may be outdated. Update to the latest version?",
                    info.version,
                    info.days_since_release.unwrap_or(0)
                )
                .yellow()
            )
        } else {
            "Update yt-dlp to the latest version?".to_string()
        }
    } else {
        "yt-dlp may need to be updated. Install the latest version?".to_string()
    };

    println!();
    println!("{}", message);
    print!("{}", "Update yt-dlp? [Y/n]: ".bold());
    std::io::stdout().flush()?;

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;
    let choice = choice.trim().to_lowercase();

    if choice == "n" || choice == "no" {
        return Ok(false);
    }

    update_ytdlp()?;
    Ok(true)
}

/// Updates yt-dlp to the latest version.
///
/// Tries multiple methods:
/// 1. pip install --upgrade --break-system-packages yt-dlp (newer pip)
/// 2. pip install --upgrade yt-dlp (older pip)
/// 3. pipx install yt-dlp --upgrade (if pipx is available)
pub fn update_ytdlp() -> Result<()> {
    println!("{}", "Updating yt-dlp...".cyan());

    let methods = [
        "pip install --upgrade --break-system-packages yt-dlp",
        "pip3 install --upgrade --break-system-packages yt-dlp",
        "pip install --upgrade yt-dlp",
        "pip3 install --upgrade yt-dlp",
        "pipx upgrade yt-dlp",
    ];

    for (i, cmd) in methods.iter().enumerate() {
        log::debug!("Trying update method {}: {}", i + 1, cmd);

        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let result = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", cmd]).output()
        } else {
            Command::new(parts[0]).args(&parts[1..]).output()
        };

        match result {
            Ok(output) if output.status.success() => {
                let new_version = get_ytdlp_version();
                if let Some(info) = new_version {
                    println!(
                        "{}",
                        format!("✓ yt-dlp updated successfully to {}", info.version).green()
                    );
                } else {
                    println!("{}", "✓ yt-dlp updated successfully".green());
                }
                return Ok(());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log::debug!("Update method {} failed: {}", i + 1, stderr);
            }
            Err(e) => {
                log::debug!("Update method {} failed to run: {}", i + 1, e);
            }
        }
    }

    // If all methods failed, provide manual instructions
    Err(YtcsError::Other(
        "Failed to auto-update yt-dlp. Please run: pip install --upgrade yt-dlp".to_string(),
    ))
}

/// Checks if yt-dlp needs an update and returns a warning message if so.
pub fn check_ytdlp_update_needed() -> Option<String> {
    if let Some(info) = get_ytdlp_version() {
        if info.is_outdated {
            return Some(format!(
                "Your yt-dlp version ({} / {} days old) is outdated. Consider updating: pip install --upgrade yt-dlp",
                info.version,
                info.days_since_release.unwrap_or(0)
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ytdlp_version() {
        let version = get_ytdlp_version();
        // We can't assert too much since the environment varies
        // Just check it doesn't panic
        drop(version);
    }
}
