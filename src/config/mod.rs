use crate::cli::{Cli, ColorChoice};
use std::path::PathBuf;

/// Runtime configuration struct derived from CLI arguments
/// This is the runtime config used during execution
/// All fields are immutable - struct implements Clone for copying
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub instruction: String,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub quiet: bool,
    pub verbose: u8,
    pub no_color: bool,
    pub color: ColorChoice,
    pub interactive: bool,
    pub force: bool,
    pub dry_run: bool,
    pub context: Option<String>,
    pub offline: bool,
    /// Number of command options to generate (1-10)
    pub num_options: u8,
    /// Show debug information (prompt sent to AI)
    pub debug: bool,
    /// Debug log file path (None = disabled, Some(path) = enabled)
    pub debug_log_file: Option<PathBuf>,
}

impl Config {
    /// Pure function to create Config from Cli struct
    /// No side effects - pure transformation
    pub fn from_cli(cli: Cli) -> Self {
        // Clamp num_options between 1 and 10
        let num_options = cli.num_options.clamp(1, 10);

        // If --no-color is set, override color to Never
        // Otherwise use the --color flag value
        let color = if cli.no_color {
            ColorChoice::Never
        } else {
            cli.color
        };

        // Handle --debug-file flag
        // None = not provided, Some("") = use default, Some(path) = use custom path
        let debug_log_file = cli.debug_file.map(|path| {
            if path.is_empty() {
                Self::default_debug_log_path()
            } else {
                Self::expand_path(&path)
            }
        });

        Self {
            instruction: cli.instruction,
            model: cli.model,
            provider: cli.provider,
            quiet: cli.quiet,
            verbose: cli.verbose,
            no_color: cli.no_color,
            color,
            interactive: cli.interactive,
            force: cli.force,
            dry_run: cli.dry_run,
            context: cli.context,
            offline: cli.offline,
            num_options,
            debug: cli.debug,
            debug_log_file,
        }
    }

    /// Get default debug log path (~/.cache/clai/debug.log)
    pub fn default_debug_log_path() -> PathBuf {
        if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.cache_dir().join("clai").join("debug.log")
        } else {
            // Fallback if we can't determine cache dir
            PathBuf::from(".clai-debug.log")
        }
    }

    /// Expand ~ in path to home directory
    pub fn expand_path(path: &str) -> PathBuf {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(base_dirs) = directories::BaseDirs::new() {
                return base_dirs.home_dir().join(stripped);
            }
        }
        PathBuf::from(path)
    }
}

// Re-export file config types
pub mod cache;
pub mod file;
pub mod loader;
pub mod merger;
pub mod paths;
pub use cache::get_file_config;
pub use file::{
    ContextConfig, FileConfig, ProviderConfig, ProviderSpecificConfig, SafetyConfig, UiConfig,
};
pub use loader::{
    check_file_permissions, load_all_configs, load_config_file, resolve_env_var_reference,
    ConfigLoadError,
};
pub use merger::merge_all_configs;
pub use paths::{config_file_exists, discover_config_paths, existing_config_paths};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;

    #[test]
    fn test_config_from_cli_immutability() {
        let cli = Cli {
            instruction: "test".to_string(),
            model: Some("test-model".to_string()),
            provider: None,
            quiet: true,
            verbose: 2,
            no_color: true,
            color: crate::cli::ColorChoice::Auto,
            interactive: false,
            force: true,
            dry_run: false,
            context: None,
            offline: true,
            num_options: 3,
            debug: false,
            debug_file: None,
        };

        let config1 = Config::from_cli(cli.clone());
        let config2 = Config::from_cli(cli);

        // Verify immutability - both configs should be equal
        assert_eq!(config1, config2);

        // Verify all fields are correctly transformed
        assert_eq!(config1.instruction, "test");
        assert_eq!(config1.model, Some("test-model".to_string()));
        assert_eq!(config1.quiet, true);
        assert_eq!(config1.verbose, 2);
        assert_eq!(config1.offline, true);
        assert_eq!(config1.num_options, 3);
    }

    #[test]
    fn test_config_clone() {
        let cli = Cli {
            instruction: "clone test".to_string(),
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
            debug: false,
            debug_file: None,
        };

        let config = Config::from_cli(cli);
        let cloned = config.clone();

        // Verify clone creates identical immutable copy
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_num_options_clamping() {
        // Test that num_options is clamped between 1 and 10
        let cli_zero = Cli {
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
            num_options: 0,
            debug: false,
            debug_file: None,
        };
        let config = Config::from_cli(cli_zero);
        assert_eq!(config.num_options, 1); // Clamped to minimum 1

        let cli_high = Cli {
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
            num_options: 50,
            debug: false,
            debug_file: None,
        };
        let config = Config::from_cli(cli_high);
        assert_eq!(config.num_options, 10); // Clamped to maximum 10
    }
}
