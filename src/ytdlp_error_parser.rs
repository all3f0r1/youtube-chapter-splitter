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

    // Check for missing JS runtime (YouTube's `n` challenge)
    // yt-dlp needs deno (or another JS runtime) to resolve audio formats.
    if error_lower.contains("n challenge")
        || error_lower.contains("javascript runtime")
        || (error_lower.contains("requested format is not available")
            && error_lower.contains("only images are available"))
    {
        return (
            "yt-dlp cannot resolve audio formats: no JavaScript runtime found.".to_string(),
            Some(
                "YouTube now requires a JS runtime to solve its `n` challenge.\n  \
                 Install deno (recommended):\n    \
                 curl -fsSL https://deno.land/install.sh | sh\n  \
                 Then add to PATH:\n    \
                 echo 'export PATH=\"$HOME/.deno/bin:$PATH\"' >> ~/.bashrc && source ~/.bashrc"
                    .to_string(),
            ),
        );
    }

    // Check for YouTube bot-detection / rate limiting
    // ("Sign in to confirm you're not a bot" or HTTP 429 Too Many Requests)
    if error_lower.contains("not a bot")
        || error_lower.contains("confirm you're not")
        || error_lower.contains("http error 429")
        || error_lower.contains("too many requests")
    {
        let message =
            "YouTube is rate-limiting this IP and asks for a signed-in session to continue."
                .to_string();
        let suggestion = if cookie_helper::cookies_available(cookies_from_browser) {
            Some("Your configured cookies were rejected or are stale. Try:\n  1. Log in to YouTube in your browser, then retry\n  2. Re-export cookies to ~/.config/ytcs/cookies.txt\n  3. Wait a few minutes — the rate limit clears on its own".to_string())
        } else {
            Some("Configure cookies so yt-dlp can authenticate:\n  • ytcs config → 'Cookies from browser' (chrome, firefox, brave, …)\n  • For LibreWolf / custom profile: use 'firefox:/path/to/profile'\n  • Or export a cookies.txt to ~/.config/ytcs/cookies.txt".to_string())
        };
        return (message, suggestion);
    }

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
            Some("Your cookies may have expired. Try:\n  1. Export fresh cookies from your browser\n  2. Update ~/.config/ytcs/cookies.txt\n  3. Or run: ytcs config (set Cookies from browser)".to_string())
        } else {
            Some("You need to authenticate. Choose one option:\n  1. Export cookies: see COOKIES_SETUP.md\n  2. Or run: ytcs config (set Cookies from browser)".to_string())
        };
        return (message, suggestion);
    }

    // Check for age-restricted content
    if error_lower.contains("age-restricted") || error_lower.contains("age restricted") {
        let message = "This video is age-restricted".to_string();
        let suggestion = if cookie_helper::cookies_available(cookies_from_browser) {
            Some("Your cookies may not have the required age verification. Try logging in to YouTube in your browser and exporting fresh cookies.".to_string())
        } else {
            Some("You need to authenticate with an age-verified account:\n  1. Log in to YouTube in your browser\n  2. Export cookies (see COOKIES_SETUP.md)\n  3. Or run: ytcs config (set Cookies from browser)".to_string())
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
    fn test_bot_detection_error() {
        let error = "ERROR: [youtube] 0cLebDx4CJc: Sign in to confirm you're not a bot.";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("rate-limiting"));
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_rate_limit_error() {
        let error = "WARNING: HTTP Error 429: Too Many Requests";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("rate-limiting"));
        assert!(suggestion.is_some());
    }

    #[test]
    fn test_js_runtime_missing_error() {
        let error = "WARNING: [youtube] n challenge solving failed: Some formats may be missing. Ensure you have a supported JavaScript runtime";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("JavaScript runtime"));
        assert!(suggestion.unwrap().contains("deno"));
    }

    #[test]
    fn test_only_images_available_error() {
        let error = "WARNING: Only images are available for download\nERROR: Requested format is not available";
        let (msg, suggestion) = parse_ytdlp_error(error, None);
        assert!(msg.contains("JavaScript runtime"));
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
