use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::utils::expand_path;

fn get_usage_file() -> PathBuf {
    if let Some(cfg) = crate::core::config::CONFIG.get() {
        if !cfg.cache_file.is_empty() {
            if let Some(p) = expand_path(&cfg.cache_file) {
                return p;
            }
        }
    }

    dirs::cache_dir()
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|mut p| {
                    p.push(".cache");
                    p
                })
                .unwrap_or_else(|| PathBuf::from("/tmp"))
        })
        .join("launchpad.cache")
}

pub fn load_usage() -> HashMap<String, usize> {
    let mut map = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(get_usage_file()) {
        for line in content.lines() {
            if let Some((name, count)) = line.split_once('=') {
                if let Ok(c) = count.parse() {
                    map.insert(name.to_string(), c);
                }
            }
        }
    }
    map
}

pub fn record_launch(app_name: &str) {
    let mut usage = load_usage();
    *usage.entry(app_name.to_string()).or_insert(0) += 1;

    let content = usage
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("\n");

    let path = get_usage_file();

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("Failed to create usage cache dir {}: {}", parent.display(), e);
            return;
        }
    }

    if let Err(e) = std::fs::write(&path, content) {
        log::warn!("Failed to write usage cache {}: {}", path.display(), e);
    }
}