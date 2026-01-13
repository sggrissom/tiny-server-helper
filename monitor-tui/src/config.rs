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
    #[serde(default)]
    pub alerts: AlertSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    pub name: String,
    pub url: String,
    pub expected_status: u16,
    pub check_interval: Option<u64>,
    #[serde(default)]
    pub alerts: Option<SiteAlertSettings>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub terminal_bell: bool,
    #[serde(default = "default_true")]
    pub desktop_notifications: bool,
    #[serde(default = "default_alert_history_size")]
    pub alert_history_size: usize,
    #[serde(default = "default_consecutive_failures")]
    pub consecutive_failures: usize,
    #[serde(default = "default_cooldown_seconds")]
    pub cooldown_seconds: u64,
    #[serde(default)]
    pub transitions: TransitionSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransitionSettings {
    #[serde(default = "default_true")]
    pub up_to_down: bool,
    #[serde(default = "default_false")]
    pub up_to_warn: bool,
    #[serde(default = "default_true")]
    pub down_to_up: bool,
    #[serde(default = "default_true")]
    pub warn_to_down: bool,
    #[serde(default = "default_true")]
    pub warn_to_up: bool,
    #[serde(default = "default_false")]
    pub down_to_warn: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteAlertSettings {
    pub enabled: Option<bool>,
    pub consecutive_failures: Option<usize>,
    pub cooldown_seconds: Option<u64>,
    pub terminal_bell: Option<bool>,
    pub desktop_notifications: Option<bool>,
}

impl Default for AlertSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            terminal_bell: true,
            desktop_notifications: true,
            alert_history_size: 200,
            consecutive_failures: 2,
            cooldown_seconds: 300,
            transitions: TransitionSettings::default(),
        }
    }
}

impl Default for TransitionSettings {
    fn default() -> Self {
        Self {
            up_to_down: true,
            up_to_warn: false,
            down_to_up: true,
            warn_to_down: true,
            warn_to_up: true,
            down_to_warn: false,
        }
    }
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

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_alert_history_size() -> usize {
    200
}

fn default_consecutive_failures() -> usize {
    2
}

fn default_cooldown_seconds() -> u64 {
    300
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
