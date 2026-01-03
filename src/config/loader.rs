use crate::config::file::FileConfig;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during config file loading
#[derive(Debug, Error, Clone)]
pub enum ConfigLoadError {
    #[error("Config file not found: {0}")]
    NotFound(String),

    #[error("Config file has insecure permissions (must be 0600): {0}")]
    InsecurePermissions(String),

    #[error("Failed to read config file: {0}")]
    ReadError(String),

    #[error("Failed to parse TOML config: {0}")]
    ParseError(String),

    #[error("Failed to check file permissions: {0}")]
    PermissionCheckError(String),
}

/// Load and parse a config file with security checks
/// 
/// Security requirements:
/// - File must exist
/// - File must have 0600 permissions (read/write for owner only)
/// - File must be valid TOML
/// 
/// Returns parsed FileConfig or ConfigLoadError
/// Pure function with I/O side effects isolated to file operations
pub fn load_config_file(path: &Path) -> Result<FileConfig, ConfigLoadError> {
    // Check if file exists
    if !path.exists() {
        return Err(ConfigLoadError::NotFound(
            path.display().to_string(),
        ));
    }

    // Check file permissions (must be 0600)
    check_file_permissions(path)?;

    // Read file contents
    let contents = fs::read_to_string(path).map_err(|e| {
        ConfigLoadError::ReadError(format!("Failed to read config file {}: {}", path.display(), e))
    })?;

    // Parse TOML
    let config: FileConfig = toml::from_str(&contents).map_err(|e| {
        ConfigLoadError::ParseError(format!(
            "Failed to parse TOML in config file {}: {}",
            path.display(),
            e
        ))
    })?;

    Ok(config)
}

/// Check if a file has secure permissions (0600)
/// 
/// On Unix systems, checks that file permissions are exactly 0600
/// (read/write for owner, no permissions for group/others)
/// 
/// On non-Unix systems, this is a no-op (returns Ok)
/// 
/// Pure function - checks permissions but doesn't modify state
#[cfg(unix)]
pub fn check_file_permissions(path: &Path) -> Result<(), ConfigLoadError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path).map_err(|e| {
        ConfigLoadError::PermissionCheckError(format!(
            "Failed to get metadata for {}: {}",
            path.display(),
            e
        ))
    })?;

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check if permissions are exactly 0600 (0o600)
    // This means: owner read/write (6), group none (0), others none (0)
    if (mode & 0o777) != 0o600 {
        return Err(ConfigLoadError::InsecurePermissions(format!(
            "File {} has permissions {:o}, but must be 0600",
            path.display(),
            mode & 0o777
        )));
    }

    Ok(())
}

/// Check file permissions on non-Unix systems
/// 
/// On non-Unix systems (Windows, etc.), we don't enforce strict permissions
/// as the permission model is different. This is a no-op.
#[cfg(not(unix))]
pub fn check_file_permissions(_path: &Path) -> Result<(), ConfigLoadError> {
    // On non-Unix systems, skip permission check
    // Windows and other systems have different permission models
    Ok(())
}

/// Resolve environment variable references in API keys
/// 
/// Supports format: ${VAR_NAME} or $VAR_NAME
/// 
/// Pure function - reads environment but doesn't modify state
pub fn resolve_env_var_reference(env_ref: &str) -> Option<String> {
    // Remove ${} or $ wrapper
    let var_name = env_ref
        .strip_prefix("${")
        .and_then(|s| s.strip_suffix("}"))
        .or_else(|| env_ref.strip_prefix("$"))
        .unwrap_or(env_ref);

    // Get environment variable
    std::env::var(var_name).ok()
}

