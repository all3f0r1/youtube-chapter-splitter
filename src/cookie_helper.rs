//! Helper functions for cookie management with yt-dlp.
//!
//! This module provides utilities to add cookie-related arguments to yt-dlp commands,
//! supporting both cookie files and browser cookie extraction.

use std::process::Command;

/// Add cookie arguments to a yt-dlp command based on configuration.
///
/// Priority order:
/// 1. If `cookies_from_browser` is configured, use `--cookies-from-browser`
/// 2. If `~/.config/ytcs/cookies.txt` exists, use `--cookies`
/// 3. Otherwise, no cookie arguments are added
///
/// # Arguments
///
/// * `cmd` - The Command to add cookie arguments to
/// * `cookies_from_browser` - Optional browser name to extract cookies from
///
/// # Returns
///
/// A message describing which cookie method was used, if any
pub fn add_cookie_args(cmd: &mut Command, cookies_from_browser: Option<&str>) -> Option<String> {
    // Priority 1: Use browser cookie extraction if configured
    if let Some(browser) = cookies_from_browser {
        cmd.arg("--cookies-from-browser").arg(browser);
        return Some(format!("Using cookies from {} browser", browser));
    }

    // Priority 2: Use cookies.txt file if it exists
    if let Some(home) = dirs::home_dir() {
        let cookies_path = home.join(".config/ytcs/cookies.txt");
        if cookies_path.exists() {
            cmd.arg("--cookies").arg(cookies_path);
            return Some("Using cookies from file".to_string());
        }
    }

    // No cookies available
    None
}

/// Check if cookies are available (either from browser config or file).
///
/// # Arguments
///
/// * `cookies_from_browser` - Optional browser name to extract cookies from
///
/// # Returns
///
/// `true` if cookies are available, `false` otherwise
pub fn cookies_available(cookies_from_browser: Option<&str>) -> bool {
    if cookies_from_browser.is_some() {
        return true;
    }

    if let Some(home) = dirs::home_dir() {
        let cookies_path = home.join(".config/ytcs/cookies.txt");
        return cookies_path.exists();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_cookie_args_with_browser() {
        let mut cmd = Command::new("yt-dlp");
        let result = add_cookie_args(&mut cmd, Some("chrome"));
        assert!(result.is_some());
        assert!(result.unwrap().contains("chrome"));
    }

    #[test]
    fn test_add_cookie_args_no_cookies() {
        let mut cmd = Command::new("yt-dlp");
        // This might return Some if cookies.txt exists, but we're just testing it doesn't panic
        let _result = add_cookie_args(&mut cmd, None);
    }

    #[test]
    fn test_cookies_available_with_browser() {
        assert!(cookies_available(Some("firefox")));
    }
}
