use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Up,      // HTTP status matches expected
    Down,    // Request failed or timeout
    Warning, // HTTP success but unexpected status code
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub timestamp: DateTime<Utc>,
    pub status: Status,
    pub response_time_ms: Option<u64>,
    pub http_status: Option<u16>,
    pub error_message: Option<String>,
}

impl CheckResult {
    pub fn new_down(error: String) -> Self {
        Self {
            timestamp: Utc::now(),
            status: Status::Down,
            response_time_ms: None,
            http_status: None,
            error_message: Some(error),
        }
    }

    pub fn new_success(
        response_time_ms: u64,
        http_status: u16,
        expected_status: u16,
        warning_threshold_ms: Option<u64>,
    ) -> Self {
        let status_mismatch = http_status != expected_status;
        let slow_response = warning_threshold_ms
            .filter(|&t| t > 0)
            .is_some_and(|t| response_time_ms > t);

        let status = if status_mismatch || slow_response {
            Status::Warning
        } else {
            Status::Up
        };

        Self {
            timestamp: Utc::now(),
            status,
            response_time_ms: Some(response_time_ms),
            http_status: Some(http_status),
            error_message: None,
        }
    }
}
