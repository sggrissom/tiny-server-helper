mod app;
mod checker;
mod config;
mod history;
mod ui;

use app::{App, AppAction, View};
use checker::spawn_checker_task;
use config::Config;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, watch};

/// RAII guard to ensure terminal is properly restored on drop
struct TerminalCleanup;

impl TerminalCleanup {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Create channels for communication
    let (tx, mut rx) = mpsc::channel(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (force_refresh_tx, _) = broadcast::channel(16);

    // Initialize app state with force refresh sender
    let mut app = App::new(config.clone(), force_refresh_tx.clone());

    // Spawn health checker tasks
    let mut tasks = Vec::new();
    for site in config.sites.clone() {
        let handle = spawn_checker_task(
            site,
            tx.clone(),
            shutdown_rx.clone(),
            force_refresh_tx.subscribe(),
            config.settings.request_timeout,
            config.settings.refresh_interval,
        );
        tasks.push(handle);
    }

    // Drop original tx so channel closes when all tasks finish
    drop(tx);

    // Set up terminal
    let _cleanup = TerminalCleanup::new()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Main event loop
    loop {
        // Check if error should be auto-dismissed
        app.check_error_dismissal();

        // Render UI based on current view
        terminal.draw(|frame| {
            match &app.current_view {
                View::Dashboard => ui::dashboard::render_dashboard(frame, &app),
                View::Detail(site_name) => ui::detail::render_detail(frame, &app, site_name),
                View::Help => ui::help::render_help(frame),
            }
        })?;

        // Poll for keyboard events with timeout (~60 FPS)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match app.handle_key_event(key) {
                    AppAction::Quit => break,
                    AppAction::Continue => {}
                }
            }
        }

        // Check for new health check results (non-blocking)
        while let Ok((site_name, result)) = rx.try_recv() {
            app.handle_check_result(site_name, result);
        }
    }

    // Graceful shutdown
    let _ = shutdown_tx.send(true);
    for task in tasks {
        let _ = task.await;
    }

    Ok(())
}
