use std::collections::HashMap;
use std::path::PathBuf;

fn expand_path(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(mut home) = dirs::home_dir() {
            home.push(stripped);
            return home;
        }
    }
    PathBuf::from(path)
}

fn get_usage_file() -> PathBuf {
    if let Some(cfg) = crate::core::config::CONFIG.get() {
        if !cfg.cache_file.is_empty() {
            return expand_path(&cfg.cache_file);
        }
    }

    dirs::cache_dir()
        .unwrap_or_else(|| {
            // Fallback if cache_dir is unavailable
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
        let _ = std::fs::create_dir_all(parent);
    }

    let _ = std::fs::write(path, content);
}