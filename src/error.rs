//! Error handling for YouTube Chapter Splitter.
//!
//! This module defines custom error types used throughout the application.

use thiserror::Error;

/// Custom error type for YouTube Chapter Splitter.
///
/// This enum groups all possible error types encountered during application execution.
#[derive(Error, Debug)]
pub enum YtcsError {
    /// Error occurred while downloading a video or thumbnail.
    #[error("Download error: {0}")]
    DownloadError(String),

    /// Error occurred during audio processing (splitting, conversion, etc.).
    #[error("Audio processing error: {0}")]
    AudioError(String),

    /// Error occurred during parsing or manipulation of chapters.
    #[error("Chapter parsing error: {0}")]
    ChapterError(String),

    /// Invalid or malformed YouTube URL.
    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),

    /// Missing required system tool (yt-dlp, ffmpeg, etc.).
    #[error("Missing system tool: {0}")]
    MissingTool(String),

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
