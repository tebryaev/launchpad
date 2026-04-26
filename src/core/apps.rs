use std::env;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::core::config::CONFIG;
use crate::core::utils::expand_path;

#[derive(Debug, Clone, PartialEq)]
pub struct AppInfo {
    pub name: String,
    pub exec: String,
    pub icon: String,
}

pub fn get_all_apps() -> Vec<AppInfo> {
    let cfg = &*CONFIG;
    let current_desktops = current_desktop_list();

    let mut apps = Vec::new();
    for path_str in &cfg.app_search_paths {
        let Some(path) = expand_path(path_str) else {
            continue;
        };

        log::debug!("Searching for desktop files in: {}", path.display());

        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            let Some(app) = parse_desktop_file(&entry_path, &current_desktops) else {
                continue;
            };
            if cfg.ignored_apps.contains(&app.name) {
                continue;
            }
            apps.push(app);
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));

    apps
}

fn current_desktop_list() -> Vec<String> {
    env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_default()
        .split(':')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn parse_desktop_file(path: &Path, current_desktops: &[String]) -> Option<AppInfo> {
    let content = fs::read_to_string(path).ok()?;

    let mut name: Option<String> = None;
    let mut exec: Option<String> = None;
    let mut icon: Option<String> = None;
    let mut try_exec: Option<String> = None;
    let mut only_show_in: Option<Vec<String>> = None;
    let mut not_show_in: Option<Vec<String>> = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut is_app = false;

    let mut in_main_section = false;

    for raw in content.lines() {
        let line = raw.trim();

        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_main_section = section == "Desktop Entry";
            // Stop parsing after the main section ends — Action sections must not override it.
            if !in_main_section && (name.is_some() || exec.is_some() || icon.is_some()) {
                break;
            }
            continue;
        }

        if !in_main_section {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();

        // Skip localized variants like "Name[ru]=..." — keep only the base key.
        if key.contains('[') {
            continue;
        }

        match key {
            "Name" if name.is_none() => name = Some(value.to_string()),
            "Exec" if exec.is_none() => exec = Some(value.to_string()),
            "Icon" if icon.is_none() => icon = Some(value.to_string()),
            "TryExec" if try_exec.is_none() => try_exec = Some(value.to_string()),
            "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
            "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
            "Type" => is_app = value == "Application",
            "OnlyShowIn" => {
                only_show_in = Some(
                    value
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect(),
                );
            }
            "NotShowIn" => {
                not_show_in = Some(
                    value
                        .split(';')
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect(),
                );
            }
            _ => {}
        }
    }

    if !is_app || no_display || hidden {
        return None;
    }

    if let Some(only) = &only_show_in
        && !current_desktops.iter().any(|d| only.contains(d))
    {
        return None;
    }
    if let Some(not_in) = &not_show_in
        && current_desktops.iter().any(|d| not_in.contains(d))
    {
        return None;
    }

    if let Some(try_path) = try_exec
        && !try_exec_resolves(&try_path)
    {
        return None;
    }

    Some(AppInfo {
        name: name?,
        exec: exec?,
        icon: icon.unwrap_or_else(|| "system-run".to_string()),
    })
}

fn try_exec_resolves(candidate: &str) -> bool {
    let p = Path::new(candidate);
    if p.is_absolute() {
        return p.is_file();
    }
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':').filter(|s| !s.is_empty()) {
            let candidate_path = PathBuf::from(dir).join(p);
            if candidate_path.is_file() {
                return true;
            }
        }
    }
    false
}

pub fn launch_app(exec_command: &str) {
    // Strip freedesktop field codes (%f, %F, %u, %U, %i, %c, %k, %d, %D, %n, %N, %v, %m).
    // Then parse via shlex to honour quoting and avoid sh -c injection.
    let stripped = strip_field_codes(exec_command);

    let argv = match shlex::split(&stripped) {
        Some(v) if !v.is_empty() => v,
        _ => {
            log::error!("Failed to parse Exec line: {}", exec_command);
            return;
        }
    };

    let result = Command::new(&argv[0])
        .args(&argv[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Detach from parent process group to prevent signal propagation.
        .process_group(0)
        .spawn();

    if let Err(e) = result {
        log::error!("Launch error: {e}");
    }
}

fn strip_field_codes(exec: &str) -> String {
    let mut out = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next) = chars.peek() {
                if next == '%' {
                    // Literal percent sign.
                    out.push('%');
                    chars.next();
                } else {
                    // Drop the field code character.
                    chars.next();
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}
