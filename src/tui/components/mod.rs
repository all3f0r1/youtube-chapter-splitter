//! Reusable TUI components
//!
//! Common UI components used across multiple screens.

pub mod box_chars;
pub mod input;
pub mod keyboard;
pub mod list;
pub mod modal;
pub mod progress;
pub mod spinner;
pub mod style;

pub use box_chars::{BoxChars, BorderStyle};
pub use input::TextInput;
pub use keyboard::KeyBindings;
pub use list::{KeyValueList, SelectableList};
pub use modal::Modal;
pub use progress::ProgressBar;
pub use spinner::Spinner;
pub use style::{AccessibleStyle, ColorMode, Symbols};
