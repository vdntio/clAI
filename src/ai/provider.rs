use crate::ai::types::{ChatRequest, ChatResponse};
use anyhow::Result;

/// Provider trait for AI chat completions
///
/// This trait defines the interface for all AI providers.
/// Implementations must be thread-safe (Send + Sync) to support
/// concurrent usage.
///
/// Uses async-trait to enable async methods in traits.
#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    /// Complete a chat request
    ///
    /// Sends a chat completion request to the AI provider and returns
    /// the generated response.
    ///
    /// # Arguments
    /// * `request` - Chat completion request
    ///
    /// # Returns
    /// * `Result<ChatResponse>` - Generated response or error
    ///
    /// # Errors
    /// Returns an error if:
    /// - API request fails (network, timeout, etc.)
    /// - API returns an error response (auth, rate limit, etc.)
    /// - Response parsing fails
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse>;

    /// Get the provider name
    ///
    /// Returns a human-readable name for this provider.
    ///
    /// # Returns
    /// * `&str` - Provider name
    fn name(&self) -> &str;

    /// Check if the provider is available
    ///
    /// Returns true if the provider is configured and available.
    /// For local providers (e.g., Ollama), this may check if the
    /// service is running.
    ///
    /// # Returns
    /// * `bool` - True if provider is available
    fn is_available(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::ChatMessage;

    /// Mock provider for testing
    struct MockProvider {
        name: String,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        async fn complete(&self, _request: ChatRequest) -> Result<ChatResponse> {
            if self.should_fail {
                anyhow::bail!("Mock provider failure");
            }
            Ok(ChatResponse::new("Mock response".to_string()))
        }

        fn name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_provider_trait() {
        let provider = MockProvider {
            name: "mock".to_string(),
            should_fail: false,
        };

        let request = ChatRequest::new(vec![ChatMessage::user("test".to_string())]);
        let response = provider.complete(request).await.unwrap();

        assert_eq!(response.content, "Mock response");
        assert_eq!(provider.name(), "mock");
        assert!(provider.is_available());
    }

    #[tokio::test]
    async fn test_provider_error_handling() {
        let provider = MockProvider {
            name: "mock".to_string(),
            should_fail: true,
        };

        let request = ChatRequest::new(vec![ChatMessage::user("test".to_string())]);
        let result = provider.complete(request).await;

        assert!(result.is_err());
    }

    // Note: Provider trait cannot be used as a trait object (dyn Provider) in stable Rust
    // because async methods are not object-safe. This is a limitation of async traits.
    // The trait can still be used with generics (e.g., `impl Provider` or `P: Provider`).
    // For dynamic dispatch with async, consider using the async-trait crate or
    // wrapping in a type-erased future.
}
