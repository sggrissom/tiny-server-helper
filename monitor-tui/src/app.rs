use crate::checker::CheckResult;
use crate::config::Config;
use crate::history::SiteHistory;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use indexmap::IndexMap;

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
}

/// Main application state
pub struct App {
    pub config: Config,
    pub sites: IndexMap<String, SiteHistory>,
    pub selected_index: usize,
    pub last_update: DateTime<Utc>,
    pub current_view: View,
}

impl App {
    /// Create a new App with the given configuration
    pub fn new(config: Config) -> Self {
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
            selected_index: 0,
            last_update: Utc::now(),
            current_view: View::Dashboard,
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

            // ESC key - return to dashboard
            KeyCode::Esc => {
                self.current_view = View::Dashboard;
                AppAction::Continue
            }

            // Enter key - open detail view for selected site
            KeyCode::Enter => {
                if self.current_view == View::Dashboard {
                    if let Some((name, _)) = self.selected_site() {
                        self.current_view = View::Detail(name.clone());
                    }
                }
                AppAction::Continue
            }

            // Navigate up (only in dashboard view)
            KeyCode::Up => {
                if self.current_view == View::Dashboard {
                    if self.sites.is_empty() {
                        return AppAction::Continue;
                    }
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        // Wrap around to bottom
                        self.selected_index = self.sites.len() - 1;
                    }
                }
                AppAction::Continue
            }

            // Navigate down (only in dashboard view)
            KeyCode::Down => {
                if self.current_view == View::Dashboard {
                    if self.sites.is_empty() {
                        return AppAction::Continue;
                    }
                    if self.selected_index < self.sites.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        // Wrap around to top
                        self.selected_index = 0;
                    }
                }
                AppAction::Continue
            }

            _ => AppAction::Continue,
        }
    }

    /// Get the currently selected site
    pub fn selected_site(&self) -> Option<(&String, &SiteHistory)> {
        self.sites.iter().nth(self.selected_index)
    }
}
