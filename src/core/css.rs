use std::fs;
use std::io::ErrorKind;

pub fn load_css() -> String {
    let default_css = include_str!("../../style.css");

    if let Some(mut config_path) = dirs::config_dir() {
        config_path.push(env!("CARGO_PKG_NAME"));
        config_path.push("style.css");

        match fs::read_to_string(&config_path) {
            Ok(custom_css) => {
                log::info!("Loaded custom CSS from: {}", config_path.display());
                return custom_css;
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                log::debug!("Custom CSS not found at {}", config_path.display());
            }
            Err(e) => {
                log::warn!("Failed to read {}: {}", config_path.display(), e);
            }
        }
    } else {
        log::debug!("Could not determine OS config directory");
    }

    log::info!("Loading embedded fallback CSS");
    default_css.to_string()
}
