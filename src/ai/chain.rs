use crate::ai::provider::Provider;
use crate::ai::providers::openrouter::OpenRouterProvider;
use crate::ai::types::{ChatRequest, ChatResponse};
use crate::config::file::FileConfig;
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Provider chain for fallback support
///
/// Implements the Provider trait and tries each provider in sequence
/// until one succeeds. Supports lazy initialization of providers.
pub struct ProviderChain {
    /// List of provider names in fallback order
    providers: Vec<String>,
    /// Lazy-initialized provider instances (with interior mutability)
    provider_instances: Arc<Mutex<Vec<Option<Arc<dyn Provider>>>>>,
    /// File config for provider settings
    config: FileConfig,
}

impl ProviderChain {
    /// Create a new provider chain from config
    ///
    /// # Arguments
    /// * `config` - File configuration with provider settings
    ///
    /// # Returns
    /// * `ProviderChain` - New chain instance
    pub fn new(config: FileConfig) -> Self {
        // Get fallback chain from config
        let mut providers = config.provider.fallback.clone();

        // Add default provider to the front if not already in chain
        let default = config.provider.default.clone();
        if !providers.contains(&default) {
            providers.insert(0, default);
        }

        Self {
            providers,
            provider_instances: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    /// Initialize a provider by name
    ///
    /// Lazy initialization - creates provider instance on first access.
    ///
    /// # Arguments
    /// * `name` - Provider name (e.g., "openrouter", "ollama")
    ///
    /// # Returns
    /// * `Result<Arc<dyn Provider>>` - Provider instance or error
    fn init_provider(&self, name: &str) -> Result<Arc<dyn Provider>> {
        match name {
            "openrouter" => {
                // Get API key from config or environment
                // Priority: 1) api_key in config, 2) api_key_env in config, 3) OPENROUTER_API_KEY env var
                let openrouter_config = self.config.providers.get("openrouter");
                
                let api_key = openrouter_config
                    .and_then(|c| c.api_key.clone())
                    .or_else(|| {
                        openrouter_config
                            .and_then(|c| c.api_key_env.as_ref())
                            .and_then(|env_var| std::env::var(env_var).ok())
                    })
                    .or_else(|| OpenRouterProvider::api_key_from_env())
                    .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not found"))?;

                // Get model from config (defaults to KimiK2 if not set)
                let model = self
                    .config
                    .providers
                    .get("openrouter")
                    .and_then(|c| c.model.clone());

                let provider = OpenRouterProvider::new(api_key, model);
                Ok(Arc::new(provider))
            }
            _ => anyhow::bail!("Unknown provider: {}", name),
        }
    }

    /// Get or initialize a provider by index
    ///
    /// # Arguments
    /// * `index` - Provider index in chain
    ///
    /// # Returns
    /// * `Result<Arc<dyn Provider>>` - Provider instance or error
    fn get_provider(&self, index: usize) -> Result<Arc<dyn Provider>> {
        let mut instances = self.provider_instances.lock().unwrap();

        // Check if already initialized
        if let Some(Some(provider)) = instances.get(index) {
            return Ok(provider.clone());
        }

        // Initialize provider
        let provider_name = self
            .providers
            .get(index)
            .ok_or_else(|| anyhow::anyhow!("Provider index out of bounds"))?;

        let provider = self.init_provider(provider_name)?;

        // Cache the provider
        if instances.len() <= index {
            instances.resize(index + 1, None);
        }
        instances[index] = Some(provider.clone());

        Ok(provider)
    }

    /// Parse model string to extract provider and model
    ///
    /// Supports formats:
    /// - "provider/model" (e.g., "openrouter/gpt-4o")
    /// - "model" (uses default provider)
    ///
    /// # Arguments
    /// * `model_str` - Model string to parse
    ///
    /// # Returns
    /// * `(String, String)` - (provider_name, model_name)
    pub fn parse_model(&self, model_str: &str) -> (String, String) {
        if let Some((provider, model)) = model_str.split_once('/') {
            (provider.to_string(), model.to_string())
        } else {
            // Use default provider
            (self.config.provider.default.clone(), model_str.to_string())
        }
    }

    /// Get the list of provider names in fallback order
    ///
    /// # Returns
    /// * `&[String]` - Provider names
    pub fn providers(&self) -> &[String] {
        &self.providers
    }
}

impl std::fmt::Debug for ProviderChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderChain")
            .field("providers", &self.providers)
            .field(
                "provider_instances",
                &format!("<{} cached>", self.provider_instances.lock().unwrap().len()),
            )
            .field("config", &self.config)
            .finish()
    }
}

