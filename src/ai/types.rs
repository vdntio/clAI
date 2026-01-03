use serde::{Deserialize, Serialize};

/// Chat message role
/// 
/// Represents the role of a message in a chat conversation.
/// Used for building chat completion requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message - provides context and instructions
    System,
    /// User message - user input/instruction
    User,
    /// Assistant message - AI response
    Assistant,
}

/// Chat message
/// 
/// Immutable message structure for chat completions.
/// Contains role and content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of the message (system, user, or assistant)
    pub role: Role,
    /// Content of the message
    pub content: String,
}

impl ChatMessage {
    /// Create a new chat message
    /// 
    /// Pure function - creates immutable message
    /// 
    /// # Arguments
    /// * `role` - Message role
    /// * `content` - Message content
    /// 
    /// # Returns
    /// * `ChatMessage` - New message instance
    pub fn new(role: Role, content: String) -> Self {
        Self { role, content }
    }

    /// Create a system message
    /// 
    /// Convenience function for creating system messages
    /// 
    /// # Arguments
    /// * `content` - System message content
    /// 
    /// # Returns
    /// * `ChatMessage` - System message
    pub fn system(content: String) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a user message
    /// 
    /// Convenience function for creating user messages
    /// 
    /// # Arguments
    /// * `content` - User message content
    /// 
    /// # Returns
    /// * `ChatMessage` - User message
    pub fn user(content: String) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message
    /// 
    /// Convenience function for creating assistant messages
    /// 
    /// # Arguments
    /// * `content` - Assistant message content
    /// 
    /// # Returns
    /// * `ChatMessage` - Assistant message
    pub fn assistant(content: String) -> Self {
        Self::new(Role::Assistant, content)
    }
}

/// Chat completion request
/// 
/// Immutable request structure for AI chat completions.
/// Contains messages and optional model/provider selection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    /// List of messages in the conversation
    pub messages: Vec<ChatMessage>,
    /// Optional model identifier (e.g., "gpt-4", "claude-3-opus")
    /// If None, provider uses default model
    pub model: Option<String>,
    /// Optional temperature for response randomness (0.0 to 2.0)
    /// If None, provider uses default temperature
    pub temperature: Option<f64>,
    /// Optional maximum tokens in response
    /// If None, provider uses default max_tokens
    pub max_tokens: Option<u32>,
}

impl ChatRequest {
    /// Create a new chat request
    /// 
    /// Pure function - creates immutable request
    /// 
    /// # Arguments
    /// * `messages` - List of chat messages
    /// 
    /// # Returns
    /// * `ChatRequest` - New request instance
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages,
            model: None,
            temperature: None,
            max_tokens: None,
        }
    }

    /// Set the model for this request
    /// 
    /// Returns a new request with the model set.
    /// 
    /// # Arguments
    /// * `model` - Model identifier
    /// 
    /// # Returns
    /// * `ChatRequest` - New request with model set
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the temperature for this request
    /// 
    /// Returns a new request with the temperature set.
    /// 
    /// # Arguments
    /// * `temperature` - Temperature value (0.0 to 2.0)
    /// 
    /// # Returns
    /// * `ChatRequest` - New request with temperature set
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the max tokens for this request
    /// 
    /// Returns a new request with max_tokens set.
    /// 
    /// # Arguments
    /// * `max_tokens` - Maximum tokens in response
    /// 
    /// # Returns
    /// * `ChatRequest` - New request with max_tokens set
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
}

/// Chat completion response
/// 
/// Immutable response structure from AI providers.
/// Contains the generated message content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Generated message content
    pub content: String,
    /// Optional model used for generation
    pub model: Option<String>,
    /// Optional usage statistics (tokens used)
    pub usage: Option<Usage>,
}

/// Token usage statistics
/// 
/// Represents token usage for a completion request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    /// Number of prompt tokens
    pub prompt_tokens: u32,
    /// Number of completion tokens
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

impl ChatResponse {
    /// Create a new chat response
    /// 
    /// Pure function - creates immutable response
    /// 
    /// # Arguments
    /// * `content` - Generated message content
    /// 
    /// # Returns
    /// * `ChatResponse` - New response instance
    pub fn new(content: String) -> Self {
        Self {
            content,
            model: None,
            usage: None,
        }
    }

    /// Set the model for this response
    /// 
    /// Returns a new response with the model set.
    /// 
    /// # Arguments
    /// * `model` - Model identifier
    /// 
    /// # Returns
    /// * `ChatResponse` - New response with model set
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the usage statistics for this response
    /// 
    /// Returns a new response with usage set.
    /// 
    /// # Arguments
    /// * `usage` - Usage statistics
    /// 
    /// # Returns
    /// * `ChatResponse` - New response with usage set
    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = Some(usage);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_creation() {
        let msg = ChatMessage::system("You are a helpful assistant.".to_string());
        assert_eq!(msg.role, Role::System);
        assert_eq!(msg.content, "You are a helpful assistant.");

        let msg = ChatMessage::user("Hello".to_string());
        assert_eq!(msg.role, Role::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_chat_request_immutability() {
        let messages = vec![
            ChatMessage::system("System".to_string()),
            ChatMessage::user("User".to_string()),
        ];
        let req1 = ChatRequest::new(messages.clone());
        let req2 = ChatRequest::new(messages);

        // Should be equal (immutable)
        assert_eq!(req1.messages.len(), req2.messages.len());
    }

    #[test]
    fn test_chat_request_builder() {
        let messages = vec![ChatMessage::user("test".to_string())];
        let req = ChatRequest::new(messages)
            .with_model("gpt-4".to_string())
            .with_temperature(0.7)
            .with_max_tokens(100);

        assert_eq!(req.model, Some("gpt-4".to_string()));
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.max_tokens, Some(100));
    }

    #[test]
    fn test_chat_response_creation() {
        let resp = ChatResponse::new("Hello, world!".to_string());
        assert_eq!(resp.content, "Hello, world!");
        assert_eq!(resp.model, None);
        assert_eq!(resp.usage, None);
    }

    #[test]
    fn test_chat_response_builder() {
        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        };
        let resp = ChatResponse::new("test".to_string())
            .with_model("gpt-4".to_string())
            .with_usage(usage.clone());

        assert_eq!(resp.model, Some("gpt-4".to_string()));
        assert_eq!(resp.usage, Some(usage));
    }

    #[test]
    fn test_role_serialization() {
        let role = Role::System;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"system\"");

        let role = Role::User;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"user\"");

        let role = Role::Assistant;
        let serialized = serde_json::to_string(&role).unwrap();
        assert_eq!(serialized, "\"assistant\"");
    }

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage::system("test".to_string());
        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: ChatMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(msg, deserialized);
    }
}

