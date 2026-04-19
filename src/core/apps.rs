use std::fs;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::core::utils::expand_path;

#[derive(Debug, Clone, PartialEq)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon: String,
}

pub fn get_all_apps() -> Vec<AppInfo> {
    let mut apps = Vec::new();

    let config = crate::core::config::CONFIG.get();
    let search_paths = config
        .map(|c| c.app_search_paths.clone())
        .unwrap_or_default();
    let ignored = config
        .map(|c| c.ignored_apps.clone())
        .unwrap_or_default();

    for path_str in search_paths {
        let Some(path) = expand_path(&path_str) else {
            log::warn!("Could not resolve ~/ in path: {}", path_str);
            continue;
        };

        log::debug!("Searching for desktop files in: {}", path.display());

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("desktop") {
                    if let Some(app) = parse_desktop_file(entry.path()) {
                        if !ignored.contains(&app.name) {
                            apps.push(app);
                        }
                    }
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name == b.name);

    apps
}

fn parse_desktop_file(path: PathBuf) -> Option<AppInfo> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut no_display = false;
    let mut is_app = false;

    let mut in_main_section = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('[') {
            in_main_section = line == "[Desktop Entry]";
            continue;
        }

        if !in_main_section {
            continue;
        }

        // Skip localized variants (e.g., Name[ru]=), use only base key
        if line.starts_with("Name=") && name.is_none() {
            name = Some(line[5..].to_string());
        }
        if line.starts_with("Exec=") && exec.is_none() {
            exec = Some(line[5..].to_string());
        }
        if line.starts_with("Icon=") && icon.is_none() {
            icon = Some(line[5..].to_string());
        }
        if line == "NoDisplay=true" {
            no_display = true;
        }
        if line == "Type=Application" {
            is_app = true;
        }
    }

    if is_app && !no_display && name.is_some() && exec.is_some() {
        Some(AppInfo {
            name: name?,
            exec: exec?,
            icon: icon.unwrap_or_else(|| "system-run".to_string()),
        })
    } else {
        None
    }
}

pub fn launch_app(exec_command: &str) {
    let clean_exec = exec_command
        .split_whitespace()
        .filter(|part| !part.starts_with('%'))
        .collect::<Vec<_>>()
        .join(" ");

    let result = Command::new("sh")
        .arg("-c")
        .arg(clean_exec)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Detach from parent process group to prevent signal propagation
        .process_group(0)
        .spawn();

    if let Err(e) = result {
        log::error!("Launch error: {}", e);
    }
}
