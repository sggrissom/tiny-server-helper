use crate::alerts::{Alert, AlertSeverity};
use crate::app::App;
use crate::checker::Status;
use crate::ui::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the alert detail view
pub fn render_alert_detail(frame: &mut Frame, app: &App, alert_index: usize) {
    // Convert reversed display index to original VecDeque index
    // Display shows alerts in reverse order (most recent first)
    // Index 0 in display = last alert in VecDeque (index N-1)
    let total_alerts = app.alert_history.len();
    let original_index = total_alerts.saturating_sub(1).saturating_sub(alert_index);

    // Try to retrieve the alert using the original index
    let alert = match app.alert_history.all_alerts().get(original_index) {
        Some(a) => a,
        None => {
            // Invalid index - show error message
            render_invalid_alert(frame, app);
            return;
        }
    };

    // Determine if we need an error bar
    let has_error = app.error_message.is_some();

    let constraints = if has_error {
        vec![
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Alert info & status change
            Constraint::Length(6),  // Timestamp & message
            Constraint::Min(5),     // Related site info
            Constraint::Length(1),  // Error bar
            Constraint::Length(1),  // Footer
        ]
    } else {
        vec![
            Constraint::Length(3),  // Header
            Constraint::Length(10), // Alert info & status change
            Constraint::Length(6),  // Timestamp & message
            Constraint::Min(5),     // Related site info
            Constraint::Length(1),  // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.size());

    render_header(frame, app, alert, chunks[0]);
    render_alert_info(frame, app, alert, chunks[1]);
    render_message(frame, app, alert, chunks[2]);
    render_site_info(frame, app, alert, chunks[3]);

    if has_error {
        render_error_bar(frame, app, chunks[4]);
        render_footer(frame, app, chunks[5]);
    } else {
        render_footer(frame, app, chunks[4]);
    }
}

/// Render error message when alert index is invalid
fn render_invalid_alert(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let error_text = "Alert not found. Press ESC to return to alerts list.";

    let error = Paragraph::new(error_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Error")
                .border_style(Style::default().fg(theme.alert_critical)),
        )
        .style(
            Style::default()
                .fg(theme.alert_critical)
                .add_modifier(Modifier::BOLD),
        );

    let area = centered_rect(60, 30, frame.size());
    frame.render_widget(error, area);
}

/// Render the header with alert severity and timestamp
fn render_header(frame: &mut Frame, app: &App, alert: &Alert, area: Rect) {
    let theme = &app.theme;

    let severity_str = match alert.severity {
        AlertSeverity::Critical => "CRITICAL ALERT",
        AlertSeverity::Warning => "WARNING ALERT",
        AlertSeverity::Recovery => "RECOVERY ALERT",
    };

    let severity_color = match alert.severity {
        AlertSeverity::Critical => theme.alert_critical,
        AlertSeverity::Warning => theme.alert_warning,
        AlertSeverity::Recovery => theme.alert_recovery,
    };

    let timestamp_str = alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string();
    let header_text = format!(
        "{}          {}          Press ESC to return",
        severity_str, timestamp_str
    );

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(
            Style::default()
                .fg(severity_color)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(header, area);
}

/// Render alert information and status change
fn render_alert_info(frame: &mut Frame, app: &App, alert: &Alert, area: Rect) {
    let theme = &app.theme;
    let (prev_color, prev_text) = status_display(theme, &alert.previous_status);
    let (curr_color, curr_text) = status_display(theme, &alert.current_status);

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Site: ",
                Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&alert.site_name, Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Status Change: ",
            Style::default()
                .fg(theme.text_primary)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                prev_text,
                Style::default()
                    .fg(prev_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  →  ", Style::default().fg(theme.text_muted)),
            Span::styled(
                curr_text,
                Style::default()
                    .fg(curr_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let info = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Alert Information")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(info, area);
}

/// Render alert message section
fn render_message(frame: &mut Frame, app: &App, alert: &Alert, area: Rect) {
    let theme = &app.theme;

    let lines = vec![
        Line::from(vec![
            Span::styled(
                "Timestamp: ",
                Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                alert.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                Style::default().fg(theme.text_secondary),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Message: ",
                Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(&alert.message, Style::default().fg(theme.text_secondary)),
        ]),
    ];

    let message = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Details")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(message, area);
}

/// Render related site information
fn render_site_info(frame: &mut Frame, app: &App, alert: &Alert, area: Rect) {
    let theme = &app.theme;

    let lines = if let Some(history) = app.sites.get(&alert.site_name) {
        let uptime = history.uptime_percentage();
        let avg_response = history.avg_response_time();

        let avg_response_str = if let Some(avg) = avg_response {
            format!("{}ms", avg)
        } else {
            "N/A".to_string()
        };

        vec![
            Line::from(vec![Span::styled(
                "Related Site Statistics:",
                Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Uptime: ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{:.1}%", uptime), Style::default().fg(theme.status_up)),
            ]),
            Line::from(vec![
                Span::styled("  Avg Response Time: ", Style::default().fg(theme.text_secondary)),
                Span::styled(avg_response_str, Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  Total Checks: ", Style::default().fg(theme.text_secondary)),
                Span::styled(format!("{}", history.len()), Style::default().fg(theme.text_primary)),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled(
                "Related Site Statistics:",
                Style::default()
                    .fg(theme.text_primary)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "  Site data not available",
                Style::default().fg(theme.text_muted),
            )]),
        ]
    };

    let site_info = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Site Context")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(site_info, area);
}

/// Render error bar (if error exists)
fn render_error_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    if let Some(error_msg) = &app.error_message {
        let error = Paragraph::new(format!(" {} ", error_msg)).style(theme.error_style());
        frame.render_widget(error, area);
    }
}

/// Render footer with keyboard shortcuts
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let footer_text = " ESC: Back to Alerts | a: Alerts List | r: Refresh | ?/h: Help | q: Quit";
    let footer = Paragraph::new(footer_text).style(Style::default().fg(theme.footer_fg));
    frame.render_widget(footer, area);
}

/// Helper function to get status display color and text
fn status_display(theme: &Theme, status: &Status) -> (ratatui::style::Color, &'static str) {
    match status {
        Status::Up => (theme.status_up, "UP"),
        Status::Down => (theme.status_down, "DOWN"),
        Status::Warning => (theme.status_warning, "WARNING"),
    }
}

/// Helper function to create a centered rect
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
