use crate::alerts::{Alert, AlertDetector, AlertHistory};
use crate::checker::{CheckResult, Status};
use crate::config::Config;
use crate::history::SiteHistory;
use crate::ui::theme::{ResponsiveLayout, Theme, ThemeName};
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use indexmap::IndexMap;
use ratatui::layout::Rect;
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
    Detail(String),      // Detail view for a specific site (by name)
    Alerts,              // Alert history view
    AlertDetail(usize),  // Alert detail view by index
    Help,                // Help screen showing keyboard shortcuts
}

/// Main application state
pub struct App {
    pub config: Config,
    pub theme: Theme,
    pub theme_name: ThemeName,
    pub sites: IndexMap<String, SiteHistory>,
    pub selected_index: Option<usize>,
    pub last_update: DateTime<Utc>,
    pub current_view: View,
    pub error_message: Option<String>,
    pub error_timestamp: Option<DateTime<Utc>>,
    force_refresh_tx: broadcast::Sender<()>,
    pub alert_history: AlertHistory,
    alert_detector: AlertDetector,
    pub alert_selected_index: Option<usize>,
}

impl App {
    /// Create a new App with the given configuration
    pub fn new(config: Config, force_refresh_tx: broadcast::Sender<()>) -> Self {
        let history_size = config.settings.history_size;
        let alert_history_size = config.settings.alerts.alert_history_size;
        let theme_name = config.settings.theme;
        let theme = Theme::from_name(theme_name);

        // Initialize empty history for each site
        let sites: IndexMap<String, SiteHistory> = config
            .sites
            .iter()
            .map(|site| (site.name.clone(), SiteHistory::new(history_size)))
            .collect();

        let alert_detector = AlertDetector::new(config.clone());
        let alert_history = AlertHistory::new(alert_history_size);

        Self {
            config,
            theme,
            theme_name,
            sites,
            selected_index: None,
            last_update: Utc::now(),
            current_view: View::Dashboard,
            error_message: None,
            error_timestamp: None,
            force_refresh_tx,
            alert_history,
            alert_detector,
            alert_selected_index: None,
        }
    }

