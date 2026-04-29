//! YouTube video download and metadata extraction.
//!
//! This module handles interaction with `yt-dlp` to download videos
//! and extract their metadata (title, duration, chapters).

use crate::chapters::{Chapter, parse_chapters_from_json};
use crate::error::{MissingToolsError, Result, YtcsError};
use crate::ytdlp_error_parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Information about a YouTube video.
#[derive(Debug)]
pub struct VideoInfo {
    pub title: String,
    pub duration: f64,
    pub chapters: Vec<Chapter>,
    pub video_id: String,
    /// Video description from yt-dlp (for chapter timestamps when JSON chapters are empty).
    pub description: Option<String>,
    /// Upload date from yt-dlp (`upload_date`, often YYYYMMDD).
    pub upload_date: Option<String>,
    /// Comma-separated categories / tags when present.
    pub genre: Option<String>,
    /// Canonical watch URL for this video.
    pub webpage_url: Option<String>,
    /// Best thumbnail URL from yt-dlp (`thumbnail` field), when present.
    pub thumbnail: Option<String>,
}

/// Checks for required system dependencies.
///
/// # Returns
///
/// Ok if all dependencies are present, otherwise an error with details
///
/// # Errors
///
/// Returns an error if `yt-dlp`, `ffmpeg`, or `deno` are missing.
///
/// `deno` is a JS runtime yt-dlp uses to solve YouTube's `n` challenge; without
/// it, audio formats are unavailable and downloads fail.
pub fn check_dependencies() -> Result<()> {
    let missing_ytdlp = Command::new("yt-dlp").arg("--version").output().is_err();
    let missing_ffmpeg = Command::new("ffmpeg").arg("-version").output().is_err();
    let missing_deno = Command::new("deno").arg("--version").output().is_err();

    if missing_ytdlp || missing_ffmpeg || missing_deno {
        return Err(YtcsError::MissingTools(MissingToolsError {
            missing_ytdlp,
            missing_ffmpeg,
            missing_deno,
        }));
    }

    Ok(())
}

/// Installs a missing system dependency.
///
/// # Arguments
///
/// * `tool` - The name of the tool to install ("yt-dlp", "ffmpeg", or "deno")
///
/// # Errors
///
/// Returns an error if the installation fails
pub fn install_dependency(tool: &str) -> Result<()> {
    let command = match tool {
        "yt-dlp" => "pip install yt-dlp",
        "ffmpeg" => {
            if cfg!(target_os = "linux") {
                "sudo apt install -y ffmpeg"
            } else if cfg!(target_os = "macos") {
                "brew install ffmpeg"
            } else {
                return Err(YtcsError::Other(
                    "Please install ffmpeg manually".to_string(),
                ));
            }
        }
        "deno" => {
            if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                "curl -fsSL https://deno.land/install.sh | sh -s -- -y"
            } else {
                return Err(YtcsError::Other(
                    "Install deno manually: see https://deno.land/#installation".to_string(),
                ));
            }
        }
        _ => return Err(YtcsError::Other(format!("Unknown tool: {}", tool))),
    };

    println!("Installing {}...", tool);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()
    } else {
        Command::new("sh").args(["-c", command]).output()
    };

    match output {
        Ok(out) if out.status.success() => {
            println!("✓ {} installed successfully", tool);
            if tool == "deno" {
                println!(
                    "  Note: deno is installed in ~/.deno/bin — add it to PATH:\n    \
                     echo 'export PATH=\"$HOME/.deno/bin:$PATH\"' >> ~/.bashrc && source ~/.bashrc"
                );
            }
            Ok(())
        }
        Ok(out) => {
            let error = String::from_utf8_lossy(&out.stderr);
            Err(YtcsError::Other(format!(
                "Failed to install {}: {}",
                tool, error
            )))
        }
        Err(e) => Err(YtcsError::Other(format!(
            "Failed to run install command: {}",
            e
        ))),
    }
}

/// Extracts a YouTube video ID from its URL.
///
/// # Arguments
///
/// * `url` - The YouTube video URL
///
/// # Returns
///
/// The video ID (11 characters)
///
/// # Errors
///
/// Returns an error if the URL is invalid or if the ID cannot be extracted
pub fn extract_video_id(url: &str) -> Result<String> {
    let patterns = [r"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})"];

    for pattern in &patterns {
        let re = regex::Regex::new(pattern)?;
        if let Some(caps) = re.captures(url)
            && let Some(id) = caps.get(1)
        {
            return Ok(id.as_str().to_string());
        }
    }

    Err(YtcsError::InvalidUrl(format!(
        "Unable to extract video ID from: {}",
        url
    )))
}

