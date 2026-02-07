//! YouTube playlist management module.
//!
//! This module detects if a URL is a playlist and extracts videos.
//!
//! **Note**: By default, all URLs are treated as single videos (mode "video only").

use crate::cookie_helper;
use crate::error::{Result, YtcsError};
use crate::ytdlp_error_parser;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Information about a video in a playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistVideo {
    /// Video ID
    pub id: String,

    /// Video title
    pub title: String,

    /// Full video URL
    pub url: String,

    /// Duration in seconds
    pub duration: f64,
}

/// Information about a playlist
#[derive(Debug, Clone)]
pub struct PlaylistInfo {
    /// Playlist ID
    pub id: String,

    /// Playlist title
    pub title: String,

    /// List of videos
    pub videos: Vec<PlaylistVideo>,
}

/// Check if a URL contains a playlist
///
/// **IMPORTANT**: Always returns None to treat all URLs as single videos.
/// This is the default behavior - playlist downloads are not supported in TUI mode.
/// To download a playlist, users should use individual video URLs.
///
/// # Arguments
///
/// * `url` - The URL to check (not used, always returns None)
///
/// # Returns
///
/// Always `None` - all URLs are treated as single videos
pub fn is_playlist_url(_url: &str) -> Option<String> {
    // Always return None - treat all URLs as single videos
    // This implements the "video only" default behavior
    None
}

/// Get playlist information
///
/// # Arguments
///
/// * `url` - The playlist URL
///
/// # Returns
///
/// Playlist information
///
/// # Errors
///
/// Returns an error if yt-dlp fails or if JSON parsing fails
pub fn get_playlist_info(url: &str, cookies_from_browser: Option<&str>) -> Result<PlaylistInfo> {
    // Use yt-dlp to get playlist information
    let mut cmd = Command::new("yt-dlp");
    cmd.args(["--dump-json", "--flat-playlist", "--no-warnings"]);

    // Add cookie arguments
    cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);

    let output = cmd
        .arg(url)
        .output()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to run yt-dlp: {}", e)))?;

    if !output.status.success() {
        let raw_error = String::from_utf8_lossy(&output.stderr);
        let (error_msg, suggestion) =
            ytdlp_error_parser::parse_ytdlp_error(&raw_error, cookies_from_browser);

        let full_error = if let Some(sug) = suggestion {
            format!("{}\n\n{}", error_msg, sug)
        } else {
            error_msg
        };

        return Err(YtcsError::DownloadError(full_error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse each JSON line (one video per line)
    let mut videos = Vec::new();
    let mut playlist_title = String::new();
    let mut playlist_id = String::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let json: serde_json::Value = serde_json::from_str(line).map_err(YtcsError::JsonError)?;

        // Extract playlist information (first line)
        if playlist_title.is_empty() {
            if let Some(title) = json.get("playlist_title").and_then(|v| v.as_str()) {
                playlist_title = title.to_string();
            } else if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                playlist_title = title.to_string();
            }

            if let Some(id) = json.get("playlist_id").and_then(|v| v.as_str()) {
                playlist_id = id.to_string();
            }
        }

        // Extract video information
        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
            let title = json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let duration = json.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let url = format!("https://www.youtube.com/watch?v={}", id);

            videos.push(PlaylistVideo {
                id: id.to_string(),
                title,
                url,
                duration,
            });
        }
    }

    if videos.is_empty() {
        return Err(YtcsError::Other("No videos found in playlist".to_string()));
    }

    Ok(PlaylistInfo {
        id: playlist_id,
        title: playlist_title,
        videos,
    })
}

/// Remove playlist parameter from a URL
///
/// # Arguments
///
/// * `url` - The URL to clean
///
/// # Returns
///
/// The URL without the playlist parameter
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::playlist::remove_playlist_param;
///
/// let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
/// let clean = remove_playlist_param(url);
/// assert_eq!(clean, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
/// ```
pub fn remove_playlist_param(url: &str) -> String {
    // Remove the list= parameter and everything after
    if let Some(pos) = url.find("&list=") {
        url[..pos].to_string()
    } else if let Some(list_pos) = url.find("?list=") {
        // If list= is the first parameter, we need to extract v= if it exists
        let base = &url[..list_pos];
        if let Some(v_pos) = url.find("v=") {
            // Find where the v= value ends (either at & or end of string)
            let v_start = v_pos + 2;
            let v_end = url[v_start..]
                .find('&')
                .map(|p| v_start + p)
                .unwrap_or(url.len());
            // Extract video ID and reconstruct clean URL
            let video_id = &url[v_start..v_end];
            format!("{}?v={}", base, video_id)
        } else {
            // No video parameter, return base URL
            base.to_string()
        }
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downloader::extract_video_id;

    #[test]
    fn test_is_playlist_url() {
        // All URLs are now treated as single videos (video only behavior)
        assert!(is_playlist_url(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"
        )
        .is_none());
        assert!(
            is_playlist_url(
                "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"
            )
            .is_none()
        );
        assert!(is_playlist_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_none());
        // All playlist URLs are treated as single videos
        assert!(
            is_playlist_url(
                "https://www.youtube.com/watch?v=Kx0wf6xqzWg&list=RDKx0wf6xqzWg&start_radio=1"
            )
            .is_none()
        );
        assert!(is_playlist_url("https://www.youtube.com/watch?v=abc&list=RDMM12345").is_none());
    }

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").ok(),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ").ok(),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert!(extract_video_id("https://www.youtube.com/").is_err());
    }

    #[test]
    fn test_remove_playlist_param() {
        let url =
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
        assert_eq!(
            remove_playlist_param(url),
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
        );

        let url2 = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        assert_eq!(remove_playlist_param(url2), url2);
    }
}