    /// Handle a new check result
    pub fn handle_check_result(&mut self, site_name: String, result: CheckResult) -> Option<Alert> {
        // Get previous status from history (clone to avoid borrow conflicts)
        let previous_status = self
            .sites
            .get(&site_name)
            .and_then(|h| h.latest())
            .map(|r| r.status.clone());

        // Add result to history (existing logic)
        if let Some(history) = self.sites.get_mut(&site_name) {
            history.add_result(result.clone());
            self.last_update = Utc::now();
        }

        // Check if this should trigger an alert
        if let Some(transition) = self.alert_detector.evaluate(
            &site_name,
            previous_status.as_ref(),
            &result.status,
        ) {
            let alert = Alert::new(
                site_name,
                transition,
                previous_status.unwrap_or(crate::checker::Status::Up),
                result.status,
            );
            self.alert_history.add_alert(alert.clone());
            return Some(alert);
        }

        None
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

            // Alert history on 'a'
            KeyCode::Char('a') => {
                self.alert_selected_index = None;
                self.current_view = View::Alerts;
                AppAction::Continue
            }

            // ESC key - return to previous view
            KeyCode::Esc => {
                match self.current_view {
                    View::AlertDetail(_) => {
                        // Return to alerts list
                        self.current_view = View::Alerts;
                    }
                    _ => {
                        // Return to dashboard
                        self.selected_index = None;
                        self.alert_selected_index = None;
                        self.current_view = View::Dashboard;
                    }
                }
                AppAction::Continue
            }

            // Enter key - open detail view for selected site or alert
            KeyCode::Enter => {
                match self.current_view {
                    View::Dashboard => {
                        if let Some(name) = self.selected_site().map(|(name, _)| name.clone()) {
                            self.selected_index = None;
                            self.current_view = View::Detail(name);
                        }
                    }
                    View::Alerts => {
                        if let Some(index) = self.alert_selected_index {
                            self.current_view = View::AlertDetail(index);
                        }
                    }
                    _ => {}
                }
                AppAction::Continue
            }

            // Navigate up
            KeyCode::Up | KeyCode::Char('k') => {
                match self.current_view {
                    View::Dashboard => {
                        if self.sites.is_empty() {
                            return AppAction::Continue;
                        }
                        self.selected_index = Some(match self.selected_index {
                            None => 0,
                            Some(idx) if idx > 0 => idx - 1,
                            Some(_) => self.sites.len() - 1, // Wrap to bottom
                        });
                    }
                    View::Alerts => {
                        if self.alert_history.is_empty() {
                            return AppAction::Continue;
                        }
                        self.alert_selected_index = Some(match self.alert_selected_index {
                            None => 0,
                            Some(idx) if idx > 0 => idx - 1,
                            Some(_) => self.alert_history.len() - 1, // Wrap to bottom
                        });
                    }
                    _ => {}
                }
                AppAction::Continue
            }

            // Navigate down
            KeyCode::Down | KeyCode::Char('j') => {
                match self.current_view {
                    View::Dashboard => {
                        if self.sites.is_empty() {
                            return AppAction::Continue;
                        }
                        self.selected_index = Some(match self.selected_index {
                            None => 0,
                            Some(idx) if idx < self.sites.len() - 1 => idx + 1,
                            Some(_) => 0, // Wrap to top
                        });
                    }
                    View::Alerts => {
                        if self.alert_history.is_empty() {
                            return AppAction::Continue;
                        }
                        self.alert_selected_index = Some(match self.alert_selected_index {
                            None => 0,
                            Some(idx) if idx < self.alert_history.len() - 1 => idx + 1,
                            Some(_) => 0, // Wrap to top
                        });
                    }
                    _ => {}
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

            // Cycle through themes
            KeyCode::Char('t') => {
                self.cycle_theme();
                AppAction::Continue
            }

            _ => AppAction::Continue,
        }
    }

    /// Cycle to the next theme
    pub fn cycle_theme(&mut self) {
        self.theme_name = self.theme_name.next();
        self.theme = Theme::from_name(self.theme_name);
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

    /// Get counts of sites by status
    pub fn status_counts(&self) -> (usize, usize, usize, usize) {
        let mut up = 0;
        let mut down = 0;
        let mut warn = 0;
        let mut unknown = 0;

        for history in self.sites.values() {
            match history.latest().map(|r| &r.status) {
                Some(Status::Up) => up += 1,
                Some(Status::Down) => down += 1,
                Some(Status::Warning) => warn += 1,
                None => unknown += 1,
            }
        }

        (up, down, warn, unknown)
    }

    /// Handle mouse input
    pub fn handle_mouse_event(&mut self, mouse: MouseEvent, frame_size: Rect) {
        // Only handle left click down events
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }

        let row = mouse.row;
        let col = mouse.column;

        match self.current_view {
            View::Dashboard => {
                self.handle_dashboard_click(row, col, frame_size);
            }
            View::Alerts => {
                self.handle_alerts_click(row, col, frame_size);
            }
            _ => {}
        }
    }

    fn handle_dashboard_click(&mut self, row: u16, _col: u16, frame_size: Rect) {
        // Layout: Header(3) + Content(variable) + StatusBar(1) + optional Error(1) + Footer(1)
        let header_height = 3;
        let footer_height = if self.error_message.is_some() { 3 } else { 2 }; // status + footer + optional error
        let content_start = header_height;
        let content_end = frame_size.height.saturating_sub(footer_height);

        // Check if click is in the content area
        if row < content_start || row >= content_end {
            return;
        }

        // Calculate item index based on click position
        let responsive = ResponsiveLayout::new(frame_size.width);
        let lines_per_item = responsive.lines_per_site_item();
        let content_row = row - content_start - 1; // -1 for top border
        let clicked_index = (content_row / lines_per_item) as usize;

        if clicked_index < self.sites.len() {
            self.selected_index = Some(clicked_index);
        }
    }

    fn handle_alerts_click(&mut self, row: u16, _col: u16, frame_size: Rect) {
        // Layout: Header(3) + Alert list(variable) + Summary(3) + StatusBar(1) + Footer(1)
        let header_height = 3;
        let footer_height = 5; // summary(3) + status(1) + footer(1)
        let content_start = header_height;
        let content_end = frame_size.height.saturating_sub(footer_height);

        if row < content_start || row >= content_end {
            return;
        }

        // Each alert takes 2 lines
        let lines_per_item = 2;
        let content_row = row - content_start - 1;
        let clicked_index = (content_row / lines_per_item) as usize;

        if clicked_index < self.alert_history.len() {
            self.alert_selected_index = Some(clicked_index);
        }
    }
}