/// Retrieves information about a YouTube video.
///
/// Uses `yt-dlp` to extract video metadata.
///
/// # Arguments
///
/// * `url` - The YouTube video URL
/// * `cookies_from_browser` - Optional browser name for cookie extraction
///
/// # Returns
///
/// Video information (title, duration, chapters, ID)
///
/// # Errors
///
/// Returns an error if yt-dlp fails or if metadata is invalid
pub fn get_video_info(url: &str, cookies_from_browser: Option<&str>) -> Result<VideoInfo> {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--dump-json").arg("--no-playlist");
    crate::ytdlp_helper::add_ejs_args(&mut cmd);
    crate::cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);
    cmd.arg(url);

    let output = cmd
        .output()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let raw_error = String::from_utf8_lossy(&output.stderr);
        let (error_msg, suggestion) =
            ytdlp_error_parser::parse_ytdlp_error(&raw_error, cookies_from_browser);
        let full_error = match suggestion {
            Some(sug) => format!("{}\n\n{}", error_msg, sug),
            None => error_msg,
        };
        return Err(YtcsError::DownloadError(full_error));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&json_str)?;

    let title = data["title"]
        .as_str()
        .unwrap_or("Untitled Video")
        .to_string();

    let duration = data["duration"].as_f64().unwrap_or(0.0);

    let video_id = data["id"].as_str().unwrap_or("").to_string();

    let description = data["description"]
        .as_str()
        .map(std::string::ToString::to_string);

    let upload_date = data["upload_date"].as_str().map(str::to_string);

    let genre = data["categories"].as_array().and_then(|arr| {
        let parts: Vec<&str> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    });

    let webpage_url = data["webpage_url"].as_str().map(str::to_string);

    let thumbnail = data["thumbnail"]
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let chapters = if let Some(chapters_array) = data["chapters"].as_array() {
        if !chapters_array.is_empty() {
            parse_chapters_from_json(&json_str).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(VideoInfo {
        title,
        duration,
        chapters,
        video_id,
        description,
        upload_date,
        genre,
        webpage_url,
        thumbnail,
    })
}

/// Downloads the audio from a YouTube video as MP3.
///
/// Uses `yt-dlp` with a progress bar to download and convert the audio.
///
/// # Arguments
///
/// * `url` - The YouTube video URL
/// * `output_path` - The output path (without extension)
///
/// # Returns
///
/// The path of the downloaded MP3 file
///
/// # Errors
///
/// Returns an error if the download fails
pub fn download_audio(url: &str, output_path: &Path) -> Result<PathBuf> {
    println!("Downloading audio from YouTube...");

    // Create an indeterminate progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Downloading...");

    // Launch yt-dlp in the background
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .arg("--no-playlist");
    crate::ytdlp_helper::add_ejs_args(&mut cmd);
    let output = cmd
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| YtcsError::DownloadError(format!("Download failed: {}", e)))?;

    pb.finish_with_message("Download complete");

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(YtcsError::DownloadError(format!(
            "yt-dlp failed: {}",
            error
        )));
    }

    // yt-dlp adds .mp3 automatically
    let mut final_path = output_path.to_path_buf();
    final_path.set_extension("mp3");

    if !final_path.exists() {
        return Err(YtcsError::DownloadError(
            "Audio file was not created".to_string(),
        ));
    }

    Ok(final_path)
}

/// Browser-like User-Agent for thumbnail CDN requests (some networks block generic clients).
const THUMBNAIL_HTTP_USER_AGENT: &str = concat!(
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ",
    "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
);

/// Hard cap on thumbnail body size to prevent OOM from a misbehaving/hostile CDN.
const MAX_THUMBNAIL_BYTES: u64 = 8 * 1024 * 1024;

/// Per-request timeout for direct CDN thumbnail fetches. Real thumbnails are <500 KB.
const THUMBNAIL_HTTP_TIMEOUT_SECS: u64 = 10;

