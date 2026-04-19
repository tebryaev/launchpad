use std::path::PathBuf;

pub fn expand_path(path: &str) -> Option<PathBuf> {
    if let Some(stripped) = path.strip_prefix("~/") {
        return dirs::home_dir().map(|mut home| {
            home.push(stripped);
            home
        });
    }
    Some(PathBuf::from(path))
}
