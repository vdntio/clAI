use crate::cli::Cli;
use crate::config::file::FileConfig;
use crate::config::loader::ConfigLoadError;
use crate::config::merger::merge_all_configs;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Global lazy-loaded configuration cache
/// 
/// This is initialized on first access via `get_file_config()`
/// Thread-safe: uses Mutex for interior mutability during initialization
static FILE_CONFIG_CACHE: Lazy<Mutex<Option<Result<FileConfig, ConfigLoadError>>>> =
    Lazy::new(|| Mutex::new(None));

/// Get the merged file configuration (lazy-loaded)
/// 
/// This function triggers config loading on first access:
/// 1. Checks if config is already loaded
/// 2. If not, loads and merges configs from files, env vars, and CLI
/// 3. Caches the result for subsequent calls
/// 
/// Thread-safe: uses Mutex to ensure only one initialization
/// 
/// # Arguments
/// * `cli` - CLI arguments to merge into config (highest priority)
/// 
/// # Returns
/// * `Result<FileConfig, ConfigLoadError>` - Merged configuration or error
pub fn get_file_config(cli: &Cli) -> Result<FileConfig, ConfigLoadError> {
    let mut cache = FILE_CONFIG_CACHE.lock().unwrap();

    // Check if already loaded
    if let Some(ref cached_result) = *cache {
        // Return cloned result (both FileConfig and ConfigLoadError are Clone)
        return cached_result.clone();
    }

    // Load and merge configs
    let result = merge_all_configs(cli);

    // Cache the result
    *cache = Some(result.clone());

    result
}

/// Reset the config cache (useful for testing and benchmarking)
/// 
/// This clears the cached config, forcing a reload on next access
#[cfg(any(test, feature = "bench"))]
pub fn reset_config_cache() {
    let mut cache = FILE_CONFIG_CACHE.lock().unwrap();
    *cache = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;

    #[test]
    fn test_lazy_config_loading() {
        reset_config_cache();

        let cli = Cli {
            instruction: "test".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: crate::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
        };

        // First call should load config
        let config1 = get_file_config(&cli);
        assert!(config1.is_ok());

        // Second call should use cached config
        let config2 = get_file_config(&cli);
        assert!(config2.is_ok());

        // Both should be equal (same config)
        assert_eq!(config1.unwrap(), config2.unwrap());
    }

    #[test]
    fn test_config_cache_reset() {
        reset_config_cache();

        let cli = Cli {
            instruction: "test".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: crate::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
        };

        // Load config
        let _config1 = get_file_config(&cli);

        // Reset cache
        reset_config_cache();

        // Load again (should reload)
        let _config2 = get_file_config(&cli);
        assert!(_config2.is_ok());
    }
}

