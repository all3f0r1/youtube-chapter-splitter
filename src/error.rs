use thiserror::Error;

#[derive(Error, Debug)]
pub enum YtcsError {
    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("Audio processing error: {0}")]
    AudioError(String),

    #[error("Chapter parsing error: {0}")]
    ChapterError(String),

    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),

    #[error("Missing system tool: {0}")]
    MissingTool(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Generic error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, YtcsError>;
