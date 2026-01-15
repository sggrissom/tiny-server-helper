use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the help screen
pub fn render_help(frame: &mut Frame, app: &App) {
    let area = frame.size();

    // Create a centered area for the help content
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(area);

    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(vertical_chunks[1]);

    let help_area = horizontal_chunks[1];

    render_help_content(frame, app, help_area);
}

/// Render the help content
fn render_help_content(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Monitor TUI - Keyboard Shortcuts",
            Style::default()
                .fg(theme.header_fg)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Global Commands",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(theme.status_up)),
            Span::styled("         Quit the application", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C", Style::default().fg(theme.status_up)),
            Span::styled("    Exit the application", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ?  or  h", Style::default().fg(theme.status_up)),
            Span::styled("  Show this help screen", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(theme.status_up)),
            Span::styled("       Return to dashboard", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  a", Style::default().fg(theme.status_up)),
            Span::styled("         View alert history", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  t", Style::default().fg(theme.status_up)),
            Span::styled("         Cycle theme (Dark/Light/High-Contrast)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Dashboard View",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ↑  or  k", Style::default().fg(theme.status_up)),
            Span::styled("  Navigate up (select previous site)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ↓  or  j", Style::default().fg(theme.status_up)),
            Span::styled("  Navigate down (select next site)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(theme.status_up)),
            Span::styled("     Open detail view for selected site", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(theme.status_up)),
            Span::styled("         Force refresh all sites immediately", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Detail View",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(theme.status_up)),
            Span::styled("       Return to dashboard", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(theme.status_up)),
            Span::styled("         Force refresh all sites immediately", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Alert History View",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ↑  or  k", Style::default().fg(theme.status_up)),
            Span::styled("  Navigate up (previous alert)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ↓  or  j", Style::default().fg(theme.status_up)),
            Span::styled("  Navigate down (next alert)", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(theme.status_up)),
            Span::styled("     View details for selected alert", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(theme.status_up)),
            Span::styled("       Return to dashboard", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Alert Detail View",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(theme.status_up)),
            Span::styled("       Return to alert history", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Status Indicators",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme.status_up)),
            Span::styled("UP  ", Style::default().fg(theme.status_up)),
            Span::styled("    Site is responding with expected HTTP status", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme.status_down)),
            Span::styled("DOWN", Style::default().fg(theme.status_down)),
            Span::styled("    Site is not responding or connection failed", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(theme.status_warning)),
            Span::styled("WARN", Style::default().fg(theme.status_warning)),
            Span::styled("    Site is responding but with wrong HTTP status", Style::default().fg(theme.text_primary)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Configuration",
            Style::default()
                .fg(theme.status_warning)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            "  Config file locations (in priority order):",
            Style::default().fg(theme.text_primary),
        )]),
        Line::from(vec![Span::styled(
            "    1. ./sites.toml",
            Style::default().fg(theme.text_secondary),
        )]),
        Line::from(vec![Span::styled(
            "    2. ~/.config/monitor/sites.toml",
            Style::default().fg(theme.text_secondary),
        )]),
        Line::from(vec![Span::styled(
            "    3. /etc/monitor/sites.toml",
            Style::default().fg(theme.text_secondary),
        )]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press ESC to return to dashboard",
            Style::default()
                .fg(theme.footer_fg)
                .add_modifier(Modifier::ITALIC),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.header_fg))
                .title(" Help "),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
