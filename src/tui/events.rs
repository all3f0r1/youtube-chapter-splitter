//! Event handling for the TUI application.

use crossterm::event::{self, Event};
use std::time::Duration;

/// Event handler for the TUI
pub struct EventHandler {
    /// Timeout for polling events
    timeout: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(timeout_ms: u64) -> Self {
        Self {
            timeout: Duration::from_millis(timeout_ms),
        }
    }
    
    /// Poll for the next event
    pub fn next(&self) -> std::io::Result<Option<Event>> {
        if event::poll(self.timeout)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(100)
    }
}
