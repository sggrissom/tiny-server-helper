use crate::app::App;
use crate::checker::Status;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the main dashboard view
pub fn render_dashboard(frame: &mut Frame, app: &App) {
    // Determine if we need an error bar
    let has_error = app.error_message.is_some();

    let constraints = if has_error {
        vec![
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Error bar
            Constraint::Length(1), // Footer
        ]
    } else {
        vec![
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.size());

    render_header(frame, app, chunks[0]);
    render_site_list(frame, app, chunks[1]);

    if has_error {
        render_error_bar(frame, app, chunks[2]);
        render_footer(frame, chunks[3]);
    } else {
        render_footer(frame, chunks[2]);
    }
}

/// Render the header with title and last update time
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let last_update_str = app.last_update.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let header_text = format!("Monitor TUI          Last Update: {}", last_update_str);

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));

    frame.render_widget(header, area);
}

/// Render the list of sites with their status
fn render_site_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .sites
        .iter()
        .enumerate()
        .map(|(idx, (site_name, history))| {
            let latest = history.latest();

            // Determine status color and text
            let (status_color, status_text) = if let Some(result) = latest {
                let color = match result.status {
                    Status::Up => Color::Green,
                    Status::Down => Color::Red,
                    Status::Warning => Color::Yellow,
                };
                let text = match result.status {
                    Status::Up => "UP  ",
                    Status::Down => "DOWN",
                    Status::Warning => "WARN",
                };
                (color, text)
            } else {
                (Color::Gray, "----")
            };

            // Get metrics
            let response_time_str = latest
                .and_then(|r| r.response_time_ms)
                .map(|ms| format!("{}ms", ms))
                .unwrap_or_else(|| "--".to_string());

            let http_status_str = latest
                .and_then(|r| r.http_status)
                .map(|s| s.to_string())
                .unwrap_or_else(|| "--".to_string());

            let uptime = if history.len() > 0 {
                format!("{:.1}%", history.uptime_percentage())
            } else {
                "--".to_string()
            };

            // Get URL from config
            let url = app
                .config
                .sites
                .iter()
                .find(|s| &s.name == site_name)
                .map(|s| s.url.as_str())
                .unwrap_or("");

            // Build the display text
            let line1 = Line::from(vec![
                Span::styled("● ", Style::default().fg(status_color)),
                Span::raw(format!("{:40}", site_name)),
                Span::styled(
                    format!("{:4}", status_text),
                    Style::default().fg(status_color),
                ),
            ]);

            let line2 = Line::from(format!("  {}", url));

            let line3 = Line::from(format!(
                "  Response: {:>6}  |  HTTP: {:>3}  |  Uptime: {:>5}",
                response_time_str, http_status_str, uptime
            ));

            // Get sparkline data (last 30 data points)
            let sparkline_data = history.recent_response_times(30);
            let sparkline_str = if !sparkline_data.is_empty() {
                // Create a simple ASCII sparkline using Unicode block characters
                let max_val = sparkline_data.iter().max().unwrap_or(&1);
                let min_val = sparkline_data.iter().min().unwrap_or(&0);
                let range = max_val.saturating_sub(*min_val).max(1);

                let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
                let sparkline: String = sparkline_data
                    .iter()
                    .map(|&val| {
                        let normalized = if range == 0 {
                            0
                        } else {
                            ((val.saturating_sub(*min_val)) * (chars.len() as u64 - 1) / range) as usize
                        };
                        chars[normalized.min(chars.len() - 1)]
                    })
                    .collect();
                format!("  Last checks: {}", sparkline)
            } else {
                "  Last checks: (no data)".to_string()
            };

            let line4 = Line::from(sparkline_str);

            // Apply selection highlighting
            let style = if app.selected_index == Some(idx) {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(vec![line1, line2, line3, line4]).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Sites"));

    frame.render_widget(list, area);
}

/// Render the error status bar
fn render_error_bar(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(error_msg) = &app.error_message {
        let error_text = format!("⚠ {}", error_msg);
        let error_bar = Paragraph::new(error_text)
            .style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(error_bar, area);
    }
}

/// Render the footer with keyboard shortcuts
fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new("↑↓: Navigate | Enter: Details | a: Alerts | r: Refresh | ?/h: Help | q: Quit")
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}
