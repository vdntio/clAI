use crate::ai::types::{ChatMessage, ChatRequest};
use regex::Regex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Response format for multi-command generation
/// 
/// The AI returns a JSON object with a "commands" array
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsResponse {
    pub commands: Vec<String>,
}

/// Pre-compiled regex for extracting commands from markdown code fences
/// 
/// Matches:
/// - ```bash\ncommand\n```
/// - ```sh\ncommand\n```
/// - ```shell\ncommand\n```
/// - ```\ncommand\n```
/// 
/// Uses lazy static initialization for performance
static COMMAND_EXTRACTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Match code fences with optional language (bash, sh, shell) or no language
    // The (?s) flag makes . match newlines
    // Capture group 1 is the command content
    Regex::new(r"(?s)```(?:bash|sh|shell)?\s*\n(.*?)\n?```")
        .expect("Failed to compile command extraction regex")
});

/// Build prompt from system context, directory context, history, and user instruction
/// 
/// Pure function - concatenates context into a structured prompt string.
/// No side effects.
/// 
/// # Arguments
/// * `system_context` - System information (JSON string from context gathering)
/// * `dir_context` - Directory/file context (JSON string)
/// * `history` - Shell history commands (vector of strings)
/// * `instruction` - User's natural language instruction
/// 
/// # Returns
/// * `String` - Complete prompt string
pub fn build_prompt(
    system_context: &str,
    dir_context: &str,
    history: &[String],
    instruction: &str,
) -> String {
    let mut prompt = String::new();

    // System context
    prompt.push_str("System Context:\n");
    prompt.push_str(system_context);
    prompt.push_str("\n\n");

    // Directory context
    prompt.push_str("Directory Context:\n");
    prompt.push_str(dir_context);
    prompt.push_str("\n\n");

    // Shell history
    if !history.is_empty() {
        prompt.push_str("Recent Shell History:\n");
        for (i, cmd) in history.iter().enumerate() {
            prompt.push_str(&format!("  {}. {}\n", i + 1, cmd));
        }
        prompt.push_str("\n");
    }

    // User instruction
    prompt.push_str("User Instruction: ");
    prompt.push_str(instruction);
    prompt.push_str("\n\n");

    // System instruction
    prompt.push_str("Respond ONLY with the executable command. Do not include markdown code fences, explanations, or any other text. Just the command itself.");

    prompt
}

/// Extract command from AI response
/// 
/// Strips markdown code fences (```bash, ```sh, ```shell, or just ```)
/// and trims whitespace. If no code fences are found, returns the full
/// response trimmed.
/// 
/// Pure function - no side effects
/// 
/// # Arguments
/// * `response` - AI response text (may contain markdown)
/// 
/// # Returns
/// * `String` - Extracted command (trimmed, no markdown)
pub fn extract_command(response: &str) -> String {
    // Try to extract from code fences
    if let Some(captures) = COMMAND_EXTRACTION_REGEX.captures(response) {
        if let Some(command) = captures.get(1) {
            return command.as_str().trim().to_string();
        }
    }

    // Fallback: return full response trimmed
    response.trim().to_string()
}

/// Build chat request from prompt (single command)
/// 
/// Creates a ChatRequest with system message and user message.
/// 
/// Pure function - creates immutable request
/// 
/// # Arguments
/// * `prompt` - Complete prompt string
/// * `model` - Optional model identifier
/// 
/// # Returns
/// * `ChatRequest` - Chat completion request
pub fn build_chat_request(prompt: String, model: Option<String>) -> ChatRequest {
    let messages = vec![
        ChatMessage::system(
            "You are a helpful assistant that converts natural language instructions into executable shell commands. Respond with ONLY the command, no explanations or markdown.".to_string()
        ),
        ChatMessage::user(prompt),
    ];

    let mut request = ChatRequest::new(messages);
    if let Some(model) = model {
        request = request.with_model(model);
    }
    request
}

