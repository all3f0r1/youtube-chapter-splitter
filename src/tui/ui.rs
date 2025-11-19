//! UI rendering for the TUI application.

use crate::tui::app::{App, AppMode};
use crate::utils::format_duration;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the entire UI
pub fn draw(f: &mut Frame, app: &App) {
    match app.mode {
        AppMode::ChapterSelection => draw_chapter_selection(f, app),
        AppMode::MetadataEdit => draw_metadata_edit(f, app),
        AppMode::Downloading => draw_downloading(f, app),
        AppMode::Splitting => draw_splitting(f, app),
        AppMode::Confirmation => draw_confirmation(f, app),
        AppMode::Help => draw_help(f, app),
    }
}

/// Draw chapter selection screen
fn draw_chapter_selection(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Header
            Constraint::Min(10),    // Chapters list
            Constraint::Length(3),  // Status bar
            Constraint::Length(3),  // Controls
        ])
        .split(f.area());
    
    // Header
    draw_header(f, chunks[0], app);
    
    // Chapters list
    draw_chapters_list(f, chunks[1], app);
    
    // Status bar
    draw_status_bar(f, chunks[2], app);
    
    // Controls
    draw_controls(f, chunks[3]);
}

/// Draw header with video info
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("Video: ", Style::default().fg(Color::Cyan)),
            Span::raw(&app.video_info.title),
        ]),
        Line::from(vec![
            Span::styled("Artist: ", Style::default().fg(Color::Green)),
            Span::raw(&app.artist),
            Span::raw("  │  "),
            Span::styled("Album: ", Style::default().fg(Color::Green)),
            Span::raw(&app.album),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Color::Yellow)),
            Span::raw(format_duration(app.video_info.duration)),
            Span::raw("  │  "),
            Span::styled("Chapters: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", app.chapters.len())),
        ]),
    ];
    
    let header = Paragraph::new(header_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" YouTube Chapter Splitter ")
                .title_alignment(Alignment::Center)
                .border_style(Style::default().fg(Color::Cyan))
        )
        .wrap(Wrap { trim: true });
    
    f.render_widget(header, area);
}

/// Draw chapters list
fn draw_chapters_list(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app.chapters
        .iter()
        .enumerate()
        .map(|(i, chapter)| {
            let checkbox = if app.selected_chapters[i] { "[✓]" } else { "[ ]" };
            let is_selected = i == app.current_selection;
            
            let line = format!(
                "{} {:2}. {:<40} ({} - {})",
                checkbox,
                i + 1,
                truncate(&chapter.title, 40),
                format_duration(chapter.start_time),
                format_duration(chapter.end_time)
            );
            
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if app.selected_chapters[i] {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            
            ListItem::new(line).style(style)
        })
        .collect();
    
    let selected_count = app.selected_chapters.iter().filter(|&&s| s).count();
    let title = format!(
        " Chapters ({} selected, {} total) ",
        selected_count,
        app.chapters.len()
    );
    
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::White))
        );
    
    f.render_widget(list, area);
}

/// Draw status bar
fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let selected_count = app.selected_chapters.iter().filter(|&&s| s).count();
    let total_duration: f64 = app.chapters
        .iter()
        .zip(app.selected_chapters.iter())
        .filter(|(_, &selected)| selected)
        .map(|(chapter, _)| chapter.duration())
        .sum();
    
    let status_text = format!(
        " Selected: {}/{} tracks  │  Total duration: {}  │  Output: {} ",
        selected_count,
        app.chapters.len(),
        format_duration(total_duration),
        app.output_dir.display()
    );
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(status, area);
}

