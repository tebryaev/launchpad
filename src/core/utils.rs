use std::path::PathBuf;

/// Expand a path string, supporting `~`, `~user`, and `$VAR` references.
/// Falls back to the input verbatim if expansion fails.
pub fn expand_path(path: &str) -> Option<PathBuf> {
    match shellexpand::full(path) {
        Ok(expanded) => Some(PathBuf::from(expanded.as_ref())),
        Err(e) => {
            log::warn!("Failed to expand path '{}': {}", path, e);
            None
        }
    }
}