/// Build chat request for multiple command options
/// 
/// Creates a ChatRequest that instructs the AI to return multiple command
/// alternatives in JSON format.
/// 
/// Pure function - creates immutable request
/// 
/// # Arguments
/// * `prompt` - Complete prompt string with context
/// * `num_options` - Number of command options to generate (1-10)
/// * `model` - Optional model identifier
/// 
/// # Returns
/// * `ChatRequest` - Chat completion request for multiple commands
pub fn build_multi_chat_request(prompt: String, num_options: u8, model: Option<String>) -> ChatRequest {
    let system_prompt = format!(
        r#"You are a helpful assistant that converts natural language instructions into executable shell commands.

Generate exactly {} different command options that accomplish the user's goal.
Each command should be a valid, executable shell command.
Provide alternatives that vary in approach, verbosity, or options used.

IMPORTANT: Respond ONLY with a valid JSON object in this exact format:
{{"commands": ["command1", "command2", "command3"]}}

Rules:
- Return exactly {} commands in the "commands" array
- Each command must be a single string (escape quotes properly)
- No explanations, comments, or markdown - just the JSON object
- Commands should be practical alternatives, not duplicates
- Order from simplest/most common to more advanced/specific"#,
        num_options, num_options
    );

    let messages = vec![
        ChatMessage::system(system_prompt),
        ChatMessage::user(prompt),
    ];

    let mut request = ChatRequest::new(messages);
    if let Some(model) = model {
        request = request.with_model(model);
    }
    request
}

