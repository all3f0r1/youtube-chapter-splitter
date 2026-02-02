//! List components for TUI
//!
//! Provides selectable list widgets for settings navigation, playlists, etc.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

/// State for a selectable list
pub struct SelectableList {
    /// Items to display
    pub items: Vec<String>,
    /// Currently selected index
    pub selected: Option<usize>,
    /// Scroll offset
    pub offset: usize,
    /// Title for the list
    pub title: String,
    /// Whether to show borders
    pub bordered: bool,
}

impl Default for SelectableList {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectableList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            offset: 0,
            title: String::new(),
            bordered: true,
        }
    }

    /// Builder: set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder: set whether bordered
    pub fn with_bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Builder: set the items
    pub fn with_items(mut self, items: Vec<String>) -> Self {
        self.items = items;
        if !self.items.is_empty() && self.selected.is_none() {
            self.selected = Some(0);
        }
        self
    }

    /// Set items from string slice
    pub fn items_from(mut self, items: &[&str]) -> Self {
        self.items = items.iter().map(|s| s.to_string()).collect();
        if !self.items.is_empty() {
            self.selected = Some(0);
        }
        self
    }

    /// Get the number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get the selected index
    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Set the selected index
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = Some(index);
        }
    }

    /// Move selection up
    pub fn select_previous(&mut self) -> bool {
        match self.selected {
            Some(0) => false,
            Some(i) => {
                self.selected = Some(i - 1);
                true
            }
            None => {
                if !self.items.is_empty() {
                    self.selected = Some(self.items.len() - 1);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) -> bool {
        match self.selected {
            Some(i) if i >= self.items.len().saturating_sub(1) => false,
            Some(i) => {
                self.selected = Some(i + 1);
                true
            }
            None => {
                if !self.items.is_empty() {
                    self.selected = Some(0);
                    true
                } else {
                    false
                }
            }
        }
    }

    /// Select first item
    pub fn select_first(&mut self) {
        if !self.items.is_empty() {
            self.selected = Some(0);
        }
    }

    /// Select last item
    pub fn select_last(&mut self) {
        if !self.items.is_empty() {
            self.selected = Some(self.items.len() - 1);
        }
    }

    /// Draw the list
    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        // Calculate visible area
        let inner_height = if self.bordered {
            area.height.saturating_sub(2) // Account for borders
        } else {
            area.height
        };

        // Adjust offset to keep selection in view
        if let Some(selected) = self.selected {
            let selected = selected as u16;
            if selected < self.offset as u16 {
                self.offset = selected as usize;
            } else if selected >= self.offset as u16 + inner_height {
                self.offset = (selected - inner_height + 1) as usize;
            }
        }

        // Convert strings to ListItems
        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .map(|s| ListItem::new(s.as_str()))
            .collect();

        // Build the list widget
        let list = List::new(list_items)
            .block(if self.bordered {
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.as_str())
                    .border_style(Style::default().fg(Color::Rgb(100, 150, 200)))
            } else {
                Block::default()
            })
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(50, 50, 100))
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(self.selected);

        f.render_stateful_widget(list, area, &mut state);
    }
}

/// Key-value list for displaying settings
pub struct KeyValueList {
    pub items: Vec<(String, String)>,
    pub selected: Option<usize>,
    pub offset: usize,
    pub title: String,
}

impl Default for KeyValueList {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyValueList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            offset: 0,
            title: String::new(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_items(mut self, items: Vec<(String, String)>) -> Self {
        self.items = items;
        if !self.items.is_empty() {
            self.selected = Some(0);
        }
        self
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn select_previous(&mut self) -> bool {
        match self.selected {
            Some(0) => false,
            Some(i) => {
                self.selected = Some(i - 1);
                true
            }
            None => {
                if !self.items.is_empty() {
                    self.selected = Some(self.items.len() - 1);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn select_next(&mut self) -> bool {
        match self.selected {
            Some(i) if i >= self.items.len().saturating_sub(1) => false,
            Some(i) => {
                self.selected = Some(i + 1);
                true
            }
            None => {
                if !self.items.is_empty() {
                    self.selected = Some(0);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let inner_height = area.height.saturating_sub(2);

        // Adjust offset
        if let Some(selected) = self.selected {
            let selected = selected as u16;
            if selected < self.offset as u16 {
                self.offset = selected as usize;
            } else if selected >= self.offset as u16 + inner_height {
                self.offset = (selected - inner_height + 1) as usize;
            }
        }

        let list_items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, (key, value))| {
                let is_selected = self.selected == Some(i);
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}: ", key), style),
                    Span::styled(value, Style::default().fg(Color::Rgb(180, 180, 180))),
                ]))
            })
            .collect();

        let list = List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(self.title.as_str())
                .border_style(Style::default().fg(Color::Rgb(100, 150, 200))),
        );

        let mut state = ListState::default();
        state.select(self.selected);

        f.render_stateful_widget(list, area, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selectable_list_new() {
        let list = SelectableList::new();
        assert!(list.is_empty());
        assert_eq!(list.selected(), None);
    }

    #[test]
    fn test_selectable_list_with_items() {
        let list = SelectableList::new().with_items(vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
        ]);

        assert_eq!(list.len(), 3);
        assert_eq!(list.selected, Some(0));
    }

    #[test]
    fn test_selectable_list_navigation() {
        let mut list = SelectableList::new().with_items(vec![
            "Item 1".to_string(),
            "Item 2".to_string(),
            "Item 3".to_string(),
        ]);

        assert!(list.select_next());
        assert_eq!(list.selected(), Some(1));

        assert!(list.select_next());
        assert_eq!(list.selected(), Some(2));

        assert!(!list.select_next()); // Already at last
        assert_eq!(list.selected(), Some(2));

        assert!(list.select_previous());
        assert_eq!(list.selected(), Some(1));

        assert!(list.select_previous());
        assert_eq!(list.selected(), Some(0));

        assert!(!list.select_previous()); // Already at first
        assert_eq!(list.selected(), Some(0));
    }

    #[test]
    fn test_key_value_list() {
        let list = KeyValueList::new().with_items(vec![
            ("Key 1".to_string(), "Value 1".to_string()),
            ("Key 2".to_string(), "Value 2".to_string()),
        ]);

        assert_eq!(list.len(), 2);
        assert_eq!(list.selected, Some(0));
    }
}
