//! TUI screen implementations
//!
//! Each screen represents a distinct view in the TUI application.

use crate::tui::app::Screen;
use crossterm::event::KeyEvent;

pub mod download;
pub mod help;
pub mod playlist;
pub mod progress;
pub mod settings;
pub mod summary;
pub mod welcome;

pub use download::DownloadScreen;
pub use help::HelpScreen;
pub use playlist::PlaylistScreen;
pub use progress::ProgressScreen;
pub use settings::SettingsScreen;
pub use summary::SummaryScreen;
pub use welcome::WelcomeScreen;

/// Result type for screen navigation
pub enum ScreenResult {
    Continue,
    NavigateTo(Screen),
    Quit,
}
