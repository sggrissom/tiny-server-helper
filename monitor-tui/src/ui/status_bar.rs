use crate::app::App;
use chrono::Local;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let (up, down, warn, _unknown) = app.status_counts();
    let total = app.sites.len();

    let last_update = app.last_update.with_timezone(&Local).format("%H:%M:%S");

    let line = Line::from(vec![
        Span::styled(
            format!(" {}/{} UP", up, total),
            Style::default().fg(if up == total {
                theme.status_up
            } else if down > 0 {
                theme.status_down
            } else {
                theme.status_warning
            }),
        ),
        Span::styled(" | ", Style::default().fg(theme.text_muted)),
        Span::styled(
            format!("{} DOWN", down),
            Style::default().fg(if down > 0 {
                theme.status_down
            } else {
                theme.text_muted
            }),
        ),
        Span::styled(" | ", Style::default().fg(theme.text_muted)),
        Span::styled(
            format!("{} WARN", warn),
            Style::default().fg(if warn > 0 {
                theme.status_warning
            } else {
                theme.text_muted
            }),
        ),
        Span::styled("  |  ", Style::default().fg(theme.text_muted)),
        Span::styled("Last: ", Style::default().fg(theme.text_secondary)),
        Span::styled(
            format!("{}", last_update),
            Style::default().fg(theme.text_primary),
        ),
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().fg(theme.text_primary));

    frame.render_widget(paragraph, area);
}
