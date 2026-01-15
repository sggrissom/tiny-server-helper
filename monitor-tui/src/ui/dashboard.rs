use crate::app::App;
use crate::ui::status_bar::render_status_bar;
use crate::ui::theme::ResponsiveLayout;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the main dashboard view
pub fn render_dashboard(frame: &mut Frame, app: &App) {
    let has_error = app.error_message.is_some();

    let constraints = if has_error {
        vec![
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
            Constraint::Length(1), // Error bar
            Constraint::Length(1), // Footer
        ]
    } else {
        vec![
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
            Constraint::Length(1), // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.size());

    render_header(frame, app, chunks[0]);
    render_site_list(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    if has_error {
        render_error_bar(frame, app, chunks[3]);
        render_footer(frame, app, chunks[4]);
    } else {
        render_footer(frame, app, chunks[3]);
    }
}

/// Render the header with title and last update time
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let last_update_str = app.last_update.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let header_text = format!("Monitor TUI          Last Update: {}", last_update_str);

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(theme.header_style());

    frame.render_widget(header, area);
}

/// Render the list of sites with their status
fn render_site_list(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let responsive = ResponsiveLayout::new(area.width);

    let items: Vec<ListItem> = app
        .sites
        .iter()
        .enumerate()
        .map(|(idx, (site_name, history))| {
            let latest = history.latest();

            // Determine status color and text
            let (status_color, status_text) = if let Some(result) = latest {
                let color = theme.status_color(&result.status);
                let text = match result.status {
                    crate::checker::Status::Up => "UP  ",
                    crate::checker::Status::Down => "DOWN",
                    crate::checker::Status::Warning => "WARN",
                };
                (color, text)
            } else {
                (theme.status_unknown, "----")
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

            // Truncate based on terminal width
            let name_width = responsive.site_name_width();
            let display_name = ResponsiveLayout::truncate(site_name, name_width);
            let display_url = ResponsiveLayout::truncate(url, responsive.url_max_len());

            // Build the display lines
            let line1 = Line::from(vec![
                Span::styled("● ", Style::default().fg(status_color)),
                Span::raw(format!("{:width$}", display_name, width = name_width)),
                Span::styled(
                    format!("{:4}", status_text),
                    Style::default().fg(status_color),
                ),
            ]);

            let line2 = Line::from(vec![
                Span::styled(format!("  {}", display_url), Style::default().fg(theme.text_secondary)),
            ]);

            let mut lines = vec![line1, line2];

            // Add metrics line if width allows
            if responsive.show_detailed_metrics() {
                let line3 = Line::from(vec![
                    Span::styled("  Response: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{:>6}", response_time_str), Style::default().fg(theme.text_primary)),
                    Span::styled("  |  HTTP: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{:>3}", http_status_str), Style::default().fg(theme.text_primary)),
                    Span::styled("  |  Uptime: ", Style::default().fg(theme.text_muted)),
                    Span::styled(format!("{:>5}", uptime), Style::default().fg(theme.text_primary)),
                ]);
                lines.push(line3);
            }

            // Add sparkline if width allows
            if responsive.show_sparkline() {
                let sparkline_data = history.recent_response_times(30);
                let sparkline_str = if !sparkline_data.is_empty() {
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

                lines.push(Line::from(vec![
                    Span::styled(sparkline_str, Style::default().fg(theme.text_secondary)),
                ]));
            }

            // Apply selection highlighting
            let style = if app.selected_index == Some(idx) {
                theme.selection_style()
            } else {
                Style::default()
            };

            ListItem::new(lines).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Sites")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(list, area);
}

/// Render the error status bar
fn render_error_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    if let Some(error_msg) = &app.error_message {
        let error_text = format!(" {} ", error_msg);
        let error_bar = Paragraph::new(error_text).style(theme.error_style());
        frame.render_widget(error_bar, area);
    }
}

/// Render the footer with keyboard shortcuts
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let footer =
        Paragraph::new(" ↑↓: Navigate | Enter: Details | a: Alerts | r: Refresh | ?/h: Help | q: Quit")
            .style(Style::default().fg(theme.footer_fg));

    frame.render_widget(footer, area);
}
