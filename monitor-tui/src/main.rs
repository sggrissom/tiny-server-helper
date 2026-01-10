mod checker;
mod config;

use checker::{spawn_checker_task, CheckResult, Status};
use config::Config;
use tokio::sync::{mpsc, watch};

#[tokio::main]
async fn main() {
    // Load configuration
    let config = match Config::load() {
        Ok(config) => {
            println!("Configuration loaded successfully!\n");
            config
        }
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    };

    println!("Starting health checks for {} sites...\n", config.sites.len());

    // Create channels
    let (tx, mut rx) = mpsc::channel::<(String, CheckResult)>(100);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Spawn checker tasks
    let mut tasks = Vec::new();
    for site in config.sites.clone() {
        println!("Spawning checker for: {}", site.name);
        let handle = spawn_checker_task(
            site,
            tx.clone(),
            shutdown_rx.clone(),
            config.settings.request_timeout,
        );
        tasks.push(handle);
    }

    // Drop original tx so channel closes when all tasks finish
    drop(tx);

    println!("\nMonitoring... (Press Ctrl+C to stop)\n");

    // Receive and log results
    while let Some((site_name, result)) = rx.recv().await {
        let status_str = match result.status {
            Status::Up => "UP",
            Status::Down => "DOWN",
            Status::Warning => "WARN",
        };

        let time_str = result
            .response_time_ms
            .map(|ms| format!("{}ms", ms))
            .unwrap_or_else(|| "--".to_string());

        let http_str = result
            .http_status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "--".to_string());

        print!("[{}] {:20} | Status: {:4} | Time: {:6} | HTTP: {}",
            result.timestamp.format("%H:%M:%S"),
            site_name,
            status_str,
            time_str,
            http_str
        );

        if let Some(error) = result.error_message {
            print!(" | Error: {}", error);
        }

        println!();
    }

    // Graceful shutdown
    println!("\nShutting down...");
    let _ = shutdown_tx.send(true);
    for task in tasks {
        let _ = task.await;
    }
}
