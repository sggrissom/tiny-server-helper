use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub settings: Settings,
    pub sites: Vec<SiteConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    #[serde(default = "default_request_timeout")]
    pub request_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    pub name: String,
    pub url: String,
    pub expected_status: u16,
    pub check_interval: Option<u64>,
}

// Default value functions
fn default_refresh_interval() -> u64 {
    5
}

fn default_history_size() -> usize {
    100
}

fn default_request_timeout() -> u64 {
    3
}

impl Config {
    /// Load configuration from file, checking multiple locations in priority order
    pub fn load() -> Result<Self> {
        let config_paths = Self::get_config_paths();

        for path in &config_paths {
            if path.exists() {
                let contents = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

                let config: Config = toml::from_str(&contents)
                    .with_context(|| format!("Failed to parse TOML config: {}", path.display()))?;

                config.validate()?;

                println!("Loaded config from: {}", path.display());
                return Ok(config);
            }
        }

        anyhow::bail!(
            "No configuration file found. Checked:\n  - {}",
            config_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join("\n  - ")
        );
    }

    /// Get list of config file paths in priority order
    fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // 1. Current directory
        paths.push(PathBuf::from("./sites.toml"));

        // 2. User config directory
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".config/monitor/sites.toml");
            paths.push(path);
        }

        // 3. System config directory
        paths.push(PathBuf::from("/etc/monitor/sites.toml"));

        paths
    }

    /// Validate configuration
    fn validate(&self) -> Result<()> {
        // Validate that we have at least one site
        if self.sites.is_empty() {
            anyhow::bail!("Configuration must define at least one site");
        }

        // Validate each site
        for site in &self.sites {
            // Check URL is valid
            if site.url.is_empty() {
                anyhow::bail!("Site '{}' has empty URL", site.name);
            }

            if !site.url.starts_with("http://") && !site.url.starts_with("https://") {
                anyhow::bail!(
                    "Site '{}' has invalid URL '{}' - must start with http:// or https://",
                    site.name,
                    site.url
                );
            }

            // Validate status code is in valid range
            if site.expected_status < 100 || site.expected_status >= 600 {
                anyhow::bail!(
                    "Site '{}' has invalid expected_status {} - must be 100-599",
                    site.name,
                    site.expected_status
                );
            }
        }

        Ok(())
    }
}
