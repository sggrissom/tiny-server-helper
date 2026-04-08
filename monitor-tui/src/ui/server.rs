use crate::app::App;
use crate::metrics_poller::MetricsSnapshot;
use crate::ui::status_bar::render_status_bar;
use chrono::Utc;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_server(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(frame.size());

    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);
    render_footer(frame, app, chunks[3]);
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let url = app
        .config
        .server_metrics
        .as_ref()
        .map(|c| c.url.as_str())
        .unwrap_or("(not configured)");

    let mut spans = vec![
        Span::styled(
            " Server Metrics  ",
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(url.to_string(), Style::default().fg(theme.text_secondary)),
    ];

    if let Some(snapshot) = &app.server_metrics {
        if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(&snapshot.collected_at) {
            let local_time = ts
                .with_timezone(&chrono::Local)
                .format("%H:%M:%S")
                .to_string();
            spans.push(Span::styled("  Last: ", Style::default().fg(theme.text_muted)));
            spans.push(Span::styled(
                local_time,
                Style::default().fg(theme.text_primary),
            ));

            if let Some(config) = &app.config.server_metrics {
                let age = Utc::now().signed_duration_since(ts.with_timezone(&Utc));
                if age.num_seconds() > (config.poll_interval * 2) as i64 {
                    spans.push(Span::styled(
                        "  STALE",
                        Style::default()
                            .fg(theme.status_warning)
                            .add_modifier(Modifier::BOLD),
                    ));
                }
            }
        }
    }

    let header = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(header, area);
}

fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    if app.config.server_metrics.is_none() {
        let msg = Paragraph::new(
            "Server metrics not configured. Add [server_metrics] to sites.toml.",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border_fg)),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_muted));
        frame.render_widget(msg, area);
        return;
    }

    if app.server_metrics.is_none() {
        let (text, style) = if let Some(err) = &app.server_metrics_error {
            (
                err.as_str(),
                Style::default().fg(theme.status_down),
            )
        } else {
            (
                "Connecting to metrics server...",
                Style::default().fg(theme.text_muted),
            )
        };
        let msg = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_fg)),
            )
            .alignment(Alignment::Center)
            .style(style);
        frame.render_widget(msg, area);
        return;
    }

    let snapshot = app.server_metrics.as_ref().unwrap();

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    render_system_panel(frame, app, panels[0], snapshot);
    render_apps_panel(frame, app, panels[1], snapshot);
}

fn fill_bar(pct: f64, width: usize) -> String {
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn render_system_panel(frame: &mut Frame, app: &App, area: Rect, snapshot: &MetricsSnapshot) {
    let theme = &app.theme;
    let bar_width = 10usize;
    let sys = &snapshot.system;

    let mem_bar = fill_bar(sys.memory.used_pct, bar_width);
    let disk_bar = fill_bar(sys.disk.used_pct, bar_width);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" Load:  ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!(
                    "{:.2} {:.2} {:.2}",
                    sys.load_avg.one, sys.load_avg.five, sys.load_avg.fifteen
                ),
                Style::default().fg(theme.text_primary),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Mem:   ", Style::default().fg(theme.text_muted)),
            Span::styled(mem_bar, Style::default().fg(theme.status_warning)),
            Span::styled(
                format!("  {:.0}%", sys.memory.used_pct),
                Style::default().fg(theme.text_primary),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" CPU:   ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{:.0}% usr", sys.cpu.user_pct),
                Style::default().fg(theme.text_primary),
            ),
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:.0}% sys", sys.cpu.system_pct),
                Style::default().fg(theme.text_secondary),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Disk:  ", Style::default().fg(theme.text_muted)),
            Span::styled(disk_bar, Style::default().fg(theme.status_warning)),
            Span::styled(
                format!("  {:.0}%", sys.disk.used_pct),
                Style::default().fg(theme.text_primary),
            ),
        ]),
    ];

    if let Some(err) = &app.server_metrics_error {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!(" ⚠ {}", err),
            Style::default().fg(theme.status_down),
        )]));
    }

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("System")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    frame.render_widget(paragraph, area);
}

fn render_apps_panel(frame: &mut Frame, app: &App, area: Rect, snapshot: &MetricsSnapshot) {
    let theme = &app.theme;

    if snapshot.apps.is_empty() {
        let msg = Paragraph::new("No apps found")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Apps")
                    .border_style(Style::default().fg(theme.border_fg)),
            )
            .alignment(Alignment::Center)
            .style(Style::default().fg(theme.text_muted));
        frame.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = snapshot
        .apps
        .iter()
        .enumerate()
        .map(|(idx, app_metrics)| {
            let disk_mb = app_metrics.disk_kb as f64 / 1024.0;

            let line1 = Line::from(vec![
                Span::styled(
                    format!("{:<20}", app_metrics.name),
                    Style::default()
                        .fg(theme.text_primary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{:6.1} MB", disk_mb),
                    Style::default().fg(theme.text_secondary),
                ),
            ]);

            let traffic = &app_metrics.traffic;
            let err_color = if traffic.error_pct > 5.0 {
                theme.status_down
            } else if traffic.error_pct > 0.0 {
                theme.status_warning
            } else {
                theme.text_muted
            };

            let line2 = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    format!("{:.1} req/min", traffic.requests_per_min),
                    Style::default().fg(theme.text_secondary),
                ),
                Span::styled("  5xx: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!("{:.1}%", traffic.error_pct),
                    Style::default().fg(err_color),
                ),
                Span::styled("  4xx: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    format!("{}", traffic.error_4xx),
                    Style::default().fg(theme.text_secondary),
                ),
            ]);

            let style = if app.server_selected_index == idx {
                theme.selection_style()
            } else {
                Style::default()
            };

            ListItem::new(vec![line1, line2]).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Apps")
            .border_style(Style::default().fg(theme.border_fg)),
    );

    let mut state = ListState::default();
    let clamped = app
        .server_selected_index
        .min(snapshot.apps.len().saturating_sub(1));
    state.select(Some(clamped));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let footer =
        Paragraph::new(" s: Refresh | ↑↓/jk: Scroll | ESC: Dashboard | q: Quit")
            .style(Style::default().fg(theme.footer_fg));
    frame.render_widget(footer, area);
}
