mod app;
mod checker;
mod config;
mod history;
mod ui;

use app::{App, AppAction};
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
use tokio::sync::{mpsc, watch};

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

    // Initialize app state
    let mut app = App::new(config.clone());

    // Create channels for communication
    let (tx, mut rx) = mpsc::channel(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Spawn health checker tasks
    let mut tasks = Vec::new();
    for site in config.sites.clone() {
        let handle = spawn_checker_task(
            site,
            tx.clone(),
            shutdown_rx.clone(),
            config.settings.request_timeout,
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
        // Render UI
        terminal.draw(|frame| {
            ui::dashboard::render_dashboard(frame, &app);
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
