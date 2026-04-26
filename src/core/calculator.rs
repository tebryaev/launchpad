use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::core::config::CONFIG;

const CALC_TIMEOUT: Duration = Duration::from_millis(500);

pub fn evaluate(query: &str) -> Option<String> {
    if query.trim().is_empty() {
        return None;
    }

    let cfg = &*CONFIG;

    let [bin, args @ ..] = cfg.calculator.command.as_slice() else {
        log::error!("Calculator command is empty in config!");
        return None;
    };

    let mut child = Command::new(bin)
        .args(args)
        .arg(query)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let exit_status = match child.wait_timeout(CALC_TIMEOUT).ok()? {
        Some(s) => s,
        None => {
            log::warn!("Calculator command timed out for query: {query}");
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
    };

    if !exit_status.success() {
        return None;
    }

    let mut buf = String::new();
    child.stdout.take()?.read_to_string(&mut buf).ok()?;

    let result = buf.trim().to_string();
    if result.is_empty() || result.to_lowercase().contains("error") || result.contains("warning") {
        None
    } else {
        Some(result)
    }
}
