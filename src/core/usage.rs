use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use crate::core::config::CONFIG;
use crate::core::utils::expand_path;

fn get_usage_file() -> PathBuf {
    let cache_setting = &CONFIG.cache_file;
    if !cache_setting.is_empty()
        && let Some(p) = expand_path(cache_setting)
    {
        return p;
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

pub fn load_usage() -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    if let Ok(content) = std::fs::read_to_string(get_usage_file()) {
        for line in content.lines() {
            if let Some((name, count)) = line.split_once('=')
                && let Ok(c) = count.parse()
            {
                map.insert(name.to_string(), c);
            }
        }
    }
    map
}

pub fn record_launch(app_name: &str) {
    let mut usage = load_usage();
    *usage.entry(app_name.to_string()).or_insert(0) += 1;

    let mut content = String::with_capacity(usage.len() * 16);
    for (k, v) in &usage {
        let _ = writeln!(content, "{k}={v}");
    }

    let path = get_usage_file();

    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        log::warn!(
            "Failed to create usage cache dir {}: {}",
            parent.display(),
            e
        );
        return;
    }

    // Atomic write: write to a temp file in the same directory, then rename.
    let tmp = path.with_extension("cache.tmp");
    if let Err(e) = std::fs::write(&tmp, &content) {
        log::warn!("Failed to write usage cache temp {}: {}", tmp.display(), e);
        return;
    }
    if let Err(e) = std::fs::rename(&tmp, &path) {
        log::warn!("Failed to rename usage cache to {}: {}", path.display(), e);
        let _ = std::fs::remove_file(&tmp);
    }
}
