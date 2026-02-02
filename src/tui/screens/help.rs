//! Help screen for the TUI
//!
//! Displays help information and keyboard shortcuts.

use crate::config::Config;
use crate::tui::app::{Screen, ScreenData};
use crate::tui::screens::ScreenResult;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Help screen
pub struct HelpScreen {
    scroll_offset: usize,
}

impl HelpScreen {
    pub fn new() -> Self {
        Self { scroll_offset: 0 }
    }

    pub fn draw(&mut self, f: &mut Frame, _data: &ScreenData, _config: &Config) {
        let size = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(0),    // Content
                Constraint::Length(4), // Footer
            ])
            .split(size);

        // Title
        let title = Paragraph::new("Keyboard Shortcuts & Help")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Content with sections
        let content = vec![
            Line::from(""),
            Line::from("═══ GLOBAL KEYBINDINGS ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Q or "),
                Span::styled(
                    "Ctrl+C",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("               Quit application"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                         Go back / Cancel current action"),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from("═══ DOWNLOAD SCREEN ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Tab",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                         Move to next field"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Shift+Tab",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                     Move to previous field"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                       Start download (on URL field)"),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from("═══ PLAYLIST SCREEN ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "↑",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" or "),
                Span::styled("k", Style::default().fg(Color::Green)),
                Span::raw("               Move up"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "↓",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" or "),
                Span::styled("j", Style::default().fg(Color::Green)),
                Span::raw("               Move down"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Space",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                        Toggle video selection"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("A", Style::default().fg(Color::Green)),
                Span::raw("                            Select all videos"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("D", Style::default().fg(Color::Green)),
                Span::raw("                            Deselect all videos"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                       Download selected videos"),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from("═══ PROGRESS SCREEN ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from("  Downloads are processed automatically"),
            Line::from("  Press "),
            Line::from(vec![
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to view results when complete"),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from("═══ SETTINGS SCREEN ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "↑↓",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" or "),
                Span::styled("jk", Style::default().fg(Color::Green)),
                Span::raw("              Navigate settings"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("                       Edit setting ("),
                Span::styled("←→", Style::default().fg(Color::Yellow)),
                Span::raw(" for enums)"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("S", Style::default().fg(Color::Green)),
                Span::raw("                            Save changes"),
            ]),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("R", Style::default().fg(Color::Green)),
                Span::raw("                            Reset to defaults"),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from("═══ TIPS ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from("  • Playlist URLs are auto-detected"),
            Line::from("  • Empty artist/album fields use auto-detection"),
            Line::from("  • Settings are saved to ~/.config/ytcs/config.toml"),
            Line::from("  • Use Page Up/Down for faster list navigation"),
            Line::from(""),
            Line::from(""),
            Line::from("═══ SUPPORT ═══").style(Style::default().fg(Color::Cyan)),
            Line::from(""),
            Line::from("  GitHub: https://github.com/all3f0r1/youtube-chapter-splitter"),
            Line::from("  Issues: https://github.com/all3f0r1/youtube-chapter-splitter/issues"),
            Line::from(""),
            Line::from(""),
            Line::from("").style(Style::default().fg(Color::Rgb(100, 100, 100))),
        ];

        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .scroll((self.scroll_offset as u16, self.scroll_offset as u16));

        f.render_widget(paragraph, chunks[1]);
    }

    pub fn handle_key(&mut self, key: KeyEvent, _data: &mut ScreenData) -> ScreenResult {
        match key.code {
            KeyCode::Esc => ScreenResult::NavigateTo(Screen::Welcome),
            KeyCode::Char('q') | KeyCode::Char('Q') => ScreenResult::Quit,
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                ScreenResult::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset += 1;
                ScreenResult::Continue
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(10);
                ScreenResult::Continue
            }
            KeyCode::PageDown => {
                self.scroll_offset += 10;
                ScreenResult::Continue
            }
            KeyCode::Home => {
                self.scroll_offset = 0;
                ScreenResult::Continue
            }
            _ => ScreenResult::Continue,
        }
    }
}

impl Default for HelpScreen {
    fn default() -> Self {
        Self::new()
    }
}
