pub mod http;
pub mod types;

use crate::config::SiteConfig;
use http::HttpChecker;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::time::Duration;
pub use types::{CheckResult, Status};

/// Spawn a background task that continuously checks a site
pub fn spawn_checker_task(
    site: SiteConfig,
    tx: mpsc::Sender<(String, CheckResult)>,
    mut shutdown: watch::Receiver<bool>,
    mut force_refresh: broadcast::Receiver<()>,
    timeout_secs: u64,
    default_interval: u64,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let checker = HttpChecker::new(timeout_secs);
        let interval = Duration::from_secs(site.check_interval.unwrap_or(default_interval));

        loop {
            // Perform check
            let result = checker.check(&site).await;

            // Send result (ignore if channel closed)
            let _ = tx.send((site.name.clone(), result)).await;

            // Sleep, force refresh, or shutdown
            tokio::select! {
                _ = tokio::time::sleep(interval) => continue,
                _ = force_refresh.recv() => {
                    // Force refresh received, skip sleep and check immediately
                    continue;
                }
                _ = shutdown.changed() => {
                    println!("Checker task for '{}' shutting down", site.name);
                    break;
                }
            }
        }
    })
}
