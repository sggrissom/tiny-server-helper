use crate::alerts::AlertSeverity;
use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_alerts(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Alert list
            Constraint::Length(3), // Summary stats
            Constraint::Length(1), // Footer
        ])
        .split(frame.size());

    render_header(frame, app, chunks[0]);
    render_alert_list(frame, app, chunks[1]);
    render_alert_summary(frame, app, chunks[2]);
    render_footer(frame, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let total_alerts = app.alert_history.len();
    let header_text = format!(
        "Alert History ({} total)          Press ESC to return",
        total_alerts
    );

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

    frame.render_widget(header, area);
}

fn render_alert_list(frame: &mut Frame, app: &App, area: Rect) {
    let alerts = app.alert_history.all_alerts();

    if alerts.is_empty() {
        let empty_message = Paragraph::new("No alerts yet")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Alerts (Most Recent First)"),
            )
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_message, area);
        return;
    }

    let items: Vec<ListItem> = alerts
        .iter()
        .rev() // Most recent first
        .enumerate()
        .map(|(idx, alert)| {
            // Color based on severity
            let (color, severity_text) = match alert.severity {
                AlertSeverity::Critical => (Color::Red, "CRITICAL"),
                AlertSeverity::Warning => (Color::Yellow, "WARNING "),
                AlertSeverity::Recovery => (Color::Green, "RECOVERY"),
            };

            let timestamp = alert.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

            let line1 = Line::from(vec![
                Span::styled("● ", Style::default().fg(color)),
                Span::styled(
                    format!("{:8}", severity_text),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" {} - {}", timestamp, alert.site_name)),
            ]);

            let line2 = Line::from(format!("  {}", alert.message));

            // Apply selection highlighting
            let style = if app.alert_selected_index == Some(idx) {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(vec![line1, line2]).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Alerts (Most Recent First)"),
    );

    frame.render_widget(list, area);
}

fn render_alert_summary(frame: &mut Frame, app: &App, area: Rect) {
    let critical_count = app
        .alert_history
        .by_severity(AlertSeverity::Critical)
        .len();
    let warning_count = app.alert_history.by_severity(AlertSeverity::Warning).len();
    let recovery_count = app
        .alert_history
        .by_severity(AlertSeverity::Recovery)
        .len();

    let summary = Line::from(vec![
        Span::raw("Summary:  "),
        Span::styled(
            format!("{} Critical", critical_count),
            Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("{} Warning", warning_count),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  |  "),
        Span::styled(
            format!("{} Recovery", recovery_count),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let paragraph = Paragraph::new(summary).block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Paragraph::new(
        "↑↓: Navigate | ESC: Dashboard | a: Alert History | r: Refresh | ?/h: Help | q: Quit",
    )
    .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, area);
}
