use crate::config::SiteConfig;
use super::types::CheckResult;
use reqwest::Client;
use std::time::{Duration, Instant};

pub struct HttpChecker {
    client: Client,
}

impl HttpChecker {
    pub fn new(timeout_secs: u64) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("monitor-tui/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    pub async fn check(&self, site: &SiteConfig, warning_threshold_ms: Option<u64>) -> CheckResult {
        let start = Instant::now();

        match self.client.get(&site.url).send().await {
            Ok(response) => {
                let elapsed = start.elapsed();
                let status_code = response.status().as_u16();

                CheckResult::new_success(
                    elapsed.as_millis() as u64,
                    status_code,
                    site.expected_status,
                    warning_threshold_ms,
                )
            }
            Err(e) => {
                let error_msg = if e.is_timeout() {
                    "Request timeout".to_string()
                } else if e.is_connect() {
                    format!("Connection failed: {}", e)
                } else {
                    format!("Request failed: {}", e)
                };

                CheckResult::new_down(error_msg)
            }
        }
    }
}
