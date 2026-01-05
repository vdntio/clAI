use std::path::{Path, PathBuf};

/// Discover all config file paths in correct precedence order
/// Follows XDG Base Directory Specification
/// Pure function - no side effects (reads environment but doesn't modify state)
///
/// Order of precedence (highest to lowest):
/// 1. ./.clai.toml (current directory)
/// 2. $XDG_CONFIG_HOME/clai/config.toml
/// 3. ~/.config/clai/config.toml (fallback if XDG_CONFIG_HOME not set)
/// 4. /etc/clai/config.toml (system-wide)
///
/// Returns paths in order from highest to lowest priority
pub fn discover_config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // 1. Current directory config (highest priority)
    paths.push(PathBuf::from("./.clai.toml"));

    // 2. XDG config home
    let xdg_config_path = get_xdg_config_path();
    if let Some(path) = xdg_config_path {
        paths.push(path);
    }

    // 3. Home directory fallback (~/.config/clai/config.toml)
    if let Some(home_path) = get_home_config_path() {
        // Only add if different from XDG path (avoid duplicates)
        if !paths.contains(&home_path) {
            paths.push(home_path);
        }
    }

    // 4. System-wide config (lowest priority)
    paths.push(PathBuf::from("/etc/clai/config.toml"));

    paths
}

/// Get XDG config home path
/// Pure function - reads environment but doesn't modify state
fn get_xdg_config_path() -> Option<PathBuf> {
    // Check XDG_CONFIG_HOME environment variable
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg_config_home.is_empty() {
            return Some(
                PathBuf::from(xdg_config_home)
                    .join("clai")
                    .join("config.toml"),
            );
        }
    }
    None
}

/// Get home directory config path (~/.config/clai/config.toml)
/// Pure function - reads environment but doesn't modify state
fn get_home_config_path() -> Option<PathBuf> {
    // Use directories crate for cross-platform home directory detection
    if let Some(home_dir) = directories::BaseDirs::new() {
        return Some(home_dir.config_dir().join("clai").join("config.toml"));
    }
    None
}

/// Check if a config file exists
/// Pure function - checks file system but doesn't modify state
pub fn config_file_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Filter config paths to only those that exist
/// Pure function - reads file system but doesn't modify state
pub fn existing_config_paths() -> Vec<PathBuf> {
    discover_config_paths()
        .into_iter()
        .filter(|path| config_file_exists(path))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_discover_config_paths_returns_all_paths() {
        let paths = discover_config_paths();

        // Should always return at least current dir and system paths
        assert!(paths.len() >= 2);

        // First should be current directory
        assert_eq!(paths[0], PathBuf::from("./.clai.toml"));

        // Last should be system path
        assert_eq!(
            paths[paths.len() - 1],
            PathBuf::from("/etc/clai/config.toml")
        );
    }

    #[test]
    fn test_discover_config_paths_order() {
        let paths = discover_config_paths();

        // Verify order: current dir first, system last
        assert_eq!(paths[0], PathBuf::from("./.clai.toml"));
        assert_eq!(
            paths[paths.len() - 1],
            PathBuf::from("/etc/clai/config.toml")
        );
    }

    #[test]
    fn test_get_xdg_config_path_with_env() {
        // Save original value
        let original = env::var("XDG_CONFIG_HOME").ok();

        // Set test value
        env::set_var("XDG_CONFIG_HOME", "/test/xdg/config");

        let path = get_xdg_config_path();
        assert_eq!(
            path,
            Some(PathBuf::from("/test/xdg/config/clai/config.toml"))
        );

        // Restore original
        match original {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    #[test]
    fn test_get_xdg_config_path_without_env() {
        // Save original value
        let original = env::var("XDG_CONFIG_HOME").ok();

        // Remove env var
        env::remove_var("XDG_CONFIG_HOME");

        let path = get_xdg_config_path();
        assert_eq!(path, None);

        // Restore original
        match original {
            Some(val) => env::set_var("XDG_CONFIG_HOME", val),
            None => {}
        }
    }

    #[test]
    fn test_config_file_exists_nonexistent() {
        let path = PathBuf::from("/nonexistent/path/config.toml");
        assert!(!config_file_exists(&path));
    }

    #[test]
    fn test_existing_config_paths_filters_nonexistent() {
        // This test depends on actual file system state
        // Just verify it doesn't panic and returns a Vec
        let paths = existing_config_paths();
        assert!(paths.len() <= discover_config_paths().len());
    }

    #[test]
    fn test_discover_config_paths_pure() {
        // Pure function - same environment, same output
        let paths1 = discover_config_paths();
        let paths2 = discover_config_paths();

        // Should return same paths in same order
        assert_eq!(paths1, paths2);
    }
}
