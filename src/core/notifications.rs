use std::process::Command;
use crate::core::config::CONFIG;

#[derive(Debug, PartialEq)]
pub enum NotificationStatus {
    Enabled,
    Muted,
}

fn spawn_cmd(cmd: &[String]) {
    if let [bin, args @ ..] = cmd {
        let _ = Command::new(bin).args(args).spawn();
    }
}

pub fn get_status() -> NotificationStatus {
    let Some(cfg) = CONFIG.get() else {
        return NotificationStatus::Enabled;
    };

    if let [bin, args @ ..] = cfg.notifications.status_cmd.as_slice() {
        if let Ok(output) = Command::new(bin).args(args).output() {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
            if result == "true" {
                return NotificationStatus::Muted;
            }
        }
    }

    NotificationStatus::Enabled
}

pub fn enable() {
    if let Some(cfg) = CONFIG.get() {
        spawn_cmd(&cfg.notifications.unmute_cmd);
    }
}

pub fn mute() {
    if let Some(cfg) = CONFIG.get() {
        spawn_cmd(&cfg.notifications.mute_cmd);
    }
}

pub fn clear_all() {
    if let Some(cfg) = CONFIG.get() {
        spawn_cmd(&cfg.notifications.clear_all_cmd);
    }
}