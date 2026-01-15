use crate::app::App;
use crate::checker::Status;
use crate::history::SiteHistory;
use crate::ui::status_bar::render_status_bar;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, Paragraph},
    Frame,
};

/// Render the site detail view
pub fn render_detail(frame: &mut Frame, app: &App, site_name: &str) {
    let has_error = app.error_message.is_some();

    let constraints = if has_error {
        vec![
            Constraint::Length(3),  // Header
            Constraint::Length(8),  // Site info & current status
            Constraint::Length(5),  // Statistics
            Constraint::Min(10),    // Chart
            Constraint::Length(8),  // Recent checks
            Constraint::Length(1),  // Status bar
            Constraint::Length(1),  // Error bar
            Constraint::Length(1),  // Footer
        ]
    } else {
        vec![
            Constraint::Length(3),  // Header
            Constraint::Length(8),  // Site info & current status
            Constraint::Length(5),  // Statistics
            Constraint::Min(10),    // Chart
            Constraint::Length(8),  // Recent checks
            Constraint::Length(1),  // Status bar
            Constraint::Length(1),  // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.size());

    render_header(frame, app, site_name, chunks[0]);

    if let Some(history) = app.sites.get(site_name) {
        let site_config = app.config.sites.iter().find(|s| &s.name == site_name);
        if let Some(config) = site_config {
            render_site_info(frame, app, config, history, chunks[1]);
            render_statistics(frame, app, history, chunks[2]);
            render_chart(frame, app, history, chunks[3]);
            render_recent_checks(frame, app, history, chunks[4]);
        }
    }

    if has_error {
        render_status_bar(frame, app, chunks[5]);
        render_error_bar(frame, app, chunks[6]);
        render_footer(frame, app, chunks[7]);
    } else {
        render_status_bar(frame, app, chunks[5]);
        render_footer(frame, app, chunks[6]);
    }
}

/// Render the header
fn render_header(frame: &mut Frame, app: &App, site_name: &str, area: Rect) {
    let theme = &app.theme;
    let header_text = format!("Site Details: {}          Press ESC to return", site_name);

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .style(theme.header_style());

    frame.render_widget(header, area);
}

/// Render site configuration and current status
fn render_site_info(
    frame: &mut Frame,
    app: &App,
    config: &crate::config::SiteConfig,
    history: &SiteHistory,
    area: Rect,
) {
    let theme = &app.theme;
    let latest = history.latest();

    // Determine status color and text
    let (status_color, status_text) = if let Some(result) = latest {
        let color = theme.status_color(&result.status);
        let text = match result.status {
            Status::Up => "UP",
            Status::Down => "DOWN",
            Status::Warning => "WARNING",
        };
        (color, text)
    } else {
        (theme.status_unknown, "NO DATA")
    };

    let last_checked = latest
        .map(|r| r.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Never".to_string());

    let response_time = latest
        .and_then(|r| r.response_time_ms)
        .map(|ms| format!("{}ms", ms))
        .unwrap_or_else(|| "N/A".to_string());

    let http_status = latest
        .and_then(|r| r.http_status)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let check_interval = config
        .check_interval
        .map(|i| format!("{}s", i))
        .unwrap_or_else(|| "default".to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("URL: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(&config.url, Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(vec![
            Span::styled("Expected Status: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(config.expected_status.to_string(), Style::default().fg(theme.text_secondary)),
            Span::styled("  |  ", Style::default().fg(theme.text_muted)),
            Span::styled("Check Interval: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(check_interval, Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Current Status: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Last Checked: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(last_checked, Style::default().fg(theme.text_secondary)),
        ]),
        Line::from(vec![
            Span::styled("Response Time: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(response_time, Style::default().fg(theme.text_secondary)),
            Span::styled("  |  ", Style::default().fg(theme.text_muted)),
            Span::styled("HTTP Status: ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
            Span::styled(http_status, Style::default().fg(theme.text_secondary)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Configuration & Status")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(paragraph, area);
}

/// Render statistics
fn render_statistics(frame: &mut Frame, app: &App, history: &SiteHistory, area: Rect) {
    let theme = &app.theme;

    let uptime = if !history.is_empty() {
        format!("{:.1}%", history.uptime_percentage())
    } else {
        "N/A".to_string()
    };

    let avg_response = history
        .avg_response_time()
        .map(|ms| format!("{}ms", ms))
        .unwrap_or_else(|| "N/A".to_string());

    let min_response = history
        .min_response_time()
        .map(|ms| format!("{}ms", ms))
        .unwrap_or_else(|| "N/A".to_string());

    let max_response = history
        .max_response_time()
        .map(|ms| format!("{}ms", ms))
        .unwrap_or_else(|| "N/A".to_string());

    let total_checks = history.len();

    let lines = vec![
        Line::from(vec![Span::styled(
            format!("Statistics (Last {} checks)", total_checks),
            Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  Uptime: ", Style::default().fg(theme.text_secondary)),
            Span::styled(uptime, Style::default().fg(theme.status_up)),
        ]),
        Line::from(vec![
            Span::styled("  Response Times: ", Style::default().fg(theme.text_secondary)),
            Span::styled(
                format!("Avg: {}  |  Min: {}  |  Max: {}", avg_response, min_response, max_response),
                Style::default().fg(theme.text_primary),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(paragraph, area);
}

/// Render response time chart
fn render_chart(frame: &mut Frame, app: &App, history: &SiteHistory, area: Rect) {
    let theme = &app.theme;
    let chart_data = history.chart_data();

    if chart_data.is_empty() {
        let paragraph = Paragraph::new("No data available for chart")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Response Time History")
                    .border_style(Style::default().fg(theme.border_fg)),
            )
            .style(Style::default().fg(theme.text_muted));
        frame.render_widget(paragraph, area);
        return;
    }

    // Find bounds for axes
    let (min_time, max_time) = chart_data
        .iter()
        .fold((f64::MAX, f64::MIN), |(min_t, max_t), (t, _)| {
            (min_t.min(*t), max_t.max(*t))
        });

    let (min_response, max_response) = chart_data
        .iter()
        .fold((f64::MAX, f64::MIN), |(min_r, max_r), (_, r)| {
            (min_r.min(*r), max_r.max(*r))
        });

    // Add some padding to the y-axis
    let y_min = (min_response * 0.9).max(0.0);
    let y_max = max_response * 1.1;

    let dataset = Dataset::default()
        .name("Response Time")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.chart_line))
        .data(&chart_data);

    let x_axis = Axis::default()
        .style(Style::default().fg(theme.chart_axis))
        .bounds([min_time, max_time]);

    let y_axis = Axis::default()
        .style(Style::default().fg(theme.chart_axis))
        .bounds([y_min, y_max])
        .labels(vec![
            Span::styled(format!("{:.0}ms", y_min), Style::default().fg(theme.text_secondary)),
            Span::styled(format!("{:.0}ms", (y_min + y_max) / 2.0), Style::default().fg(theme.text_secondary)),
            Span::styled(format!("{:.0}ms", y_max), Style::default().fg(theme.text_secondary)),
        ]);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Response Time History")
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}

/// Render recent checks list
fn render_recent_checks(frame: &mut Frame, app: &App, history: &SiteHistory, area: Rect) {
    let theme = &app.theme;
    let results = history.all_results();

    let items: Vec<ListItem> = results
        .iter()
        .rev()
        .take(5)
        .map(|result| {
            let status_symbol = Span::styled("●", Style::default().fg(theme.status_color(&result.status)));

            let timestamp = result.timestamp.format("%H:%M:%S").to_string();

            let response = result
                .response_time_ms
                .map(|ms| format!("{}ms", ms))
                .unwrap_or_else(|| "--".to_string());

            let http = result
                .http_status
                .map(|s| s.to_string())
                .unwrap_or_else(|| "--".to_string());

            let error = result
                .error_message
                .as_ref()
                .map(|e| {
                    let truncated = if e.len() > 40 {
                        format!("{}...", &e[..37])
                    } else {
                        e.clone()
                    };
                    format!(" | {}", truncated)
                })
                .unwrap_or_default();

            let line = Line::from(vec![
                status_symbol,
                Span::styled(
                    format!(" {} | {:>6} | HTTP {:>3}{}", timestamp, response, http, error),
                    Style::default().fg(theme.text_primary),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Recent Checks")
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

/// Render the footer
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let footer = Paragraph::new(" ESC: Back to Dashboard | a: Alerts | r: Refresh | ?/h: Help | q: Quit")
        .style(Style::default().fg(theme.footer_fg));

    frame.render_widget(footer, area);
}
