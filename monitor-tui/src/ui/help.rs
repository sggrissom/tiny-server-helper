use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// Render the help screen
pub fn render_help(frame: &mut Frame) {
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

    render_help_content(frame, help_area);
}

/// Render the help content
fn render_help_content(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Monitor TUI - Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Global Commands",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  q", Style::default().fg(Color::Green)),
            Span::raw("         Quit the application"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C", Style::default().fg(Color::Green)),
            Span::raw("    Exit the application"),
        ]),
        Line::from(vec![
            Span::styled("  ?  or  h", Style::default().fg(Color::Green)),
            Span::raw("  Show this help screen"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Green)),
            Span::raw("       Return to dashboard"),
        ]),
        Line::from(vec![
            Span::styled("  a", Style::default().fg(Color::Green)),
            Span::raw("         View alert history"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Dashboard View",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ↑  or  k", Style::default().fg(Color::Green)),
            Span::raw("  Navigate up (select previous site)"),
        ]),
        Line::from(vec![
            Span::styled("  ↓  or  j", Style::default().fg(Color::Green)),
            Span::raw("  Navigate down (select next site)"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green)),
            Span::raw("     Open detail view for selected site"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Green)),
            Span::raw("         Force refresh all sites immediately"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Detail View",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Green)),
            Span::raw("       Return to dashboard"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Green)),
            Span::raw("         Force refresh all sites immediately"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Alert History View",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ↑  or  k", Style::default().fg(Color::Green)),
            Span::raw("  Navigate up (previous alert)"),
        ]),
        Line::from(vec![
            Span::styled("  ↓  or  j", Style::default().fg(Color::Green)),
            Span::raw("  Navigate down (next alert)"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green)),
            Span::raw("     View details for selected alert"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Green)),
            Span::raw("       Return to dashboard"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Alert Detail View",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Green)),
            Span::raw("       Return to alert history"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Status Indicators",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::Green)),
            Span::styled("UP  ", Style::default().fg(Color::Green)),
            Span::raw("    Site is responding with expected HTTP status"),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::Red)),
            Span::styled("DOWN", Style::default().fg(Color::Red)),
            Span::raw("    Site is not responding or connection failed"),
        ]),
        Line::from(vec![
            Span::styled("  ● ", Style::default().fg(Color::Yellow)),
            Span::styled("WARN", Style::default().fg(Color::Yellow)),
            Span::raw("    Site is responding but with wrong HTTP status"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Configuration",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from("  Config file locations (in priority order):"),
        Line::from("    1. ./sites.toml"),
        Line::from("    2. ~/.config/monitor/sites.toml"),
        Line::from("    3. /etc/monitor/sites.toml"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Press ESC to return to dashboard",
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            ),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" Help "),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}
