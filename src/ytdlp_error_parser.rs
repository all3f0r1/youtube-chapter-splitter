//! Parser for yt-dlp error messages to provide user-friendly error messages.
//!
//! This module analyzes raw yt-dlp error output and transforms it into
//! actionable, user-friendly error messages with suggestions.

use crate::cookie_helper;

/// Parse a yt-dlp error message and return a user-friendly version.
///
/// # Arguments
///
/// * `raw_error` - The raw error message from yt-dlp stderr
/// * `cookies_from_browser` - Optional browser name configured for cookies
///
/// # Returns
///
/// A tuple of (error_message, optional_suggestion)
pub fn parse_ytdlp_error(
    raw_error: &str,
    cookies_from_browser: Option<&str>,
) -> (String, Option<String>) {
    let error_lower = raw_error.to_lowercase();

    // Check for authentication/membership errors
    if error_lower.contains("members-only")
        || error_lower.contains("this video is only available")
        || error_lower.contains("join this channel")
        || error_lower.contains("private video")
        || error_lower.contains("sign in to confirm")
    {
        let message =
            "This video requires authentication (member-only or private content)".to_string();
        let suggestion = if cookie_helper::cookies_available(cookies_from_browser) {
            Some("Your cookies may have expired. Try:\n  1. Export fresh cookies from your browser\n  2. Update ~/.config/ytcs/cookies.txt\n  3. Or reconfigure: ytcs set cookies_from_browser <browser>".to_string())
        } else {
            Some("You need to authenticate. Choose one option:\n  1. Export cookies: See COOKIES_SETUP.md for instructions\n  2. Or configure browser: ytcs set cookies_from_browser chrome".to_string())
        };
        return (message, suggestion);
    }

    // Check for age-restricted content
    if error_lower.contains("age-restricted") || error_lower.contains("age restricted") {
        let message = "This video is age-restricted".to_string();
        let suggestion = if cookie_helper::cookies_available(cookies_from_browser) {
            Some("Your cookies may not have the required age verification. Try logging in to YouTube in your browser and exporting fresh cookies.".to_string())
        } else {
            Some("You need to authenticate with an age-verified account:\n  1. Log in to YouTube in your browser\n  2. Export cookies (see COOKIES_SETUP.md)\n  3. Or configure: ytcs set cookies_from_browser chrome".to_string())
        };
        return (message, suggestion);
    }

    // Check for geo-blocking
    if error_lower.contains("not available in your country")
        || error_lower.contains("geo-restricted")
        || error_lower.contains("blocked in your country")
    {
        return (
            "This video is not available in your country (geo-restricted)".to_string(),
            Some("You may need to use a VPN or proxy to access this content.".to_string()),
        );
    }

    // Check for video unavailable/deleted
    if error_lower.contains("video unavailable")
        || error_lower.contains("has been removed")
        || error_lower.contains("this video is no longer available")
    {
        return (
            "This video is no longer available (deleted or made private)".to_string(),
            None,
        );
    }

    // Check for network errors
    if error_lower.contains("unable to download")
        || error_lower.contains("http error")
        || error_lower.contains("connection")
        || error_lower.contains("timeout")
    {
        return (
            "Network error while downloading".to_string(),
            Some("Check your internet connection and try again.".to_string()),
        );
    }

    // Check for invalid URL
    if error_lower.contains("invalid url") || error_lower.contains("unsupported url") {
        return (
            "Invalid or unsupported YouTube URL".to_string(),
            Some("Make sure you're using a valid YouTube video URL.".to_string()),
        );
    }

    // Default: return cleaned error message
    let cleaned_error = clean_error_message(raw_error);
    (cleaned_error, None)
}

/// Clean up a raw yt-dlp error message by removing technical noise.
fn clean_error_message(raw_error: &str) -> String {
    // Take only the first few lines and remove common prefixes
    let lines: Vec<&str> = raw_error.lines().take(3).collect();

    let mut cleaned = lines.join(" ");

    // Remove common yt-dlp prefixes
    cleaned = cleaned.replace("ERROR:", "");
    cleaned = cleaned.replace("[youtube]", "");
    cleaned = cleaned.replace("[download]", "");

    // Trim and limit length
    cleaned = cleaned.trim().to_string();
    if cleaned.len() > 200 {
        cleaned.truncate(197);
        cleaned.push_str("...");
    }

    if cleaned.is_empty() {
        "yt-dlp failed with an unknown error".to_string()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_members_only_error() {
        let error = "ERROR: [youtube] This video is only available for members";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("authentication"));
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_age_restricted_error() {
        let error = "ERROR: This video is age-restricted";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("age-restricted"));
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_geo_restricted_error() {
        let error = "ERROR: This video is not available in your country";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("geo-restricted"));
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_video_unavailable() {
        let error = "ERROR: Video unavailable";
        let (msg, _) = parse_ytdlp_error(error, None);
        assert!(msg.contains("no longer available"));
    }

    #[test]
    fn test_network_error() {
        let error = "ERROR: HTTP Error 500: Internal Server Error";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("Network error"));
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_clean_error_message() {
        let error = "ERROR: [youtube] Something went wrong with the download";
        let cleaned = clean_error_message(error);
        assert!(!cleaned.contains("ERROR:"));
        assert!(!cleaned.contains("[youtube]"));
    }
}
