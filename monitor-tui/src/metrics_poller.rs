use crate::config::ServerMetricsConfig;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, watch};

#[derive(Debug, Clone, Deserialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MemoryStats {
    pub total_kb: u64,
    pub available_kb: u64,
    pub used_kb: u64,
    pub used_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpuStats {
    pub user_pct: f64,
    pub system_pct: f64,
    pub idle_pct: f64,
    pub iowait_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiskStats {
    pub total_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub used_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SystemStats {
    pub load_avg: LoadAvg,
    pub memory: MemoryStats,
    pub cpu: CpuStats,
    pub disk: DiskStats,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppTraffic {
    pub window_seconds: u64,
    pub requests_total: u64,
    pub requests_per_min: f64,
    pub error_4xx: u64,
    pub error_5xx: u64,
    pub error_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppMetrics {
    pub name: String,
    pub disk_kb: u64,
    pub traffic: AppTraffic,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsSnapshot {
    pub collected_at: String,
    pub system: SystemStats,
    pub apps: Vec<AppMetrics>,
}

pub enum MetricsPoll {
    Ok(MetricsSnapshot),
    Error(String),
}

pub fn spawn_metrics_task(
    config: ServerMetricsConfig,
    tx: mpsc::Sender<MetricsPoll>,
    mut shutdown: watch::Receiver<bool>,
    mut force_refresh: broadcast::Receiver<()>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("monitor-tui/0.1.0")
            .build()
            .expect("Failed to build metrics HTTP client");

        let interval = Duration::from_secs(config.poll_interval);

        loop {
            let result = match client.get(&config.url).send().await {
                Ok(resp) => match resp.json::<MetricsSnapshot>().await {
                    Ok(snapshot) => MetricsPoll::Ok(snapshot),
                    Err(e) => MetricsPoll::Error(format!("Failed to parse metrics: {}", e)),
                },
                Err(e) => MetricsPoll::Error(format!("Failed to fetch metrics: {}", e)),
            };

            let _ = tx.send(result).await;

            tokio::select! {
                _ = tokio::time::sleep(interval) => continue,
                _ = force_refresh.recv() => continue,
                _ = shutdown.changed() => break,
            }
        }
    })
}
