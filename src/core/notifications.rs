use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::core::config::CONFIG;

const STATUS_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, PartialEq)]
pub enum NotificationStatus {
    Enabled,
    Muted,
}

fn spawn_cmd(cmd: &[String]) {
    if let [bin, args @ ..] = cmd {
        let _ = Command::new(bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

pub fn get_status() -> NotificationStatus {
    let cfg = &*CONFIG;

    if let [bin, args @ ..] = cfg.notifications.status_cmd.as_slice() {
        let mut child = match Command::new(bin)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to spawn notifications status command: {e}");
                return NotificationStatus::Enabled;
            }
        };

        match child.wait_timeout(STATUS_TIMEOUT) {
            Ok(Some(_status)) => {
                let mut buf = String::new();
                if let Some(mut out) = child.stdout.take() {
                    use std::io::Read;
                    let _ = out.read_to_string(&mut buf);
                }
                if buf.trim().eq_ignore_ascii_case("true") {
                    return NotificationStatus::Muted;
                }
            }
            Ok(None) => {
                log::warn!("Notifications status command timed out, treating as enabled");
                let _ = child.kill();
                let _ = child.wait();
            }
            Err(e) => {
                log::warn!("Error waiting for notifications status command: {e}");
            }
        }
    }

    NotificationStatus::Enabled
}

pub fn enable() {
    spawn_cmd(&CONFIG.notifications.unmute_cmd);
}

pub fn mute() {
    spawn_cmd(&CONFIG.notifications.mute_cmd);
}

pub fn clear_all() {
    spawn_cmd(&CONFIG.notifications.clear_all_cmd);
}
