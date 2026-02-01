//! Dependency detection and installation module
//!
//! This module handles detection of required external dependencies (yt-dlp, ffmpeg)
//! and provides platform-specific installation methods.

pub mod detect;
pub mod install;

pub use detect::{DependencyStatus, InstallMethod, Platform};
pub use install::DependencyInstaller;

/// Overall dependency state for the application
#[derive(Debug, Clone)]
pub struct DependencyState {
    pub ytdlp: DependencyStatus,
    pub ffmpeg: DependencyStatus,
}

impl DependencyState {
    /// Check all dependencies and return their status
    pub fn check_all() -> Self {
        Self {
            ytdlp: DependencyStatus::check("yt-dlp"),
            ffmpeg: DependencyStatus::check("ffmpeg"),
        }
    }

    /// Returns true if all required dependencies are installed
    pub fn all_present(&self) -> bool {
        self.ytdlp.installed && self.ffmpeg.installed
    }

    /// Returns a list of missing dependency names
    pub fn missing(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !self.ytdlp.installed {
            missing.push("yt-dlp");
        }
        if !self.ffmpeg.installed {
            missing.push("ffmpeg");
        }
        missing
    }
}
