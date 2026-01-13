use crate::checker::Status;
use crate::config::Config;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum StatusTransition {
    UpToDown,
    UpToWarn,
    DownToUp,
    WarnToDown,
    WarnToUp,
    DownToWarn,
}

impl StatusTransition {
    pub fn from_statuses(from: &Status, to: &Status) -> Option<Self> {
        match (from, to) {
            (Status::Up, Status::Down) => Some(Self::UpToDown),
            (Status::Up, Status::Warning) => Some(Self::UpToWarn),
            (Status::Down, Status::Up) => Some(Self::DownToUp),
            (Status::Warning, Status::Down) => Some(Self::WarnToDown),
            (Status::Warning, Status::Up) => Some(Self::WarnToUp),
            (Status::Down, Status::Warning) => Some(Self::DownToWarn),
            _ => None,
        }
    }
}

/// Tracks state needed for alert detection decisions
struct SiteAlertState {
    consecutive_failures: usize,
    last_alert_time: Option<DateTime<Utc>>,
    last_alert_status: Option<Status>,
    stable_status: Option<Status>, // Status before current failure streak began
}

pub struct AlertDetector {
    config: Config,
    site_states: HashMap<String, SiteAlertState>,
}

impl AlertDetector {
    pub fn new(config: Config) -> Self {
        let site_states = config
            .sites
            .iter()
            .map(|site| {
                (
                    site.name.clone(),
                    SiteAlertState {
                        consecutive_failures: 0,
                        last_alert_time: None,
                        last_alert_status: None,
                        stable_status: None,
                    },
                )
            })
            .collect();

        Self {
            config,
            site_states,
        }
    }

    /// Evaluate whether a status change should trigger an alert
    pub fn evaluate(
        &mut self,
        site_name: &str,
        previous_status: Option<&Status>,
        current_status: &Status,
    ) -> Option<StatusTransition> {
        // Get alert settings for this site (merging global and site-specific)
        let site_config = self.config.sites.iter().find(|s| s.name == site_name)?;
        let global_alerts = &self.config.settings.alerts;
        let site_alerts = &site_config.alerts;

        // Check if alerts are enabled
        let enabled = site_alerts
            .as_ref()
            .and_then(|a| a.enabled)
            .unwrap_or(global_alerts.enabled);
        if !enabled {
            return None;
        }

        let state = self.site_states.get_mut(site_name)?;

        // Update stable status and consecutive failure count
        if *current_status == Status::Down || *current_status == Status::Warning {
            // If this is the first failure, remember what status we were in before
            if state.consecutive_failures == 0 {
                state.stable_status = previous_status.cloned();
            }
            state.consecutive_failures += 1;
        } else {
            // Back to UP status - reset everything
            state.consecutive_failures = 0;
            state.stable_status = Some(current_status.clone());
        }

        // Check consecutive failure threshold
        let threshold = site_alerts
            .as_ref()
            .and_then(|a| a.consecutive_failures)
            .unwrap_or(global_alerts.consecutive_failures);

        // For non-UP statuses, wait for threshold before alerting
        if (*current_status == Status::Down || *current_status == Status::Warning)
            && state.consecutive_failures < threshold
        {
            return None;
        }

        // Check cooldown period
        let cooldown_seconds = site_alerts
            .as_ref()
            .and_then(|a| a.cooldown_seconds)
            .unwrap_or(global_alerts.cooldown_seconds);

        if let Some(last_time) = state.last_alert_time {
            let elapsed = Utc::now().signed_duration_since(last_time);
            if elapsed < Duration::seconds(cooldown_seconds as i64) {
                // During cooldown, only alert if status changed
                if let Some(last_status) = &state.last_alert_status {
                    if last_status == current_status {
                        return None;
                    }
                }
            }
        }

        // Determine if this status transition should alert
        // Use stable_status (status before failures began) instead of immediate previous_status
        let from_status = if *current_status == Status::Up {
            // Recovery: use previous_status
            previous_status
        } else {
            // Failure: use stable_status (what we were before failures began)
            state.stable_status.as_ref()
        };

        let transition = from_status.and_then(|prev| StatusTransition::from_statuses(prev, current_status))?;

        let should_alert = match transition {
            StatusTransition::UpToDown => global_alerts.transitions.up_to_down,
            StatusTransition::UpToWarn => global_alerts.transitions.up_to_warn,
            StatusTransition::DownToUp => global_alerts.transitions.down_to_up,
            StatusTransition::WarnToDown => global_alerts.transitions.warn_to_down,
            StatusTransition::WarnToUp => global_alerts.transitions.warn_to_up,
            StatusTransition::DownToWarn => global_alerts.transitions.down_to_warn,
        };

        if should_alert {
            state.last_alert_time = Some(Utc::now());
            state.last_alert_status = Some(current_status.clone());
            Some(transition)
        } else {
            None
        }
    }
}
