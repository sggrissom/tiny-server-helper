use crate::alerts::{Alert, AlertSeverity};
use crate::config::Config;
use notify_rust::{Notification, Urgency};
use std::io::{self, Write};

#[derive(Clone)]
pub struct AlertNotifier {
    config: Config,
}

impl AlertNotifier {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn notify(&self, alert: &Alert) {
        let site_config = self.config.sites.iter().find(|s| s.name == alert.site_name);
        let global_alerts = &self.config.settings.alerts;
        let site_alerts = site_config.and_then(|s| s.alerts.as_ref());

        // Determine which notification methods to use
        let terminal_bell = site_alerts
            .and_then(|a| a.terminal_bell)
            .unwrap_or(global_alerts.terminal_bell);

        let desktop_notifications = site_alerts
            .and_then(|a| a.desktop_notifications)
            .unwrap_or(global_alerts.desktop_notifications);

        // Send terminal bell
        if terminal_bell {
            self.send_terminal_bell();
        }

        // Send desktop notification
        if desktop_notifications {
            self.send_desktop_notification(alert);
        }
    }

    fn send_terminal_bell(&self) {
        // ASCII BEL character (0x07)
        print!("\x07");
        let _ = io::stdout().flush();
    }

    fn send_desktop_notification(&self, alert: &Alert) {
        // Determine urgency based on severity
        let urgency = match alert.severity {
            AlertSeverity::Critical => Urgency::Critical,
            AlertSeverity::Warning => Urgency::Normal,
            AlertSeverity::Recovery => Urgency::Low,
        };

        // Build notification
        let result = Notification::new()
            .summary("Monitor TUI Alert")
            .body(&alert.message)
            .urgency(urgency)
            .timeout(10000) // 10 seconds
            .show();

        // Log error if notification fails (don't crash)
        if let Err(e) = result {
            eprintln!("Failed to send desktop notification: {}", e);
        }
    }
}
