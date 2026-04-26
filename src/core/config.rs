use serde::Deserialize;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::sync::LazyLock;

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(load_config);

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub cache_file: String,
    pub battery: BatteryConfig,
    pub ignored_apps: Vec<String>,
    pub app_search_paths: Vec<String>,
    pub notifications: NotificationConfig,
    pub calculator: CalculatorConfig,
}

#[derive(Deserialize, Debug)]
pub struct BatteryConfig {
    pub device: String,
}

#[derive(Deserialize, Debug)]
pub struct NotificationConfig {
    pub status_cmd: Vec<String>,
    pub mute_cmd: Vec<String>,
    pub unmute_cmd: Vec<String>,
    pub clear_all_cmd: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct CalculatorConfig {
    pub command: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            cache_file: "~/.cache/launchpad.cache".to_string(),
            battery: BatteryConfig::default(),
            ignored_apps: vec!["xterm".to_string(), "uxterm".to_string()],
            app_search_paths: default_app_search_paths(),
            notifications: NotificationConfig::default(),
            calculator: CalculatorConfig::default(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            status_cmd: vec!["dunstctl".to_string(), "is-paused".to_string()],
            mute_cmd: vec![
                "dunstctl".to_string(),
                "set-paused".to_string(),
                "true".to_string(),
            ],
            unmute_cmd: vec![
                "dunstctl".to_string(),
                "set-paused".to_string(),
                "false".to_string(),
            ],
            clear_all_cmd: vec!["dunstctl".to_string(), "close-all".to_string()],
        }
    }
}

impl Default for CalculatorConfig {
    fn default() -> Self {
        Self {
            command: vec!["qalc".to_string(), "-t".to_string(), "--".to_string()],
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            device: "BAT0".to_string(),
        }
    }
}

fn default_app_search_paths() -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut paths = Vec::new();

    let push =
        |p: String, seen: &mut std::collections::HashSet<String>, paths: &mut Vec<String>| {
            if seen.insert(p.clone()) {
                paths.push(p);
            }
        };

    // XDG_DATA_HOME or ~/.local/share
    let data_home = env::var("XDG_DATA_HOME")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "~/.local/share".to_string());
    push(
        format!("{}/applications", data_home.trim_end_matches('/')),
        &mut seen,
        &mut paths,
    );

    // XDG_DATA_DIRS or default freedesktop fallbacks
    let data_dirs = env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".to_string());
    for dir in data_dirs.split(':').filter(|s| !s.is_empty()) {
        push(
            format!("{}/applications", dir.trim_end_matches('/')),
            &mut seen,
            &mut paths,
        );
    }

    // Flatpak system exports are not always covered by XDG_DATA_DIRS
    push(
        "/var/lib/flatpak/exports/share/applications".to_string(),
        &mut seen,
        &mut paths,
    );

    paths
}

fn load_config() -> AppConfig {
    let config_path = dirs::config_dir().map(|mut p| {
        p.push(env!("CARGO_PKG_NAME"));
        p.push("config.toml");
        p
    });

    if let Some(path) = config_path {
        log::debug!("Looking for config at: {}", path.display());

        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(cfg) => {
                    log::info!("Successfully loaded config from: {}", path.display());
                    return cfg;
                }
                Err(e) => {
                    log::warn!("Syntax error in TOML config {}: {}", path.display(), e);
                    log::warn!("Falling back to default configuration.");
                }
            },
            Err(e) if e.kind() == ErrorKind::NotFound => {
                log::debug!(
                    "Config file not found at {}, using defaults.",
                    path.display()
                );
            }
            Err(e) => {
                log::warn!("Failed to read config file {}: {}", path.display(), e);
                log::warn!("Falling back to default configuration.");
            }
        }
    } else {
        log::warn!("Could not determine OS config directory, using defaults.");
    }

    AppConfig::default()
}
