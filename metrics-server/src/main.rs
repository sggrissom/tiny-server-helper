mod apps;
mod metrics;
mod system;

use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use metrics::{Config, MetricsSnapshot};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

type SharedState = Arc<RwLock<MetricsSnapshot>>;

async fn healthz() -> &'static str {
    "ok"
}

async fn get_metrics(State(state): State<SharedState>) -> impl IntoResponse {
    Json(state.read().await.clone())
}

async fn run_collector(config: Arc<Config>, state: SharedState) {
    let mut prev_cpu = None;
    loop {
        let snapshot = metrics::collect(&config, &mut prev_cpu).await;
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

    let state: SharedState = Arc::new(RwLock::new(MetricsSnapshot::default()));

    tokio::spawn(run_collector(config.clone(), state.clone()));

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
