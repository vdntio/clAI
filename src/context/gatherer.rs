use crate::cli::Cli;
use crate::config::{get_file_config, Config};
use crate::context::directory::scan_directory;
use crate::context::history::get_shell_history;
use crate::context::stdin::read_stdin_default;
use crate::context::system::get_formatted_system_info;
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::env;

/// Context data structure for gathering
/// Immutable snapshot of all context information
#[derive(Debug, Clone)]
pub struct ContextData {
    pub system: HashMap<String, String>,
    pub cwd: String,
    pub files: Vec<String>,
    pub history: Vec<String>,
    pub stdin: Option<String>,
}

/// Gather all context information and format as structured JSON
/// 
/// This is the main orchestrator function that:
/// 1. Collects system information
/// 2. Gets current working directory
/// 3. Scans directory for files
/// 4. Reads shell history
/// 5. Reads stdin if piped
/// 6. Applies redaction if configured
/// 7. Formats everything as pretty-printed JSON
/// 
/// Pure function after I/O operations - returns immutable String
/// 
/// # Arguments
/// * `config` - Configuration with context settings (max_files, max_history, redact_paths, etc.)
/// 
/// # Returns
/// * `Result<String>` - Pretty-printed JSON string, or error
pub fn gather_context(config: &Config) -> Result<String> {
    // Get system information
    let system = get_formatted_system_info();

    // Get current working directory
    let cwd = env::current_dir()
        .context("Failed to get current working directory")?
        .to_string_lossy()
        .to_string();

    // Get file config for context settings
    // Use defaults if file config not available
    let cli = Cli {
        instruction: config.instruction.clone(),
        model: config.model.clone(),
        provider: config.provider.clone(),
        quiet: config.quiet,
        verbose: config.verbose,
        no_color: config.no_color,
        color: config.color,
        interactive: config.interactive,
        force: config.force,
        dry_run: config.dry_run,
        context: config.context.clone(),
        offline: config.offline,
        num_options: config.num_options,
    };
    let file_config = get_file_config(&cli).unwrap_or_default();

    // Scan directory for files
    let max_files = file_config.context.max_files;
    let redact_paths = file_config.context.redact_paths;
    let files = scan_directory(max_files, redact_paths);

    // Get shell history
    let max_history = file_config.context.max_history;
    let history = get_shell_history(max_history);

    // Read stdin if piped
    let stdin = read_stdin_default();

    // Build context data structure
    let context_data = ContextData {
        system,
        cwd: if redact_paths {
            crate::context::directory::redact_path_internal(&cwd)
        } else {
            cwd
        },
        files,
        history,
        stdin,
    };

    // Format as JSON
    format_context_json(&context_data)
}


/// Format context data as pretty-printed JSON
/// 
/// Converts ContextData into a structured JSON object with 2-space indentation.
/// 
/// Pure function - no side effects
/// 
/// # Arguments
/// * `data` - Context data to format
/// 
/// # Returns
/// * `Result<String>` - Pretty-printed JSON string, or error
fn format_context_json(data: &ContextData) -> Result<String> {
    // Build JSON object
    let mut json_obj = json!({
        "system": data.system,
        "cwd": data.cwd,
        "files": data.files,
        "history": data.history,
    });

    // Add stdin if present
    if let Some(ref stdin_content) = data.stdin {
        json_obj["stdin"] = json!(stdin_content);
    } else {
        json_obj["stdin"] = json!(null);
    }

    // Pretty-print with 2-space indentation
    serde_json::to_string_pretty(&json_obj)
        .context("Failed to serialize context to JSON")
}

/// Get context as JSON string (convenience function)
/// 
/// Wrapper around gather_context that handles errors gracefully.
/// 
/// # Arguments
/// * `config` - Configuration with context settings
/// 
/// # Returns
/// * `String` - JSON string (empty on error)
pub fn get_context_json(config: &Config) -> String {
    gather_context(config).unwrap_or_else(|e| {
        // On error, return minimal context
        json!({
            "error": format!("Failed to gather context: {}", e),
            "system": {},
            "cwd": "",
            "files": [],
            "history": [],
            "stdin": null
        })
        .to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use crate::config::Config;

    fn create_test_config() -> Config {
        Config {
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
        }
    }

    #[test]
    fn test_format_context_json() {
        let data = ContextData {
            system: {
                let mut map = HashMap::new();
                map.insert("os_name".to_string(), "Linux".to_string());
                map.insert("shell".to_string(), "bash".to_string());
                map
            },
            cwd: "/home/test".to_string(),
            files: vec!["file1.txt".to_string(), "file2.txt".to_string()],
            history: vec!["ls -la".to_string(), "cd /tmp".to_string()],
            stdin: Some("test input".to_string()),
        };

        let json_str = format_context_json(&data).unwrap();
        
        // Verify it's valid JSON
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        
        assert!(parsed.get("system").is_some());
        assert!(parsed.get("cwd").is_some());
        assert!(parsed.get("files").is_some());
        assert!(parsed.get("history").is_some());
        assert!(parsed.get("stdin").is_some());
    }

    #[test]
    fn test_format_context_json_no_stdin() {
        let data = ContextData {
            system: HashMap::new(),
            cwd: "/home/test".to_string(),
            files: vec![],
            history: vec![],
            stdin: None,
        };

        let json_str = format_context_json(&data).unwrap();
        
        // Verify it's valid JSON
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        
        assert_eq!(parsed.get("stdin").unwrap().as_null(), Some(()));
    }

    #[test]
    fn test_gather_context() {
        let config = create_test_config();
        
        // This will actually gather real context
        let result = gather_context(&config);
        
        // Should succeed (unless we're in a weird test environment)
        if let Ok(json_str) = result {
            // Verify it's valid JSON
            let parsed: Value = serde_json::from_str(&json_str).unwrap();
            
            assert!(parsed.get("system").is_some());
            assert!(parsed.get("cwd").is_some());
            assert!(parsed.get("files").is_some());
            assert!(parsed.get("history").is_some());
            assert!(parsed.get("stdin").is_some());
        }
    }

    #[test]
    fn test_get_context_json() {
        let config = create_test_config();
        
        // Should always return a string (even on error)
        let json_str = get_context_json(&config);
        
        // Verify it's valid JSON
        let parsed: Value = serde_json::from_str(&json_str).unwrap();
        
        assert!(parsed.get("system").is_some());
    }
}