/// Strip query parameters from a URL for safe logging (signed CDN URLs may carry tokens).
fn redact_url_query(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

/// Outcome of a single CDN fetch attempt.
enum FetchOutcome {
    /// 200 OK with a non-empty body within the size cap.
    Body(Vec<u8>),
    /// 4xx, empty body — no point retrying or trying further attempts on this URL.
    Permanent(String),
    /// 5xx, transport error, body read error — retry up to the per-URL attempt budget.
    Retryable(String),
}

fn try_fetch_thumbnail_body(agent: &ureq::Agent, url: &str) -> FetchOutcome {
    match agent
        .get(url)
        .set("User-Agent", THUMBNAIL_HTTP_USER_AGENT)
        .call()
    {
        Ok(response) => {
            let status = response.status();
            if (500..600).contains(&status) {
                return FetchOutcome::Retryable(format!("HTTP {}", status));
            }
            if status != 200 {
                return FetchOutcome::Permanent(format!("HTTP {}", status));
            }
            let mut reader = response.into_reader();
            let mut limited = std::io::Read::take(&mut reader, MAX_THUMBNAIL_BYTES);
            let mut bytes = Vec::new();
            if let Err(e) = std::io::Read::read_to_end(&mut limited, &mut bytes) {
                return FetchOutcome::Retryable(format!("read body failed: {}", e));
            }
            if bytes.is_empty() {
                return FetchOutcome::Permanent("empty body".to_string());
            }
            FetchOutcome::Body(bytes)
        }
        Err(e) => FetchOutcome::Retryable(format!("transport error: {}", e)),
    }
}

fn thumbnail_candidate_urls(info: &VideoInfo, page_url: &str) -> Result<Vec<String>> {
    let video_id = if !info.video_id.is_empty() {
        info.video_id.clone()
    } else {
        extract_video_id(page_url)?
    };

    let mut urls = Vec::new();
    let mut seen = std::collections::HashSet::<String>::new();
    let mut push_unique = |u: String| {
        if seen.insert(u.clone()) {
            urls.push(u);
        }
    };

    if let Some(ref t) = info.thumbnail {
        let t = t.trim();
        if t.starts_with("http://") || t.starts_with("https://") {
            push_unique(t.to_string());
        }
    }

    for base in ["https://i.ytimg.com/vi", "https://img.youtube.com/vi"] {
        for name in ["maxresdefault", "hqdefault", "mqdefault"] {
            push_unique(format!("{}/{}/{}.jpg", base, video_id, name));
        }
    }

    Ok(urls)
}

/// First nonempty `cover.{jpg,jpeg,webp,png}` in `output_dir`, if any.
pub fn album_cover_path(output_dir: &Path) -> Option<PathBuf> {
    for ext in ["jpg", "jpeg", "webp", "png"] {
        let p = output_dir.join(format!("cover.{ext}"));
        if p.is_file() && std::fs::metadata(&p).map(|m| m.len() > 0).unwrap_or(false) {
            return Some(p);
        }
    }
    None
}

/// Fallback path: invoke `yt-dlp --write-thumbnail` to produce `cover.jpg` in `output_dir`.
///
/// Used when direct CDN fetches fail. Returns the path to the written cover, or a
/// `DownloadError` whose message is yt-dlp's stderr (caller must avoid leaking it
/// directly to user-facing UI — keep it in `log` instead).
fn download_thumbnail_ytdlp(
    page_url: &str,
    output_dir: &Path,
    cookies_from_browser: Option<&str>,
) -> Result<PathBuf> {
    let out_template = output_dir.join("cover.%(ext)s");
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--no-playlist")
        .arg("--skip-download")
        .arg("--write-thumbnail")
        .arg("--convert-thumbnails")
        .arg("jpg")
        .arg("-o")
        .arg(&out_template)
        .arg(page_url);
    crate::ytdlp_helper::add_ejs_args(&mut cmd);
    crate::cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);

    let output = cmd.output().map_err(|e| {
        YtcsError::DownloadError(format!("Failed to run yt-dlp for thumbnail: {}", e))
    })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        log::debug!("yt-dlp thumbnail fallback failed: {}", err.trim());
        return Err(YtcsError::DownloadError(format!(
            "yt-dlp could not fetch thumbnail: {}",
            err.trim()
        )));
    }

    album_cover_path(output_dir).ok_or_else(|| {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::debug!(
            "yt-dlp thumbnail fallback exited 0 but no cover file present. stderr: {}",
            stderr.trim()
        );
        YtcsError::DownloadError("yt-dlp finished but no cover image was written".to_string())
    })
}