/// Draw controls help
fn draw_controls(f: &mut Frame, area: Rect) {
    let controls = vec![
        Line::from(vec![
            Span::styled("[↑↓/jk]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Navigate  "),
            Span::styled("[Space]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Toggle  "),
            Span::styled("[a]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" All  "),
            Span::styled("[n]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" None  "),
            Span::styled("[i]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" Invert  "),
            Span::styled("[e]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Edit  "),
            Span::styled("[s/Enter]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Start  "),
            Span::styled("[?]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(" Help  "),
            Span::styled("[q/Esc]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" Quit"),
        ]),
    ];
    
    let paragraph = Paragraph::new(controls)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(paragraph, area);
}

/// Draw metadata edit screen
fn draw_metadata_edit(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Edit fields
            Constraint::Length(3),  // Controls
        ])
        .split(f.area());
    
    // Title
    let title = Paragraph::new("Edit Metadata")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    // Edit fields
    let mut items = vec![
        format!("Artist:  {}", if app.edit_field_index == 0 { &app.edit_buffer } else { &app.artist }),
        format!("Album:   {}", if app.edit_field_index == 1 { &app.edit_buffer } else { &app.album }),
        String::new(),
        "Track Titles:".to_string(),
    ];
    
    for (i, chapter) in app.chapters.iter().enumerate() {
        let field_idx = i + 2;
        let text = if app.edit_field_index == field_idx {
            &app.edit_buffer
        } else {
            &chapter.title
        };
        
        let prefix = if app.edit_field_index == field_idx { "> " } else { "  " };
        items.push(format!("{}{:2}. {}", prefix, i + 1, text));
    }
    
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let style = if i == app.edit_field_index || (i >= 4 && i - 4 == app.edit_field_index - 2) {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(text.as_str()).style(style)
        })
        .collect();
    
    let list = List::new(list_items)
        .block(Block::default().borders(Borders::ALL).title(" Fields "));
    
    f.render_widget(list, chunks[1]);
    
    // Controls
    let controls = Paragraph::new("[Tab] Next  [Shift+Tab] Previous  [Enter] Save & Next  [Esc] Cancel")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(controls, chunks[2]);
}

/// Draw downloading screen
fn draw_downloading(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(f.area());
    
    let title = Paragraph::new("Downloading...")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Progress "))
        .gauge_style(Style::default().fg(Color::Green))
        .percent((app.download_progress * 100.0) as u16);
    f.render_widget(gauge, chunks[1]);
    
    draw_logs(f, chunks[2], app);
}

/// Draw splitting screen
fn draw_splitting(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(5),
        ])
        .split(f.area());
    
    let title = Paragraph::new("Splitting Audio...")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);
    
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(" Overall Progress "))
        .gauge_style(Style::default().fg(Color::Green))
        .percent((app.split_progress * 100.0) as u16);
    f.render_widget(gauge, chunks[1]);
    
    if let Some(ref track) = app.current_track {
        let current = Paragraph::new(format!("Current: {}", track))
            .block(Block::default().borders(Borders::ALL).title(" Current Track "));
        f.render_widget(current, chunks[2]);
    }
    
    draw_logs(f, chunks[3], app);
}

/// Draw logs
fn draw_logs(f: &mut Frame, area: Rect, app: &App) {
    let log_items: Vec<ListItem> = app.logs
        .iter()
        .rev()
        .take(area.height.saturating_sub(2) as usize)
        .rev()
        .map(|log| ListItem::new(log.as_str()))
        .collect();
    
    let logs = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).title(" Logs "));
    
    f.render_widget(logs, area);
}

/// Draw confirmation dialog
fn draw_confirmation(f: &mut Frame, app: &App) {
    let selected_count = app.selected_chapters.iter().filter(|&&s| s).count();
    
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Start download and split?", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(format!("Selected chapters: {}", selected_count)),
        Line::from(format!("Artist: {}", app.artist)),
        Line::from(format!("Album: {}", app.album)),
        Line::from(format!("Output: {}", app.output_dir.display())),
        Line::from(""),
        Line::from(vec![
            Span::styled("[Y]", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(" Yes  "),
            Span::styled("[N]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(" No"),
        ]),
    ];
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Confirmation ")
        .border_style(Style::default().fg(Color::Yellow));
    
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    
    let area = centered_rect(60, 50, f.area());
    f.render_widget(paragraph, area);
}

/// Draw help screen
fn draw_help(f: &mut Frame, _app: &App) {
    let help_text = vec![
        Line::from(vec![
            Span::styled("YouTube Chapter Splitter - Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ↑/↓ or j/k    - Move up/down"),
        Line::from("  Space         - Toggle chapter selection"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Selection:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  a             - Select all chapters"),
        Line::from("  n             - Select none"),
        Line::from("  i             - Invert selection"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  e             - Edit metadata"),
        Line::from("  s or Enter    - Start download"),
        Line::from("  ?             - Show this help"),
        Line::from("  q or Esc      - Quit"),
        Line::from(""),
        Line::from("Press any key to close this help screen"),
    ];
    
    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Help "))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    
    let area = centered_rect(70, 70, f.area());
    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Truncate string to max length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
