use crate::cli::Cli;
use crate::config::file::FileConfig;
use crate::config::loader::load_all_configs;
use std::collections::HashMap;

/// Merge configurations from multiple sources in precedence order
///
/// Precedence (highest to lowest):
/// 1. CLI flags (highest priority)
/// 2. Environment variables (CLAI_*)
/// 3. Config files (in discovery order, highest priority first)
/// 4. Defaults (lowest priority)
///
/// Pure function - takes immutable inputs and returns merged config
/// No side effects (except reading environment variables)
pub fn merge_all_configs(cli: &Cli) -> Result<FileConfig, crate::config::loader::ConfigLoadError> {
    // Start with defaults
    let mut merged = FileConfig::default();

    // 1. Load config files (lowest priority in merge, but we'll override later)
    let file_config = load_all_configs()?;
    merged = merge_file_configs(merged, file_config);

    // 2. Apply environment variables (override files)
    let env_config = extract_env_config();
    merged = merge_env_config(merged, env_config);

    // 3. Apply CLI flags (highest priority, override everything)
    merged = merge_cli_config(merged, cli);

    Ok(merged)
}

/// Extract configuration from environment variables
///
/// Environment variables follow pattern: CLAI_<SECTION>_<FIELD>
/// Examples:
/// - CLAI_PROVIDER_DEFAULT
/// - CLAI_CONTEXT_MAX_FILES
/// - CLAI_UI_COLOR
///
/// Pure function - reads environment but doesn't modify state
fn extract_env_config() -> HashMap<String, String> {
    let mut env_config = HashMap::new();

    // Collect all CLAI_* environment variables
    for (key, value) in std::env::vars() {
        if let Some(stripped) = key.strip_prefix("CLAI_") {
            // Convert to lowercase for consistency
            let config_key = stripped.to_lowercase();
            env_config.insert(config_key, value);
        }
    }

    env_config
}

/// Merge file configs (deep merge for nested structures)
///
/// Pure function - takes two immutable configs and returns merged config
/// No side effects
fn merge_file_configs(base: FileConfig, override_config: FileConfig) -> FileConfig {
    // Deep merge: override_config takes precedence, but we merge nested structures
    FileConfig {
        provider: merge_provider_config(base.provider, override_config.provider),
        context: merge_context_config(base.context, override_config.context),
        safety: merge_safety_config(base.safety, override_config.safety),
        ui: merge_ui_config(base.ui, override_config.ui),
        providers: {
            // Merge provider-specific configs
            let mut merged = base.providers;
            for (key, value) in override_config.providers {
                merged.insert(key, value);
            }
            merged
        },
    }
}

/// Merge provider configs
fn merge_provider_config(
    base: crate::config::file::ProviderConfig,
    override_config: crate::config::file::ProviderConfig,
) -> crate::config::file::ProviderConfig {
    let default_provider = crate::config::file::ProviderConfig::default();
    crate::config::file::ProviderConfig {
        default: if override_config.default != default_provider.default {
            override_config.default
        } else {
            base.default
        },
        fallback: if !override_config.fallback.is_empty() {
            override_config.fallback
        } else {
            base.fallback
        },
    }
}

/// Merge context configs
///
/// For boolean fields, we check if override differs from default - if so, use override.
/// This allows explicit `false` in override to take precedence over `true` in base.
fn merge_context_config(
    base: crate::config::file::ContextConfig,
    override_config: crate::config::file::ContextConfig,
) -> crate::config::file::ContextConfig {
    let default_context = crate::config::file::ContextConfig::default();
    crate::config::file::ContextConfig {
        max_files: if override_config.max_files != default_context.max_files {
            override_config.max_files
        } else {
            base.max_files
        },
        max_history: if override_config.max_history != default_context.max_history {
            override_config.max_history
        } else {
            base.max_history
        },
        // For booleans: if override differs from default, use override; otherwise use base
        redact_paths: if override_config.redact_paths != default_context.redact_paths {
            override_config.redact_paths
        } else {
            base.redact_paths
        },
        redact_username: if override_config.redact_username != default_context.redact_username {
            override_config.redact_username
        } else {
            base.redact_username
        },
    }
}

/// Merge safety configs
fn merge_safety_config(
    base: crate::config::file::SafetyConfig,
    override_config: crate::config::file::SafetyConfig,
) -> crate::config::file::SafetyConfig {
    let default_safety = crate::config::file::SafetyConfig::default();
    crate::config::file::SafetyConfig {
        dangerous_patterns: if !override_config.dangerous_patterns.is_empty() {
            override_config.dangerous_patterns
        } else {
            base.dangerous_patterns
        },
        // For booleans: if override differs from default, use override; otherwise use base
        confirm_dangerous: if override_config.confirm_dangerous != default_safety.confirm_dangerous
        {
            override_config.confirm_dangerous
        } else {
            base.confirm_dangerous
        },
    }
}

/// Merge UI configs
fn merge_ui_config(
    base: crate::config::file::UiConfig,
    override_config: crate::config::file::UiConfig,
) -> crate::config::file::UiConfig {
    let default_ui = crate::config::file::UiConfig::default();
    crate::config::file::UiConfig {
        color: if override_config.color != default_ui.color {
            override_config.color
        } else {
            base.color
        },
        debug_log_file: override_config.debug_log_file.or(base.debug_log_file),
        // For booleans: if override differs from default, use override; otherwise use base
        interactive: if override_config.interactive != default_ui.interactive {
            override_config.interactive
        } else {
            base.interactive
        },
    }
}

