use crate::checker::{CheckResult, Status};
use std::collections::VecDeque;

/// Historical data for a single monitored site
pub struct SiteHistory {
    results: VecDeque<CheckResult>,
    max_size: usize,
}

impl SiteHistory {
    /// Create a new SiteHistory with specified maximum size
    pub fn new(max_size: usize) -> Self {
        Self {
            results: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add a new check result, maintaining size limit
    pub fn add_result(&mut self, result: CheckResult) {
        if self.results.len() >= self.max_size {
            self.results.pop_front();
        }
        self.results.push_back(result);
    }

    /// Get the most recent check result
    pub fn latest(&self) -> Option<&CheckResult> {
        self.results.back()
    }

    /// Calculate average response time from recent results
    pub fn avg_response_time(&self) -> Option<u64> {
        let times: Vec<u64> = self
            .results
            .iter()
            .filter_map(|r| r.response_time_ms)
            .collect();

        if times.is_empty() {
            None
        } else {
            Some(times.iter().sum::<u64>() / times.len() as u64)
        }
    }

    /// Calculate uptime percentage (% of Up status results)
    pub fn uptime_percentage(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }

        let up_count = self
            .results
            .iter()
            .filter(|r| r.status == Status::Up)
            .count();

        (up_count as f64 / self.results.len() as f64) * 100.0
    }

    /// Get the number of stored results
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Check if history is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get recent response times for sparkline visualization
    /// Returns up to the last `limit` response times in chronological order
    pub fn recent_response_times(&self, limit: usize) -> Vec<u64> {
        self.results
            .iter()
            .rev()
            .take(limit)
            .filter_map(|r| r.response_time_ms)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get minimum response time from all results
    pub fn min_response_time(&self) -> Option<u64> {
        self.results
            .iter()
            .filter_map(|r| r.response_time_ms)
            .min()
    }

    /// Get maximum response time from all results
    pub fn max_response_time(&self) -> Option<u64> {
        self.results
            .iter()
            .filter_map(|r| r.response_time_ms)
            .max()
    }

    /// Get chart data points for visualization
    /// Returns (timestamp_seconds, response_time_ms) pairs
    pub fn chart_data(&self) -> Vec<(f64, f64)> {
        self.results
            .iter()
            .filter_map(|r| {
                r.response_time_ms.map(|time| {
                    let timestamp = r.timestamp.timestamp() as f64;
                    (timestamp, time as f64)
                })
            })
            .collect()
    }

    /// Get all results (for detailed view)
    pub fn all_results(&self) -> &VecDeque<CheckResult> {
        &self.results
    }
}
