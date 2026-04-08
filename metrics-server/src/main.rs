mod apps;
mod log_reader;
mod metrics;
mod system;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
};
use log_reader::LogReader;
use metrics::{Config, MetricsSnapshot};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

#[derive(Clone)]
struct AppState {
    metrics: Arc<RwLock<MetricsSnapshot>>,
    api_key: Option<String>,
}

async fn healthz() -> &'static str {
    "ok"
}

async fn get_metrics(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(key) = &state.api_key {
        let expected = format!("Bearer {}", key);
        let provided = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if provided != expected {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }
    Json(state.metrics.read().await.clone()).into_response()
}

async fn run_collector(config: Arc<Config>, state: Arc<RwLock<MetricsSnapshot>>) {
    let mut prev_cpu = None;
    let mut log_reader = LogReader::new(&config.log_dir, config.metrics_window_seconds);
    loop {
        let snapshot = metrics::collect(&config, &mut prev_cpu, &mut log_reader).await;
        *state.write().await = snapshot;
        tokio::time::sleep(Duration::from_secs(config.collect_interval)).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port: u16 = std::env::var("PORT")
        .expect("PORT env var is required")
        .parse()
        .expect("PORT must be a valid port number");

    let config = Arc::new(Config {
        port,
        apps_dir: std::env::var("APPS_DIR").unwrap_or_else(|_| "/srv/apps".into()),
        log_dir: std::env::var("LOG_DIR").unwrap_or_else(|_| "/var/log/caddy".into()),
        collect_interval: std::env::var("COLLECT_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30),
        metrics_window_seconds: std::env::var("METRICS_WINDOW_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900),
    });

    let api_key = std::env::var("API_KEY").ok();

    let state = AppState {
        metrics: Arc::new(RwLock::new(MetricsSnapshot::default())),
        api_key,
    };

    tokio::spawn(run_collector(config.clone(), state.metrics.clone()));

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(get_metrics))
        .with_state(state);

    let addr = format!("127.0.0.1:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("metrics-server listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
