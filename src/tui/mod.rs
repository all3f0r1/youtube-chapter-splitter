//! TUI (Text User Interface) module for interactive chapter selection and metadata editing.

pub mod app;
pub mod ui;
pub mod events;

pub use app::App;
pub use events::EventHandler;

use crate::error::Result;

/// Run the TUI application
pub fn run(app: App) -> Result<App> {
    app.run()
}
