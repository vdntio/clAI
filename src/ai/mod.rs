pub mod chain;
pub mod handler;
pub mod prompt;
pub mod provider;
pub mod providers;
pub mod types;

pub use chain::ProviderChain;
pub use handler::{generate_command, generate_commands};
pub use prompt::{build_chat_request, build_multi_chat_request, build_prompt, extract_command, extract_commands, CommandsResponse};
pub use provider::Provider;
pub use providers::openrouter::OpenRouterProvider;
pub use types::{ChatMessage, ChatRequest, ChatResponse, Role};