/// Merge environment variable config into file config
///
/// Pure function - takes immutable inputs and returns merged config
/// No side effects
fn merge_env_config(base: FileConfig, env: HashMap<String, String>) -> FileConfig {
    let mut merged = base;

    // Parse environment variables and apply to config
    // Format: CLAI_<SECTION>_<FIELD> = value

    // Provider section
    if let Some(default) = env.get("provider_default") {
        merged.provider.default = default.clone();
    }
    if let Some(fallback) = env.get("provider_fallback") {
        // Parse comma-separated list
        merged.provider.fallback = fallback
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Context section
    if let Some(max_files) = env.get("context_max_files") {
        if let Ok(val) = max_files.parse::<u32>() {
            merged.context.max_files = val;
        }
    }
    if let Some(max_history) = env.get("context_max_history") {
        if let Ok(val) = max_history.parse::<u32>() {
            merged.context.max_history = val;
        }
    }
    if let Some(redact_paths) = env.get("context_redact_paths") {
        merged.context.redact_paths = redact_paths.parse().unwrap_or(false);
    }
    if let Some(redact_username) = env.get("context_redact_username") {
        merged.context.redact_username = redact_username.parse().unwrap_or(false);
    }

    // Safety section
    if let Some(patterns) = env.get("safety_dangerous_patterns") {
        merged.safety.dangerous_patterns = patterns
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    if let Some(confirm) = env.get("safety_confirm_dangerous") {
        merged.safety.confirm_dangerous = confirm.parse().unwrap_or(true);
    }

    // UI section
    if let Some(color) = env.get("ui_color") {
        merged.ui.color = color.clone();
    }

    merged
}

/// Merge CLI flags into config
///
/// Pure function - takes immutable inputs and returns merged config
/// No side effects
fn merge_cli_config(base: FileConfig, cli: &Cli) -> FileConfig {
    let mut merged = base;

    // Apply CLI flags (highest priority)
    // First, set provider if specified
    if let Some(provider) = &cli.provider {
        merged.provider.default = provider.clone();
    }

    // Then, set model if specified (use the provider, or default if not set)
    if let Some(model) = &cli.model {
        let provider_name = cli.provider.as_ref().unwrap_or(&merged.provider.default);
        // Find or create provider config
        if let Some(provider_config) = merged.providers.get_mut(provider_name) {
            provider_config.model = Some(model.clone());
        } else {
            // Create new provider config entry
            let provider_config = crate::config::file::ProviderSpecificConfig {
                model: Some(model.clone()),
                ..Default::default()
            };
            merged
                .providers
                .insert(provider_name.clone(), provider_config);
        }
    }

    // Note: Other CLI flags like --quiet, --verbose, --no-color, etc.
    // are runtime flags and don't affect the file config structure
    // They're handled separately in the runtime Config struct

    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;

    #[test]
    fn test_extract_env_config() {
        // Set test environment variables
        std::env::set_var("CLAI_PROVIDER_DEFAULT", "test-provider");
        std::env::set_var("CLAI_CONTEXT_MAX_FILES", "25");

        let env_config = extract_env_config();

        assert_eq!(
            env_config.get("provider_default"),
            Some(&"test-provider".to_string())
        );
        assert_eq!(env_config.get("context_max_files"), Some(&"25".to_string()));

        // Clean up
        std::env::remove_var("CLAI_PROVIDER_DEFAULT");
        std::env::remove_var("CLAI_CONTEXT_MAX_FILES");
    }

    #[test]
    fn test_merge_cli_config() {
        let base = FileConfig::default();
        let cli = Cli {
            instruction: "test".to_string(),
            model: Some("gpt-4".to_string()),
            provider: Some("openai".to_string()),
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

        let merged = merge_cli_config(base, &cli);

        assert_eq!(merged.provider.default, "openai");
        // Model should be set in the provider config
        assert!(merged.providers.get("openai").is_some());
    }

    #[test]
    fn test_merge_env_config() {
        let base = FileConfig::default();
        let mut env = HashMap::new();
        env.insert("provider_default".to_string(), "test-provider".to_string());
        env.insert("context_max_files".to_string(), "30".to_string());

        let merged = merge_env_config(base, env);

        assert_eq!(merged.provider.default, "test-provider");
        assert_eq!(merged.context.max_files, 30);
    }

    #[test]
    fn test_merge_file_configs() {
        let base = FileConfig::default();
        let mut override_config = FileConfig::default();
        override_config.context.max_files = 50;
        override_config.provider.default = "custom".to_string();

        let merged = merge_file_configs(base, override_config);

        assert_eq!(merged.context.max_files, 50);
        assert_eq!(merged.provider.default, "custom");
        // Other fields should remain from base (defaults)
        assert_eq!(merged.context.max_history, 3); // default
    }

    #[test]
    fn test_merge_precedence() {
        // Test that CLI overrides env, env overrides file, file overrides default
        let cli = Cli {
            instruction: "test".to_string(),
            provider: Some("cli-provider".to_string()),
            model: None,
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

        // Set env var
        std::env::set_var("CLAI_PROVIDER_DEFAULT", "env-provider");

        let merged = merge_all_configs(&cli).unwrap();

        // CLI should win
        assert_eq!(merged.provider.default, "cli-provider");

        // Clean up
        std::env::remove_var("CLAI_PROVIDER_DEFAULT");
    }
}
