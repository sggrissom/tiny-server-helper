mod config;

use config::Config;

fn main() {
    match Config::load() {
        Ok(config) => {
            println!("Configuration loaded successfully!\n");
            println!("Settings:");
            println!("  Refresh interval: {}s", config.settings.refresh_interval);
            println!("  History size: {}", config.settings.history_size);
            println!("  Request timeout: {}s", config.settings.request_timeout);
            println!("\nSites:");
            for site in &config.sites {
                println!("  - {} ({})", site.name, site.url);
                println!("    Expected status: {}", site.expected_status);
                if let Some(interval) = site.check_interval {
                    println!("    Check interval: {}s", interval);
                }
            }
        }
        Err(e) => {
            eprintln!("Error loading configuration: {}", e);
            std::process::exit(1);
        }
    }
}
