//! Centralized error handling with TUI modal support
//!
//! This module provides unified error handling across the application,
//! with support for both CLI and TUI error presentation.

use std::fmt;

/// Application-level errors that can be displayed to users
#[derive(Debug, Clone)]
pub enum AppError {
    /// Dependency missing - can be resolved
    DependencyMissing {
        name: String,
        install_suggestion: String,
    },

    /// Download failed - may be retryable
    DownloadFailed {
        url: String,
        reason: String,
        can_retry: bool,
    },

    /// Authentication required - can prompt for cookies
    AuthenticationRequired { url: String },

    /// Terminal too small for TUI
    TerminalTooSmall {
        current: (u16, u16),
        required: (u16, u16),
    },

    /// Configuration error
    ConfigError {
        key: String,
        value: String,
        reason: String,
    },

    /// Generic error with message
    Generic {
        message: String,
        suggestion: Option<String>,
    },
}

impl AppError {
    /// Returns user-friendly error message
    pub fn display(&self) -> String {
        match self {
            AppError::DependencyMissing { name, .. } => {
                format!("⚠ Missing dependency: {}", name)
            }
            AppError::DownloadFailed { reason, .. } => {
                format!("❌ Download failed: {}", reason)
            }
            AppError::AuthenticationRequired { .. } => {
                "🔒 This video requires authentication (member-only content)".to_string()
            }
            AppError::TerminalTooSmall { current, required } => {
                format!(
                    "⚠ Terminal too small ({}×{}). Minimum: {}×{}",
                    current.0, current.1, required.0, required.1
                )
            }
            AppError::ConfigError { key, reason, .. } => {
                format!("Configuration error for '{}': {}", key, reason)
            }
            AppError::Generic { message, .. } => message.clone(),
        }
    }

    /// Returns detailed explanation of what happened
    pub fn explanation(&self) -> String {
        match self {
            AppError::DependencyMissing { name, .. } => {
                format!("The tool requires {} to download YouTube videos.", name)
            }
            AppError::DownloadFailed { reason, .. } => {
                format!("The download could not complete: {}", reason)
            }
            AppError::AuthenticationRequired { .. } => {
                "YouTube members-only content cannot be accessed without login.".to_string()
            }
            AppError::TerminalTooSmall { .. } => {
                "The TUI requires a minimum terminal size to display properly.".to_string()
            }
            AppError::ConfigError { reason, .. } => reason.clone(),
            AppError::Generic { message, .. } => message.clone(),
        }
    }

    /// Returns suggested action (if any)
    pub fn suggestion(&self) -> Option<String> {
        match self {
            AppError::DependencyMissing {
                install_suggestion, ..
            } => Some(install_suggestion.clone()),
            AppError::DownloadFailed {
                can_retry: true, ..
            } => Some("Press 'r' to retry or 'q' to quit".to_string()),
            AppError::DownloadFailed {
                can_retry: false, ..
            } => Some("Press 'q' to quit".to_string()),
            AppError::AuthenticationRequired { .. } => {
                Some("Configure cookies from browser: Press 'c' for cookie setup".to_string())
            }
            AppError::TerminalTooSmall { .. } => {
                Some("Resize your terminal to at least 40×12 characters".to_string())
            }
            AppError::ConfigError { .. } => {
                Some("Check your configuration file or run 'ytcs reset'".to_string())
            }
            AppError::Generic { suggestion, .. } => suggestion.clone(),
        }
    }

    /// Whether error should show as modal (blocking)
    pub fn is_modal(&self) -> bool {
        matches!(
            self,
            AppError::AuthenticationRequired { .. }
                | AppError::TerminalTooSmall { .. }
                | AppError::ConfigError { .. }
        )
    }

    /// Whether error is critical (requires explicit dismissal)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            AppError::TerminalTooSmall { .. } | AppError::ConfigError { .. }
        )
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::error::Error for AppError {}

/// Error message template for dependency missing
pub fn dependency_missing_error(name: &str, platform: &str) -> AppError {
    let suggestion = match platform {
        "linux" => "  Ubuntu/Debian: sudo apt install yt-dlp ffmpeg\n\
              Fedora: sudo dnf install yt-dlp ffmpeg\n\
             Arch: sudo pacman -S yt-dlp ffmpeg"
            .to_string(),
        "macos" => "  brew install yt-dlp ffmpeg".to_string(),
        "windows" => "  winget install yt-dlp ffmpeg\n\
             Or use pip: python -m pip install yt-dlp"
            .to_string(),
        _ => "  Install using your system package manager".to_string(),
    };

    AppError::DependencyMissing {
        name: name.to_string(),
        install_suggestion: format!("Install manually:\n{}", suggestion),
    }
}

/// Error message template for authentication required
pub fn auth_required_error() -> AppError {
    AppError::AuthenticationRequired { url: String::new() }
}

/// Error message template for terminal too small
pub fn terminal_too_small_error(current: (u16, u16)) -> AppError {
    AppError::TerminalTooSmall {
        current,
        required: (40, 12),
    }
}

/// Error message template for download failed
pub fn download_failed_error(reason: &str, can_retry: bool) -> AppError {
    AppError::DownloadFailed {
        url: String::new(),
        reason: reason.to_string(),
        can_retry,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_missing_display() {
        let err = AppError::DependencyMissing {
            name: "yt-dlp".to_string(),
            install_suggestion: "sudo apt install yt-dlp".to_string(),
        };
        assert!(err.display().contains("yt-dlp"));
        assert!(err.suggestion().is_some());
    }

    #[test]
    fn test_terminal_too_small() {
        let err = AppError::TerminalTooSmall {
            current: (30, 10),
            required: (40, 12),
        };
        assert!(err.is_modal());
        assert!(err.is_critical());
    }
}
