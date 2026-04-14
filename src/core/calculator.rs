use std::process::Command;
use crate::core::config::CONFIG;

pub fn evaluate(query: &str) -> Option<String> {
    if query.trim().is_empty() {
        return None;
    }

    let cfg = CONFIG.get()?;

    if let [bin, args @ ..] = cfg.calculator.command.as_slice() {
        let output = Command::new(bin)
            .args(args)
            .arg(query)
            .output()
            .ok()?;

        if output.status.success() {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if result.is_empty()
                || result.to_lowercase().contains("error")
                || result.contains("warning")
            {
                None
            } else {
                Some(result)
            }
        } else {
            None
        }
    } else {
        log::error!("Calculator command is empty in config!");
        None
    }
}