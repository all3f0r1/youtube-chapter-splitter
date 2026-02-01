//! Keyboard key bindings for TUI
//!
//! This module will handle customizable keyboard shortcuts.

pub struct KeyBindings {
    pub quit: String,
    pub help: String,
    pub confirm: String,
}

impl KeyBindings {
    pub fn from_config() -> Self {
        // Load from config - for now use defaults
        Self {
            quit: "q".to_string(),
            help: "?".to_string(),
            confirm: "Enter".to_string(),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::from_config()
    }
}