/// Downloads the thumbnail using metadata from [`get_video_info`] (preferred: yt-dlp `thumbnail` URL + CDN fallbacks, then `yt-dlp --write-thumbnail`).
///
/// # Errors
///
/// Returns [`YtcsError::ThumbnailFailed`] when both the direct CDN path and the yt-dlp
/// fallback fail. The error carries both failure reasons so callers can present a
/// single concise message and log full detail at debug level.
pub fn download_thumbnail_from_info(
    info: &VideoInfo,
    page_url: &str,
    output_dir: &Path,
    cookies_from_browser: Option<&str>,
) -> Result<PathBuf> {
    let thumbnail_urls = thumbnail_candidate_urls(info, page_url)?;

    let output_path = output_dir.join("cover.jpg");

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(THUMBNAIL_HTTP_TIMEOUT_SECS))
        .build();

    let mut last_http_error: Option<String> = None;

    for thumb_url in thumbnail_urls {
        let safe_url = redact_url_query(&thumb_url);
        for attempt in 1..=3 {
            match try_fetch_thumbnail_body(&agent, &thumb_url) {
                FetchOutcome::Body(bytes) => {
                    std::fs::write(&output_path, &bytes).map_err(|e| {
                        YtcsError::DownloadError(format!("Failed to write thumbnail: {}", e))
                    })?;
                    log::debug!("Thumbnail saved from {} ({} bytes)", safe_url, bytes.len());
                    return Ok(output_path);
                }
                FetchOutcome::Permanent(reason) => {
                    let msg = format!("{} ({})", reason, safe_url);
                    log::debug!("Thumbnail attempt {}/3 (no retry): {}", attempt, msg);
                    last_http_error = Some(msg);
                    break;
                }
                FetchOutcome::Retryable(reason) => {
                    let msg = format!("{} ({})", reason, safe_url);
                    log::debug!("Thumbnail attempt {}/3 (retryable): {}", attempt, msg);
                    last_http_error = Some(msg);
                    if attempt < 3 {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        continue;
                    }
                    break;
                }
            }
        }
    }

    log::debug!(
        "Direct CDN thumbnail fetch failed (last error: {}); falling back to yt-dlp",
        last_http_error.as_deref().unwrap_or("none")
    );
    download_thumbnail_ytdlp(page_url, output_dir, cookies_from_browser).map_err(|ytdlp_err| {
        let ytdlp_msg = match ytdlp_err {
            YtcsError::DownloadError(s) => s,
            other => other.to_string(),
        };
        let http_msg = last_http_error.unwrap_or_else(|| "no candidate URLs".to_string());
        YtcsError::ThumbnailFailed {
            http: http_msg,
            ytdlp: ytdlp_msg,
        }
    })
}

/// Downloads the thumbnail using only the page URL (no `yt-dlp` thumbnail field; uses CDN fallbacks only).
///
/// Prefer [`download_thumbnail_from_info`] when [`VideoInfo`] is already available from [`get_video_info`].
pub fn download_thumbnail(url: &str, output_dir: &Path) -> Result<PathBuf> {
    let video_id = extract_video_id(url)?;
    let info = VideoInfo {
        title: String::new(),
        duration: 0.0,
        chapters: Vec::new(),
        video_id,
        description: None,
        upload_date: None,
        genre: None,
        webpage_url: None,
        thumbnail: None,
    };
    download_thumbnail_from_info(&info, url, output_dir, None)
}

#[cfg(test)]
mod thumbnail_helpers_tests {
    use super::*;

    #[test]
    fn redact_strips_query_string() {
        assert_eq!(
            redact_url_query("https://i.ytimg.com/vi/abc/maxresdefault.jpg?sig=SECRET&pot=TOKEN"),
            "https://i.ytimg.com/vi/abc/maxresdefault.jpg"
        );
    }

    #[test]
    fn redact_passes_through_when_no_query() {
        let url = "https://i.ytimg.com/vi/abc/hqdefault.jpg";
        assert_eq!(redact_url_query(url), url);
    }

    #[test]
    fn redact_handles_empty_query() {
        assert_eq!(
            redact_url_query("https://example.com/x?"),
            "https://example.com/x"
        );
    }
}
