//! Terminal User Interface (TUI) module
//!
//! This module provides an interactive terminal interface for ytcs with:
//! - Welcome screen with URL input
//! - Playlist video selection
//! - Download progress display
//! - Settings and quality/format selection
//! - Help and keyboard navigation

pub mod app;
pub mod download_manager;
pub mod layout;
pub mod presenter;
pub mod screens;

pub mod components {
    pub mod box_chars;
    pub mod input;
    pub mod keyboard;
    pub mod list;
    pub mod modal;
    pub mod progress;
    pub mod spinner;
}

pub use app::{App, DownloadResult, Screen, ScreenData};
pub use layout::{LayoutConstraints, LayoutEngine, TerminalCapabilities};
pub use presenter::TuiStylePresenter;