/// Extract multiple commands from AI response JSON
/// 
/// Parses the AI response which should be a JSON object with a "commands" array.
/// Handles various edge cases like markdown code fences wrapping JSON.
/// 
/// Pure function - no side effects
/// 
/// # Arguments
/// * `response` - AI response text (should be JSON)
/// 
/// # Returns
/// * `Result<Vec<String>, String>` - Extracted commands or error message
pub fn extract_commands(response: &str) -> Result<Vec<String>, String> {
    let response = response.trim();
    
    // Try to extract JSON from markdown code fences if present
    let json_str = if response.starts_with("```") {
        // Remove markdown code fences
        let without_start = response
            .strip_prefix("```json")
            .or_else(|| response.strip_prefix("```"))
            .unwrap_or(response);
        without_start
            .strip_suffix("```")
            .unwrap_or(without_start)
            .trim()
    } else {
        response
    };
    
    // Try to parse as CommandsResponse
    match serde_json::from_str::<CommandsResponse>(json_str) {
        Ok(parsed) => {
            if parsed.commands.is_empty() {
                Err("AI returned empty commands array".to_string())
            } else {
                // Filter out empty commands and trim whitespace
                let commands: Vec<String> = parsed
                    .commands
                    .into_iter()
                    .map(|c| c.trim().to_string())
                    .filter(|c| !c.is_empty())
                    .collect();
                
                if commands.is_empty() {
                    Err("All commands in AI response were empty".to_string())
                } else {
                    Ok(commands)
                }
            }
        }
        Err(e) => {
            // Try to extract from array directly (in case AI returns just an array)
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(json_str) {
                if arr.is_empty() {
                    return Err("AI returned empty array".to_string());
                }
                return Ok(arr.into_iter().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).collect());
            }
            
            // Fallback: try to find JSON object in response
            if let Some(start) = json_str.find('{') {
                if let Some(end) = json_str.rfind('}') {
                    let potential_json = &json_str[start..=end];
                    if let Ok(parsed) = serde_json::from_str::<CommandsResponse>(potential_json) {
                        if !parsed.commands.is_empty() {
                            return Ok(parsed.commands.into_iter().map(|c| c.trim().to_string()).filter(|c| !c.is_empty()).collect());
                        }
                    }
                }
            }
            
            // Last fallback: treat entire response as single command
            let single_cmd = extract_command(response);
            if !single_cmd.is_empty() {
                Ok(vec![single_cmd])
            } else {
                Err(format!("Failed to parse AI response as JSON: {}. Response: {}", e, response))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::types::Role;

    #[test]
    fn test_extract_command_with_bash_fence() {
        let response = "```bash\nls -la\n```";
        let command = extract_command(response);
        assert_eq!(command, "ls -la");
    }

    #[test]
    fn test_extract_command_with_sh_fence() {
        let response = "```sh\ncd /tmp\n```";
        let command = extract_command(response);
        assert_eq!(command, "cd /tmp");
    }

    #[test]
    fn test_extract_command_with_shell_fence() {
        let response = "```shell\ngrep -r \"test\" .\n```";
        let command = extract_command(response);
        // The regex captures everything between fences, including newlines
        // So we need to trim to handle the newline after "shell"
        assert_eq!(command.trim(), "grep -r \"test\" .");
    }

    #[test]
    fn test_extract_command_with_no_lang_fence() {
        let response = "```\nfind . -name '*.rs'\n```";
        let command = extract_command(response);
        assert_eq!(command, "find . -name '*.rs'");
    }

    #[test]
    fn test_extract_command_multi_line() {
        let response = "```bash\nfor i in {1..10}; do\n  echo $i\ndone\n```";
        let command = extract_command(response);
        assert_eq!(command, "for i in {1..10}; do\n  echo $i\ndone");
    }

    #[test]
    fn test_extract_command_no_fence() {
        let response = "ls -la";
        let command = extract_command(response);
        assert_eq!(command, "ls -la");
    }

    #[test]
    fn test_extract_command_with_explanation() {
        let response = "Here's the command:\n```bash\nls -la\n```\nThis will list all files.";
        let command = extract_command(response);
        assert_eq!(command, "ls -la");
    }

    #[test]
    fn test_extract_command_empty() {
        let response = "";
        let command = extract_command(response);
        assert_eq!(command, "");
    }

    #[test]
    fn test_extract_command_whitespace() {
        let response = "```bash\n  ls -la  \n```";
        let command = extract_command(response);
        assert_eq!(command, "ls -la");
    }

    #[test]
    fn test_build_prompt() {
        let system = r#"{"os_name": "Linux"}"#;
        let dir = r#"{"files": ["file1.txt"]}"#;
        let history = vec!["ls -la".to_string(), "cd /tmp".to_string()];
        let instruction = "list python files";

        let prompt = build_prompt(system, dir, &history, instruction);

        assert!(prompt.contains("System Context:"));
        assert!(prompt.contains("Directory Context:"));
        assert!(prompt.contains("Recent Shell History:"));
        assert!(prompt.contains("list python files"));
        assert!(prompt.contains("Respond ONLY with the executable command"));
    }

    #[test]
    fn test_build_prompt_no_history() {
        let system = r#"{"os_name": "Linux"}"#;
        let dir = r#"{"files": []}"#;
        let history = vec![];
        let instruction = "test";

        let prompt = build_prompt(system, dir, &history, instruction);

        assert!(!prompt.contains("Recent Shell History:"));
    }

    #[test]
    fn test_build_chat_request() {
        let prompt = "test prompt".to_string();
        let request = build_chat_request(prompt.clone(), Some("gpt-4".to_string()));

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, Role::System);
        assert_eq!(request.messages[1].role, Role::User);
        assert_eq!(request.messages[1].content, prompt);
        assert_eq!(request.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_build_chat_request_no_model() {
        let prompt = "test".to_string();
        let request = build_chat_request(prompt, None);

        // When model is None, it should be None (not set)
        assert_eq!(request.model, None);
    }

    #[test]
    fn test_build_multi_chat_request() {
        let prompt = "list files".to_string();
        let request = build_multi_chat_request(prompt.clone(), 3, Some("gpt-4".to_string()));

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, Role::System);
        assert!(request.messages[0].content.contains("3 different command options"));
        assert!(request.messages[0].content.contains("JSON"));
        assert_eq!(request.messages[1].content, prompt);
    }

    #[test]
    fn test_extract_commands_valid_json() {
        let response = r#"{"commands": ["ls -la", "ls -lah", "ls -l --color"]}"#;
        let result = extract_commands(response);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "ls -la");
        assert_eq!(commands[1], "ls -lah");
        assert_eq!(commands[2], "ls -l --color");
    }

    #[test]
    fn test_extract_commands_with_markdown() {
        let response = "```json\n{\"commands\": [\"ls -la\", \"ls -lah\"]}\n```";
        let result = extract_commands(response);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_extract_commands_array_only() {
        let response = r#"["ls -la", "ls -lah"]"#;
        let result = extract_commands(response);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_extract_commands_fallback_single() {
        // If AI returns plain text, fallback to single command
        let response = "ls -la";
        let result = extract_commands(response);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], "ls -la");
    }

    #[test]
    fn test_extract_commands_empty_array() {
        let response = r#"{"commands": []}"#;
        let result = extract_commands(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_commands_json_in_text() {
        let response = r#"Here's the response: {"commands": ["ls -la", "dir"]} Hope this helps!"#;
        let result = extract_commands(response);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_extract_commands_trims_whitespace() {
        let response = r#"{"commands": ["  ls -la  ", "  ls -lah  "]}"#;
        let result = extract_commands(response);
        assert!(result.is_ok());
        let commands = result.unwrap();
        assert_eq!(commands[0], "ls -la");
        assert_eq!(commands[1], "ls -lah");
    }
}