#[async_trait::async_trait]
impl Provider for ProviderChain {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Try each provider in sequence
        let mut last_error = None;

        for (index, provider_name) in self.providers.iter().enumerate() {
            // Get or initialize provider
            let provider = match self.get_provider(index) {
                Ok(p) => p,
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            };

            // Check if provider is available
            if !provider.is_available() {
                last_error = Some(anyhow::anyhow!(
                    "Provider {} is not available",
                    provider_name
                ));
                continue;
            }

            // Try to complete request
            match provider.complete(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Provider {} failed: {}", provider_name, e));
                    // Continue to next provider
                    continue;
                }
            }
        }

        // All providers failed
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All providers in chain failed")))
    }

    fn name(&self) -> &str {
        "provider-chain"
    }

    fn is_available(&self) -> bool {
        // Chain is available if at least one provider is available
        self.providers.iter().any(|name| {
            // Quick check without full initialization
            match name.as_str() {
                "openrouter" => OpenRouterProvider::api_key_from_env().is_some(),
                _ => false,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::file::{
        ContextConfig, FileConfig, ProviderConfig, ProviderSpecificConfig, SafetyConfig, UiConfig,
    };
    use std::collections::HashMap;

    fn create_test_config() -> FileConfig {
        let mut providers = HashMap::new();
        providers.insert(
            "openrouter".to_string(),
            ProviderSpecificConfig {
                api_key: None,
                api_key_env: Some("OPENROUTER_API_KEY".to_string()),
                model: Some("openai/gpt-4o".to_string()),
                endpoint: None,
            },
        );

        FileConfig {
            provider: ProviderConfig {
                default: "openrouter".to_string(),
                fallback: vec!["openrouter".to_string()],
            },
            context: ContextConfig::default(),
            safety: SafetyConfig::default(),
            ui: UiConfig::default(),
            providers,
        }
    }

    #[test]
    fn test_provider_chain_creation() {
        let config = create_test_config();
        let chain = ProviderChain::new(config);

        assert_eq!(chain.providers().len(), 1);
        assert_eq!(chain.providers()[0], "openrouter");
    }

    // Note: ProviderChain doesn't implement Clone because it uses Arc<Mutex<...>>
    // This is intentional for thread-safe lazy initialization

    #[test]
    fn test_parse_model_with_provider() {
        let config = create_test_config();
        let chain = ProviderChain::new(config);

        let (provider, model) = chain.parse_model("openrouter/gpt-4o");
        assert_eq!(provider, "openrouter");
        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_parse_model_without_provider() {
        let config = create_test_config();
        let chain = ProviderChain::new(config);

        let (provider, model) = chain.parse_model("gpt-4o");
        assert_eq!(provider, "openrouter"); // Uses default
        assert_eq!(model, "gpt-4o");
    }

    #[test]
    fn test_provider_chain_fallback_order() {
        let mut config = create_test_config();
        config.provider.fallback = vec!["openrouter".to_string(), "ollama".to_string()];
        config.provider.default = "openrouter".to_string();

        let chain = ProviderChain::new(config);
        let providers = chain.providers();

        // Should have default first, then fallbacks
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0], "openrouter");
        assert_eq!(providers[1], "ollama");
    }
}
