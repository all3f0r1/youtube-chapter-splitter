//! Error handling for YouTube Chapter Splitter.
//!
//! This module defines custom error types used throughout the application.

use std::fmt;
use thiserror::Error;

/// Missing `yt-dlp`, `ffmpeg`, and/or `deno` after a dependency check.
///
/// `deno` is a JS runtime yt-dlp now requires to solve YouTube's `n` challenge
/// (otherwise audio formats are not available; only images are returned).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissingToolsError {
    pub missing_ytdlp: bool,
    pub missing_ffmpeg: bool,
    pub missing_deno: bool,
}

impl MissingToolsError {
    /// Tools to pass to [`crate::downloader::install_dependency`].
    pub fn tools_to_install(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.missing_ytdlp {
            v.push("yt-dlp");
        }
        if self.missing_ffmpeg {
            v.push("ffmpeg");
        }
        if self.missing_deno {
            v.push("deno");
        }
        v
    }

    fn ffmpeg_install_hint() -> &'static str {
        if cfg!(target_os = "linux") {
            "sudo apt install ffmpeg"
        } else if cfg!(target_os = "macos") {
            "brew install ffmpeg"
        } else {
            "Download from https://ffmpeg.org/download.html"
        }
    }

    fn deno_install_hint() -> &'static str {
        if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
            "curl -fsSL https://deno.land/install.sh | sh"
        } else if cfg!(target_os = "windows") {
            "irm https://deno.land/install.ps1 | iex"
        } else {
            "See https://deno.land/#installation"
        }
    }
}

impl fmt::Display for MissingToolsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Missing dependencies:")?;
        if self.missing_ytdlp {
            writeln!(f, "  - yt-dlp: pip install yt-dlp")?;
        }
        if self.missing_ffmpeg {
            writeln!(f, "  - ffmpeg: {}", Self::ffmpeg_install_hint())?;
        }
        if self.missing_deno {
            writeln!(
                f,
                "  - deno (JS runtime for yt-dlp's `n` challenge): {}",
                Self::deno_install_hint()
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for MissingToolsError {}

/// Custom error type for YouTube Chapter Splitter.
///
/// This enum groups all possible error types encountered during application execution.
#[derive(Error, Debug)]
pub enum YtcsError {
    /// Error occurred while downloading a video or thumbnail.
    #[error("Download error: {0}")]
    DownloadError(String),

    /// Both the direct CDN fetch and the yt-dlp fallback failed when retrieving a thumbnail.
    #[error("Thumbnail download failed (cdn: {http}; yt-dlp: {ytdlp})")]
    ThumbnailFailed { http: String, ytdlp: String },

    /// Error occurred during audio processing (splitting, conversion, etc.).
    #[error("Audio processing error: {0}")]
    AudioError(String),

    /// Error occurred during parsing or manipulation of chapters.
    #[error("Chapter parsing error: {0}")]
    ChapterError(String),

    /// Invalid or malformed YouTube URL.
    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),

    /// Missing required system tools (`yt-dlp`, `ffmpeg`).
    #[error(transparent)]
    MissingTools(#[from] MissingToolsError),

    /// I/O error (files, network, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error (video metadata, chapters, etc.).
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Regex compilation or execution error.
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Installation error for dependencies.
    #[error("Installation error: {0}")]
    InstallError(String),

    /// Generic error for cases not covered by other variants.
    #[error("Generic error: {0}")]
    Other(String),
}

/// Type alias for `Result<T, YtcsError>`.
///
/// Simplifies function signatures by using our custom error type.
pub type Result<T> = std::result::Result<T, YtcsError>;
