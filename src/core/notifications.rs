use std::process::Command;
use crate::core::config::CONFIG;

#[derive(Debug, PartialEq)]
pub enum NotificationStatus {
    Enabled,
    Muted,
}

pub fn get_status() -> NotificationStatus {
    let cfg = CONFIG.get().expect("Config not initialized");

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
        if let [bin, args @ ..] = cfg.notifications.unmute_cmd.as_slice() {
            let _ = Command::new(bin).args(args).spawn();
        }
    }
}

pub fn mute() {
    if let Some(cfg) = CONFIG.get() {
        if let [bin, args @ ..] = cfg.notifications.mute_cmd.as_slice() {
            let _ = Command::new(bin).args(args).spawn();
        }
    }
}

pub fn clear_all() {
    if let Some(cfg) = CONFIG.get() {
        if let [bin, args @ ..] = cfg.notifications.clear_all_cmd.as_slice() {
            let _ = Command::new(bin).args(args).spawn();
        }
    }
}