use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::apps;
use crate::log_reader::LogReader;
use crate::system::{self, CpuSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryInfo {
    pub total_kb: u64,
    pub available_kb: u64,
    pub used_kb: u64,
    pub used_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CpuInfo {
    pub user_pct: f64,
    pub system_pct: f64,
    pub idle_pct: f64,
    pub iowait_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiskInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub used_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    pub load_avg: LoadAvg,
    pub memory: MemoryInfo,
    pub cpu: CpuInfo,
    pub disk: DiskInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrafficWindow {
    pub window_seconds: u64,
    pub requests_total: u64,
    pub requests_per_min: f64,
    pub error_4xx: u64,
    pub error_5xx: u64,
    pub error_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetrics {
    pub name: String,
    pub disk_kb: u64,
    pub traffic: TrafficWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub collected_at: DateTime<Utc>,
    pub system: SystemMetrics,
    pub apps: Vec<AppMetrics>,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            collected_at: Utc::now(),
            system: SystemMetrics::default(),
            apps: Vec::new(),
        }
    }
}

pub struct Config {
    pub port: u16,
    pub apps_dir: String,
    pub log_dir: String,
    pub collect_interval: u64,
    pub metrics_window_seconds: u64,
}

pub async fn collect(
    config: &Config,
    prev_cpu: &mut Option<CpuSnapshot>,
    log_reader: &mut LogReader,
) -> MetricsSnapshot {
    let load_avg = system::read_load_avg().unwrap_or_default();
    let memory = system::read_memory().unwrap_or_default();
    let disk = system::read_disk().unwrap_or_default();

    let curr_snapshot = system::read_cpu_snapshot().ok();
    let cpu = match (prev_cpu.as_ref(), curr_snapshot.as_ref()) {
        (Some(prev), Some(curr)) => system::cpu_diff(prev, curr),
        _ => CpuInfo::default(),
    };
    *prev_cpu = curr_snapshot;

    let apps_dir = Path::new(&config.apps_dir);
    let app_names = apps::scan_apps(apps_dir);
    let mut traffic_map = log_reader.update(&app_names);

    let app_metrics = app_names
        .into_iter()
        .map(|name| {
            let app_dir = apps_dir.join(&name);
            let disk_kb = apps::disk_usage_kb(&app_dir);
            let traffic = traffic_map.remove(&name).unwrap_or(TrafficWindow {
                window_seconds: config.metrics_window_seconds,
                ..Default::default()
            });
            AppMetrics { name, disk_kb, traffic }
        })
        .collect();

    MetricsSnapshot {
        collected_at: Utc::now(),
        system: SystemMetrics { load_avg, memory, cpu, disk },
        apps: app_metrics,
    }
}