/// Load config from all discovered paths, merging in precedence order
/// 
/// Returns the merged config from all existing config files
/// Files are loaded in order of precedence (highest to lowest)
/// 
/// This function has I/O side effects (file reading) but is otherwise pure
pub fn load_all_configs() -> Result<FileConfig, ConfigLoadError> {
    use crate::config::paths::existing_config_paths;

    let paths = existing_config_paths();

    if paths.is_empty() {
        // No config files found, return defaults
        return Ok(FileConfig::default());
    }

    // Load configs in order (highest priority first)
    // Later configs will override earlier ones in the merge
    let mut merged_config = FileConfig::default();

    for path in paths.iter().rev() {
        // Load from lowest to highest priority (reverse order)
        // So highest priority overrides lower priority
        match load_config_file(path) {
            Ok(config) => {
                merged_config = merge_configs(merged_config, config);
            }
            Err(e) => {
                // For non-fatal errors (file not found), continue to next file
                // For fatal errors (parse, permissions), return immediately
                match e {
                    ConfigLoadError::NotFound(_) => {
                        // File not found is non-fatal - continue to next config file
                        continue;
                    }
                    _ => {
                        // Parse errors, permission errors, etc. are fatal
                        return Err(e);
                    }
                }
                // Log error but continue with other configs
                eprintln!("Warning: Failed to load config from {}: {}", path.display(), e);
            }
        }
    }

    Ok(merged_config)
}

/// Merge two configs, with `override_config` taking precedence
/// 
/// Pure function - takes two immutable configs and returns merged config
/// No side effects
fn merge_configs(base: FileConfig, override_config: FileConfig) -> FileConfig {
    // For now, simple merge: override_config takes precedence
    // In a full implementation, we'd do deep merging for nested structures
    // For this subtask, we'll use the override config if it has any non-default values
    
    // Simple strategy: use override_config if it's not default, otherwise use base
    // This is a placeholder - full deep merge will be implemented in subtask 2.4
    if override_config != FileConfig::default() {
        override_config
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    use tempfile::TempDir;

    #[test]
    #[cfg(unix)]
    fn test_check_file_permissions_secure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.toml");

        // Create file with 0600 permissions
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"# test config").unwrap();
        drop(file);

        // Set permissions to 0600
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600)).unwrap();

        // Should pass
        assert!(check_file_permissions(&file_path).is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_check_file_permissions_insecure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.toml");

        // Create file with 0644 permissions (insecure)
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"# test config").unwrap();
        drop(file);

        // Set permissions to 0644
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o644)).unwrap();

        // Should fail
        let result = check_file_permissions(&file_path);
        assert!(result.is_err());
        match result {
            Err(ConfigLoadError::InsecurePermissions(_)) => {}
            _ => panic!("Expected InsecurePermissions error"),
        }
    }

    #[test]
    fn test_load_config_file_nonexistent() {
        let path = Path::new("/nonexistent/config.toml");
        let result = load_config_file(path);
        assert!(result.is_err());
        match result {
            Err(ConfigLoadError::NotFound(_)) => {}
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_load_config_file_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.toml");

        let toml_content = r#"
[provider]
default = "openrouter"

[context]
max-files = 20
"#;

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();
        drop(file);

        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o600)).unwrap();

        let result = load_config_file(&file_path);
        assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

        let config = result.unwrap();
        assert_eq!(config.provider.default, "openrouter");
        // Verify TOML parsing works - max_files should be 20 from TOML
        // The #[serde(default = "default_max_files")] only applies if field is missing
        // Since max_files = 20 is in the TOML, it should be 20, not the default 10
        assert_eq!(
            config.context.max_files, 20,
            "Expected max_files=20 from TOML, but got {}. TOML content:\n{}",
            config.context.max_files, toml_content
        );
    }

    #[test]
    fn test_resolve_env_var_reference() {
        // Set a test environment variable
        std::env::set_var("TEST_API_KEY", "test-key-value");

        // Test ${VAR} format
        assert_eq!(
            resolve_env_var_reference("${TEST_API_KEY}"),
            Some("test-key-value".to_string())
        );

        // Test $VAR format
        assert_eq!(
            resolve_env_var_reference("$TEST_API_KEY"),
            Some("test-key-value".to_string())
        );

        // Test nonexistent variable
        assert_eq!(resolve_env_var_reference("${NONEXISTENT}"), None);

        // Clean up
        std::env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_load_all_configs_no_files() {
        // Should return default config when no files exist
        let result = load_all_configs();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.provider.default, "openrouter");
    }
}

