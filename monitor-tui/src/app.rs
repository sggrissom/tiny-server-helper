use crate::checker::CheckResult;
use crate::config::Config;
use crate::history::SiteHistory;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use indexmap::IndexMap;
use tokio::sync::broadcast;

/// Actions that can result from handling events
pub enum AppAction {
    Continue, // Keep running
    Quit,     // Exit application
}

/// Current view state
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    Detail(String), // Detail view for a specific site (by name)
    Help,           // Help screen showing keyboard shortcuts
}

/// Main application state
pub struct App {
    pub config: Config,
    pub sites: IndexMap<String, SiteHistory>,
    pub selected_index: Option<usize>,
    pub last_update: DateTime<Utc>,
    pub current_view: View,
    pub error_message: Option<String>,
    pub error_timestamp: Option<DateTime<Utc>>,
    force_refresh_tx: broadcast::Sender<()>,
}

impl App {
    /// Create a new App with the given configuration
    pub fn new(config: Config, force_refresh_tx: broadcast::Sender<()>) -> Self {
        let history_size = config.settings.history_size;

        // Initialize empty history for each site
        let sites: IndexMap<String, SiteHistory> = config
            .sites
            .iter()
            .map(|site| (site.name.clone(), SiteHistory::new(history_size)))
            .collect();

        Self {
            config,
            sites,
            selected_index: None,
            last_update: Utc::now(),
            current_view: View::Dashboard,
            error_message: None,
            error_timestamp: None,
            force_refresh_tx,
        }
    }

    /// Handle a new check result
    pub fn handle_check_result(&mut self, site_name: String, result: CheckResult) {
        if let Some(history) = self.sites.get_mut(&site_name) {
            history.add_result(result);
            self.last_update = Utc::now();
        }
    }

    /// Handle keyboard input
    pub fn handle_key_event(&mut self, key: KeyEvent) -> AppAction {
        match key.code {
            // Quit on 'q' or Ctrl+C
            KeyCode::Char('q') => AppAction::Quit,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => AppAction::Quit,

            // Help screen on '?' or 'h'
            KeyCode::Char('?') | KeyCode::Char('h') => {
                self.selected_index = None;
                self.current_view = View::Help;
                AppAction::Continue
            }

            // ESC key - return to dashboard
            KeyCode::Esc => {
                self.selected_index = None;
                self.current_view = View::Dashboard;
                AppAction::Continue
            }

            // Enter key - open detail view for selected site
            KeyCode::Enter => {
                if self.current_view == View::Dashboard {
                    if let Some(name) = self.selected_site().map(|(name, _)| name.clone()) {
                        self.selected_index = None;
                        self.current_view = View::Detail(name);
                    }
                }
                AppAction::Continue
            }

            // Navigate up (only in dashboard view)
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_view == View::Dashboard {
                    if self.sites.is_empty() {
                        return AppAction::Continue;
                    }
                    self.selected_index = Some(match self.selected_index {
                        None => 0,
                        Some(idx) if idx > 0 => idx - 1,
                        Some(_) => self.sites.len() - 1, // Wrap to bottom
                    });
                }
                AppAction::Continue
            }

            // Navigate down (only in dashboard view)
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_view == View::Dashboard {
                    if self.sites.is_empty() {
                        return AppAction::Continue;
                    }
                    self.selected_index = Some(match self.selected_index {
                        None => 0,
                        Some(idx) if idx < self.sites.len() - 1 => idx + 1,
                        Some(_) => 0, // Wrap to top
                    });
                }
                AppAction::Continue
            }

            // Force refresh all sites
            KeyCode::Char('r') => {
                // Send broadcast to all checker tasks
                // Ignore errors (no receivers is fine)
                let _ = self.force_refresh_tx.send(());
                AppAction::Continue
            }

            _ => AppAction::Continue,
        }
    }

    /// Get the currently selected site
    pub fn selected_site(&self) -> Option<(&String, &SiteHistory)> {
        self.selected_index.and_then(|idx| self.sites.iter().nth(idx))
    }

    /// Set an error message (reserved for future use)
    #[allow(dead_code)]
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.error_timestamp = Some(Utc::now());
    }

    /// Clear the error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
        self.error_timestamp = None;
    }

    /// Check if error should be auto-dismissed (after 5 seconds)
    pub fn check_error_dismissal(&mut self) {
        if let Some(timestamp) = self.error_timestamp {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(timestamp);
            if elapsed.num_seconds() >= 5 {
                self.clear_error();
            }
        }
    }
}
