//! Module de gestion des playlists YouTube.
//!
//! Ce module détecte si une URL est une playlist et extrait les vidéos.

use crate::cookie_helper;
use crate::error::{Result, YtcsError};
use crate::ytdlp_error_parser;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Regex pour détecter une URL de playlist YouTube
static PLAYLIST_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[?&]list=([a-zA-Z0-9_-]+)").unwrap());

/// Information sur une vidéo dans une playlist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistVideo {
    /// ID de la vidéo
    pub id: String,

    /// Titre de la vidéo
    pub title: String,

    /// URL complète de la vidéo
    pub url: String,

    /// Durée en secondes
    pub duration: f64,
}

/// Information sur une playlist
#[derive(Debug, Clone)]
pub struct PlaylistInfo {
    /// ID de la playlist
    pub id: String,

    /// Titre de la playlist
    pub title: String,

    /// Liste des vidéos
    pub videos: Vec<PlaylistVideo>,
}

/// Check if a URL contains a playlist
///
/// # Arguments
///
/// * `url` - L'URL à vérifier
///
/// # Returns
///
/// `Some(playlist_id)` si l'URL contient une playlist, `None` sinon
///
/// # Exemples
///
/// ```
/// use youtube_chapter_splitter::playlist::is_playlist_url;
///
/// assert!(is_playlist_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf").is_some());
/// assert!(is_playlist_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_none());
/// ```
pub fn is_playlist_url(url: &str) -> Option<String> {
    PLAYLIST_REGEX.captures(url).map(|cap| cap[1].to_string())
}

/// Extract video ID from a YouTube URL
///
/// # Arguments
///
/// * `url` - L'URL YouTube
///
/// # Returns
///
/// `Some(video_id)` si trouvé, `None` sinon
pub fn extract_video_id(url: &str) -> Option<String> {
    // Regex pour extraire l'ID de vidéo
    let video_regex = Regex::new(r"(?:v=|/)([a-zA-Z0-9_-]{11})").unwrap();
    video_regex.captures(url).map(|cap| cap[1].to_string())
}

/// Obtenir les informations d'une playlist
///
/// # Arguments
///
/// * `url` - L'URL de la playlist
///
/// # Returns
///
/// Les informations de la playlist
///
/// # Errors
///
/// Returns an error if yt-dlp fails or if JSON parsing fails
pub fn get_playlist_info(url: &str, cookies_from_browser: Option<&str>) -> Result<PlaylistInfo> {
    // Utiliser yt-dlp pour obtenir les informations de la playlist
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

    // Parser chaque ligne JSON (une vidéo par ligne)
    let mut videos = Vec::new();
    let mut playlist_title = String::new();
    let mut playlist_id = String::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let json: serde_json::Value = serde_json::from_str(line).map_err(YtcsError::JsonError)?;

        // Extraire les informations de la playlist (première ligne)
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

        // Extraire les informations de la vidéo
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
/// * `url` - L'URL à nettoyer
///
/// # Returns
///
/// L'URL sans le paramètre de playlist
///
/// # Exemples
///
/// ```
/// use youtube_chapter_splitter::playlist::remove_playlist_param;
///
/// let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
/// let clean = remove_playlist_param(url);
/// assert_eq!(clean, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
/// ```
pub fn remove_playlist_param(url: &str) -> String {
    // Supprimer le paramètre list= et tout ce qui suit
    if let Some(pos) = url.find("&list=") {
        url[..pos].to_string()
    } else if let Some(pos) = url.find("?list=") {
        // Si list= est le premier paramètre
        if let Some(video_pos) = url.find("?v=") {
            // Garder le paramètre v=
            url[..video_pos + url[video_pos..].find('&').unwrap_or(url.len() - video_pos)]
                .to_string()
        } else {
            url[..pos].to_string()
        }
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_playlist_url() {
        assert!(is_playlist_url(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"
        )
        .is_some());
        assert!(
            is_playlist_url(
                "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf"
            )
            .is_some()
        );
        assert!(is_playlist_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").is_none());
    }

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert_eq!(
            extract_video_id("https://youtu.be/dQw4w9WgXcQ"),
            Some("dQw4w9WgXcQ".to_string())
        );
        assert!(extract_video_id("https://www.youtube.com/").is_none());
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
