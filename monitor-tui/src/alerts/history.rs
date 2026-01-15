use crate::alerts::StatusTransition;
use crate::checker::Status;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum AlertSeverity {
    Critical, // Site went down
    Warning,  // Site has warning status
    Recovery, // Site recovered
}

impl AlertSeverity {
    pub fn from_transition(transition: &StatusTransition) -> Self {
        match transition {
            StatusTransition::UpToDown | StatusTransition::WarnToDown => Self::Critical,
            StatusTransition::UpToWarn => Self::Warning,
            StatusTransition::DownToUp | StatusTransition::WarnToUp => Self::Recovery,
            StatusTransition::DownToWarn => Self::Warning,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub timestamp: DateTime<Utc>,
    pub site_name: String,
    #[allow(dead_code)]
    pub transition: StatusTransition,
    pub severity: AlertSeverity,
    pub current_status: Status,
    pub previous_status: Status,
    pub message: String,
}

impl Alert {
    pub fn new(
        site_name: String,
        transition: StatusTransition,
        previous_status: Status,
        current_status: Status,
    ) -> Self {
        let severity = AlertSeverity::from_transition(&transition);
        let message = Self::format_message(&site_name, &transition);

        Self {
            timestamp: Utc::now(),
            site_name,
            transition,
            severity,
            current_status,
            previous_status,
            message,
        }
    }

    fn format_message(site_name: &str, transition: &StatusTransition) -> String {
        match transition {
            StatusTransition::UpToDown => {
                format!("{} is DOWN", site_name)
            }
            StatusTransition::UpToWarn => {
                format!("{} has WARNING status", site_name)
            }
            StatusTransition::DownToUp => {
                format!("{} has RECOVERED", site_name)
            }
            StatusTransition::WarnToDown => {
                format!("{} went from WARNING to DOWN", site_name)
            }
            StatusTransition::WarnToUp => {
                format!("{} recovered from WARNING", site_name)
            }
            StatusTransition::DownToWarn => {
                format!("{} went from DOWN to WARNING", site_name)
            }
        }
    }
}

/// History of alerts with ring buffer storage
pub struct AlertHistory {
    alerts: VecDeque<Alert>,
    max_size: usize,
}

impl AlertHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            alerts: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add_alert(&mut self, alert: Alert) {
        if self.alerts.len() >= self.max_size {
            self.alerts.pop_front();
        }
        self.alerts.push_back(alert);
    }

    #[allow(dead_code)]
    pub fn latest(&self) -> Option<&Alert> {
        self.alerts.back()
    }

    pub fn all_alerts(&self) -> &VecDeque<Alert> {
        &self.alerts
    }

    pub fn len(&self) -> usize {
        self.alerts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.alerts.is_empty()
    }

    /// Get alerts filtered by severity
    pub fn by_severity(&self, severity: AlertSeverity) -> Vec<&Alert> {
        self.alerts
            .iter()
            .filter(|a| a.severity == severity)
            .collect()
    }

    /// Get alerts for a specific site
    #[allow(dead_code)]
    pub fn by_site(&self, site_name: &str) -> Vec<&Alert> {
        self.alerts
            .iter()
            .filter(|a| a.site_name == site_name)
            .collect()
    }
}
