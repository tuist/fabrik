use std::path::PathBuf;

/// Get default cache directory following XDG conventions
///
/// - Linux/Unix: $XDG_CACHE_HOME/fabrik or ~/.cache/fabrik
/// - macOS: ~/Library/Caches/fabrik
/// - Windows: %LOCALAPPDATA%/fabrik/cache
pub fn default_cache_dir() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("fabrik")
    } else {
        // Fallback to current directory if we can't determine cache dir
        PathBuf::from(".fabrik/cache")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cache_dir() {
        let cache_dir = default_cache_dir();
        assert!(cache_dir.to_string_lossy().contains("fabrik"));
    }
}
