use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration structure for TOML file parsing
/// Represents the complete config file structure with all sections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FileConfig {
    /// Provider configuration section
    #[serde(default)]
    pub provider: ProviderConfig,

    /// Context configuration section
    #[serde(default)]
    pub context: ContextConfig,

    /// Safety configuration section
    #[serde(default)]
    pub safety: SafetyConfig,

    /// UI configuration section
    #[serde(default)]
    pub ui: UiConfig,

    /// Provider-specific configurations (e.g., [openrouter], [ollama])
    #[serde(flatten)]
    pub providers: HashMap<String, ProviderSpecificConfig>,
}

/// Provider configuration section
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProviderConfig {
    /// Default provider to use
    #[serde(default = "default_provider_default")]
    pub default: String,

    /// Fallback providers in order
    #[serde(default)]
    pub fallback: Vec<String>,
}

/// Provider-specific configuration (e.g., [openrouter], [ollama])
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProviderSpecificConfig {
    /// API key directly stored in config (protected by 0600 file permissions)
    pub api_key: Option<String>,

    /// API key environment variable name (alternative to api_key)
    pub api_key_env: Option<String>,

    /// Model to use for this provider
    pub model: Option<String>,

    /// Endpoint URL (for local providers like Ollama)
    pub endpoint: Option<String>,
}

/// Context configuration section
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContextConfig {
    /// Maximum number of files to include in context
    #[serde(default = "default_max_files")]
    pub max_files: u32,

    /// Maximum number of history commands to include
    #[serde(default = "default_max_history")]
    pub max_history: u32,

    /// Whether to redact file paths before sending to API
    #[serde(default)]
    pub redact_paths: bool,

    /// Whether to redact usernames before sending to API
    #[serde(default)]
    pub redact_username: bool,
}

/// Safety configuration section
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SafetyConfig {
    /// List of dangerous command patterns to detect
    #[serde(default = "default_dangerous_patterns")]
    pub dangerous_patterns: Vec<String>,

    /// Whether to confirm dangerous commands interactively
    #[serde(default = "default_confirm_dangerous")]
    pub confirm_dangerous: bool,
}

/// UI configuration section
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UiConfig {
    /// Color mode: "auto", "always", or "never"
    #[serde(default = "default_color")]
    pub color: String,

    /// Debug log file path (enables file logging when set)
    #[serde(default)]
    pub debug_log_file: Option<String>,
}

// Default value functions for serde defaults

fn default_provider_default() -> String {
    "openrouter".to_string()
}

fn default_max_files() -> u32 {
    10
}

fn default_max_history() -> u32 {
    3
}

fn default_dangerous_patterns() -> Vec<String> {
    vec![
        "rm -rf".to_string(),
        "sudo rm".to_string(),
        "mkfs".to_string(),
        "dd if=".to_string(),
        "> /dev/".to_string(),
        "format".to_string(),
    ]
}

fn default_confirm_dangerous() -> bool {
    true
}

fn default_color() -> String {
    "auto".to_string()
}

/// Default configuration instance
/// Pure constant - immutable default values
impl Default for FileConfig {
    fn default() -> Self {
        Self {
            provider: ProviderConfig {
                default: default_provider_default(),
                fallback: Vec::new(),
            },
            context: ContextConfig {
                max_files: default_max_files(),
                max_history: default_max_history(),
                redact_paths: false,
                redact_username: false,
            },
            safety: SafetyConfig {
                dangerous_patterns: default_dangerous_patterns(),
                confirm_dangerous: default_confirm_dangerous(),
            },
            ui: UiConfig {
                color: default_color(),
                debug_log_file: None,
            },
            providers: HashMap::new(),
        }
    }
}

// Implement Default for nested structs using our default functions
impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            default: default_provider_default(),
            fallback: Vec::new(),
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_files: default_max_files(),
            max_history: default_max_history(),
            redact_paths: false,
            redact_username: false,
        }
    }
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            dangerous_patterns: default_dangerous_patterns(),
            confirm_dangerous: default_confirm_dangerous(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            color: default_color(),
            debug_log_file: None,
        }
    }
}

impl Default for ProviderSpecificConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_key_env: None,
            model: None,
            endpoint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FileConfig::default();

        assert_eq!(config.provider.default, "openrouter");
        assert_eq!(config.context.max_files, 10);
        assert_eq!(config.context.max_history, 3);
        assert_eq!(config.safety.dangerous_patterns.len(), 6);
        assert_eq!(config.ui.color, "auto");
        assert_eq!(config.safety.confirm_dangerous, true);
    }

    #[test]
    fn test_config_serialize_deserialize() {
        let config = FileConfig::default();

        // Serialize to TOML
        let toml_string = toml::to_string(&config).expect("Failed to serialize config");

        // Deserialize back
        let deserialized: FileConfig =
            toml::from_str(&toml_string).expect("Failed to deserialize config");

        // Verify values match
        assert_eq!(config.provider.default, deserialized.provider.default);
        assert_eq!(config.context.max_files, deserialized.context.max_files);
        assert_eq!(config.context.max_history, deserialized.context.max_history);
        assert_eq!(
            config.safety.dangerous_patterns,
            deserialized.safety.dangerous_patterns
        );
        assert_eq!(config.ui.color, deserialized.ui.color);
    }

    #[test]
    fn test_config_clone() {
        let config1 = FileConfig::default();
        let config2 = config1.clone();

        // Verify clone creates identical copy
        assert_eq!(config1, config2);
    }

    #[test]
    fn test_dangerous_patterns_default() {
        let config = FileConfig::default();
        let patterns = &config.safety.dangerous_patterns;

        assert!(patterns.contains(&"rm -rf".to_string()));
        assert!(patterns.contains(&"sudo rm".to_string()));
        assert!(patterns.contains(&"mkfs".to_string()));
        assert!(patterns.contains(&"dd if=".to_string()));
        assert!(patterns.contains(&"> /dev/".to_string()));
        assert!(patterns.contains(&"format".to_string()));
    }
}
