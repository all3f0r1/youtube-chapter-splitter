//! Error handling for YouTube Chapter Splitter
//!
//! This module defines custom error types used throughout the application.

use thiserror::Error;

/// Custom error type for YouTube Chapter Splitter
///
/// This enum groups all possible error types encountered during application execution.
#[derive(Error, Debug)]
pub enum YtcsError {
    /// Error during video or thumbnail download
    #[error("Download error: {0}")]
    DownloadError(String),

    /// Error during audio processing (splitting, conversion, etc.)
    #[error("Audio processing error: {0}")]
    AudioError(String),

    /// Error during chapter parsing or manipulation
    #[error("Chapter parsing error: {0}")]
    ChapterError(String),

    /// Invalid or malformed YouTube URL
    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),

    /// Missing required system tool (yt-dlp, ffmpeg, etc.)
    #[error("Missing system tool: {0}")]
    MissingTool(String),

    /// Error during dependency installation
    #[error("Installation error: {0}")]
    InstallError(String),

    /// I/O error (files, network, etc.)
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON parsing error (video metadata, chapters, etc.)
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Regular expression compilation or execution error
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Configuration error (TOML file, invalid values, etc.)
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Generic error for cases not covered by other variants
    #[error("Generic error: {0}")]
    Other(String),
}

/// Type alias for `Result<T, YtcsError>`
///
/// Simplifies function signatures by using our custom error type.
pub type Result<T> = std::result::Result<T, YtcsError>;
