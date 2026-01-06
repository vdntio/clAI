use crate::ai::provider::Provider;
use crate::ai::types::{ChatMessage, ChatRequest, ChatResponse, Role, Usage};
use crate::logging::FileLogger;
use anyhow::Result;
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

/// Global file logger instance (initialized once at startup)
static FILE_LOGGER: OnceCell<Arc<FileLogger>> = OnceCell::new();

/// Initialize the global file logger
///
/// Should be called once at application startup if file logging is enabled.
pub fn init_file_logger(logger: Arc<FileLogger>) {
    let _ = FILE_LOGGER.set(logger);
}

/// Get the global file logger if initialized
pub fn get_file_logger() -> Option<&'static Arc<FileLogger>> {
    FILE_LOGGER.get()
}

/// OpenRouter API endpoint
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Default model for OpenRouter (Qwen3 Coder)
const DEFAULT_OPENROUTER_MODEL: &str = "qwen/qwen3-coder";

/// OpenRouter provider implementation
///
/// Implements the Provider trait for OpenRouter API.
/// Uses OpenAI-compatible request/response format.
#[derive(Debug, Clone)]
pub struct OpenRouterProvider {
    /// HTTP client for making requests
    client: Client,
    /// API key for authentication
    api_key: String,
    /// Default model to use if not specified in request
    default_model: Option<String>,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider
    ///
    /// # Arguments
    /// * `api_key` - OpenRouter API key
    /// * `default_model` - Optional default model identifier
    ///
    /// # Returns
    /// * `OpenRouterProvider` - New provider instance
    pub fn new(api_key: String, default_model: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            default_model,
        }
    }

    /// Get API key from environment or config
    ///
    /// Checks for OPENROUTER_API_KEY environment variable.
    ///
    /// # Returns
    /// * `Option<String>` - API key if found
    pub fn api_key_from_env() -> Option<String> {
        std::env::var("OPENROUTER_API_KEY").ok()
    }

    /// Convert our ChatMessage to OpenAI format
    fn to_openai_message(msg: &ChatMessage) -> OpenAIMessage {
        OpenAIMessage {
            role: match msg.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: msg.content.clone(),
        }
    }

    /// Convert OpenAI response to our ChatResponse
    fn from_openai_response(resp: OpenAIResponse) -> ChatResponse {
        let content = resp
            .choices
            .first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let model = resp.model;
        let usage = resp.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        let mut response = ChatResponse::new(content).with_model(model);
        if let Some(usage) = usage {
            response = response.with_usage(usage);
        }
        response
    }

    /// Make API request with retry logic for rate limits
    async fn make_request_with_retry(&self, request: OpenAIRequest) -> Result<OpenAIResponse> {
        let mut retries = 3;
        let mut delay = Duration::from_secs(1);

        loop {
            match self.make_request(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    // Check if it's a rate limit error (429)
                    if e.to_string().contains("429") && retries > 0 {
                        retries -= 1;
                        tokio::time::sleep(delay).await;
                        delay *= 2; // Exponential backoff
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Make API request
    async fn make_request(&self, request: &OpenAIRequest) -> Result<OpenAIResponse> {
        let response = match self
            .client
            .post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/clai") // Optional attribution
            .header("X-Title", "clai") // Optional app name
            .json(request)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                // Log network error
                if let Some(logger) = get_file_logger() {
                    logger.log_error(
                        "network_error",
                        &e.to_string(),
                        Some(serde_json::json!({"url": OPENROUTER_API_URL})),
                    );
                }
                // Network/timeout errors - no status code
                return Err(anyhow::anyhow!(
                    "Network error: Failed to send request to OpenRouter: {}",
                    e
                )
                .context("API request failed"));
            }
        };

        let status = response.status();
        if !status.is_success() {
            let status_code = status.as_u16();
            let error_text = response.text().await.unwrap_or_default();

            // Log API error
            if let Some(logger) = get_file_logger() {
                logger.log_error(
                    "api_error",
                    &error_text,
                    Some(serde_json::json!({
                        "status_code": status_code,
                        "model": &request.model
                    })),
                );
            }

            // Distinguish error types for better error messages
            let error_msg = match status_code {
                401 | 403 => format!(
                    "Authentication error ({}): Invalid or missing API key. {}",
                    status_code, error_text
                ),
                429 => format!(
                    "Rate limit error ({}): Too many requests. {}",
                    status_code, error_text
                ),
                408 | 504 => format!(
                    "Timeout error ({}): Request timed out. {}",
                    status_code, error_text
                ),
                _ => format!("API error ({}): {}", status_code, error_text),
            };

            anyhow::bail!("{}", error_msg);
        }

        let api_response: OpenAIResponse = match response.json::<OpenAIResponse>().await {
            Ok(r) => r,
            Err(e) => {
                // Log parse error
                if let Some(logger) = get_file_logger() {
                    logger.log_error(
                        "parse_error",
                        &e.to_string(),
                        Some(serde_json::json!({"model": &request.model})),
                    );
                }
                return Err(anyhow::anyhow!(
                    "Failed to parse OpenRouter response: {}",
                    e
                ));
            }
        };

        Ok(api_response)
    }
}

#[async_trait::async_trait]
impl Provider for OpenRouterProvider {
    async fn complete(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Determine model to use
        // Priority: request.model > provider default > global default
        let model = request
            .model
            .clone()
            .or_else(|| self.default_model.clone())
            .unwrap_or_else(|| DEFAULT_OPENROUTER_MODEL.to_string());

        // Log the request before sending (with full message content)
        if let Some(logger) = get_file_logger() {
            logger.log_request(
                Some(&model),
                &request.messages,
                request.temperature,
                request.max_tokens,
            );
        }

        // Convert messages to OpenAI format
        let messages: Vec<OpenAIMessage> = request
            .messages
            .iter()
            .map(Self::to_openai_message)
            .collect();

        // Build OpenAI-compatible request
        let openai_request = OpenAIRequest {
            model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        // Make request with retry logic
        let response = self.make_request_with_retry(openai_request).await?;

        // Log the response
        if let Some(logger) = get_file_logger() {
            let content = response
                .choices
                .first()
                .map(|c| c.message.content.as_str())
                .unwrap_or("");
            let usage = response.usage.as_ref().map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            });
            logger.log_response(Some(&response.model), 200, content, usage.as_ref());
        }

        // Convert to our response format
        Ok(Self::from_openai_response(response))
    }

    fn name(&self) -> &str {
        "openrouter"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// OpenAI-compatible message format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI-compatible request format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

/// OpenAI-compatible response format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIResponse {
    id: Option<String>,
    model: String,
    choices: Vec<Choice>,
    usage: Option<OpenAIUsage>,
}

/// Choice in OpenAI response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Choice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

/// Usage in OpenAI response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::ChatMessage;

    #[test]
    fn test_openrouter_provider_creation() {
        let provider = OpenRouterProvider::new("test-key".to_string(), None);
        assert_eq!(provider.name(), "openrouter");
        assert!(provider.is_available());
    }

    #[test]
    fn test_openrouter_provider_no_api_key() {
        let provider = OpenRouterProvider::new("".to_string(), None);
        assert!(!provider.is_available());
    }

    #[test]
    fn test_to_openai_message() {
        let msg = ChatMessage::system("test".to_string());
        let openai_msg = OpenRouterProvider::to_openai_message(&msg);
        assert_eq!(openai_msg.role, "system");
        assert_eq!(openai_msg.content, "test");
    }

    #[test]
    fn test_from_openai_response() {
        let openai_resp = OpenAIResponse {
            id: Some("test-id".to_string()),
            model: "gpt-4".to_string(),
            choices: vec![Choice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: "Hello, world!".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(OpenAIUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        };

        let resp = OpenRouterProvider::from_openai_response(openai_resp);
        assert_eq!(resp.content, "Hello, world!");
        assert_eq!(resp.model, Some("gpt-4".to_string()));
        assert!(resp.usage.is_some());
    }
}
