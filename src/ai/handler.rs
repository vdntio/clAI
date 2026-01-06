use crate::ai::chain::ProviderChain;
use crate::ai::prompt::{
    build_chat_request, build_multi_chat_request, build_prompt, extract_command, extract_commands,
};
use crate::ai::provider::Provider;
use crate::config::{get_file_config, Config};
use crate::context::gatherer::gather_context;
use anyhow::{Context, Result};

/// Build context and prompt from configuration
///
/// Shared helper that gathers context and builds the prompt string.
/// Pure function after I/O operations.
///
/// # Arguments
/// * `config` - Runtime configuration
///
/// # Returns
/// * `Result<String>` - Built prompt string or error
fn build_context_prompt(config: &Config) -> Result<String> {
    // Gather context
    let context_json = gather_context(config).context("Failed to gather context")?;

    // Parse context JSON to extract components
    let context: serde_json::Value =
        serde_json::from_str(&context_json).context("Failed to parse context JSON")?;

    // Extract components from context
    let system_context = context
        .get("system")
        .map(|s| serde_json::to_string(s).unwrap_or_default())
        .unwrap_or_default();

    let dir_context = format!(
        "Current directory: {}\nFiles: {}",
        context.get("cwd").and_then(|c| c.as_str()).unwrap_or(""),
        context
            .get("files")
            .and_then(|f| f.as_array())
            .map(|arr| arr.len().to_string())
            .unwrap_or_else(|| "0".to_string())
    );

    let history: Vec<String> = context
        .get("history")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let stdin_context = context
        .get("stdin")
        .and_then(|s| s.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| format!("Stdin input: {}", s));

    // Build prompt
    let mut prompt = build_prompt(&system_context, &dir_context, &history, &config.instruction);

    // Add stdin context if present
    if let Some(stdin) = stdin_context {
        prompt.push_str(&format!("\n\n{}", stdin));
    }

    Ok(prompt)
}

/// Create provider chain from configuration
///
/// Helper that creates the AI provider chain with proper model parsing.
///
/// # Arguments
/// * `config` - Runtime configuration
///
/// # Returns
/// * `(ProviderChain, Option<String>)` - Provider chain and parsed model
fn create_provider_chain(config: &Config) -> (ProviderChain, Option<String>) {
    // Get file config for provider chain
    let cli = crate::cli::Cli {
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
        debug: config.debug,
        debug_file: config
            .debug_log_file
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
    };

    let file_config = get_file_config(&cli).unwrap_or_default();

    // Create provider chain
    let chain = ProviderChain::new(file_config);

    // Parse model if provided
    let model = config.model.as_ref().map(|m| {
        let (provider, model_name) = chain.parse_model(m);
        if provider == chain.providers()[0] {
            // Model is for the primary provider
            model_name
        } else {
            // Keep full "provider/model" format
            m.clone()
        }
    });

    (chain, model)
}

/// Handle AI command generation (single command)
///
/// Orchestrates the full flow:
/// 1. Gather context (system, directory, history, stdin)
/// 2. Build prompt from context and instruction
/// 3. Create chat request
/// 4. Call provider chain
/// 5. Extract command from response
///
/// Pure function after I/O operations - returns immutable String
///
/// # Arguments
/// * `config` - Runtime configuration
///
/// # Returns
/// * `Result<String>` - Generated command or error
pub async fn generate_command(config: &Config) -> Result<String> {
    let prompt = build_context_prompt(config)?;
    let (chain, model) = create_provider_chain(config);

    // Build chat request for single command
    let request = build_chat_request(prompt, model);

    // Debug output: show the request that will be sent to AI
    if config.debug {
        eprintln!("\n=== DEBUG: Request to be sent to AI ===");
        eprintln!("Model: {:?}", request.model);
        eprintln!("Temperature: {:?}", request.temperature);
        eprintln!("Max Tokens: {:?}", request.max_tokens);
        eprintln!("\nMessages:");
        for (i, msg) in request.messages.iter().enumerate() {
            eprintln!("  {}. Role: {:?}", i + 1, msg.role);
            eprintln!("     Content: {}", msg.content);
            if i < request.messages.len() - 1 {
                eprintln!();
            }
        }
        eprintln!("=== END DEBUG ===\n");
    }

    // Call provider chain
    let response = chain
        .complete(request)
        .await
        .context("Failed to get response from AI provider")?;

    // Extract command
    let command = extract_command(&response.content);

    Ok(command)
}

/// Handle AI command generation (multiple options)
///
/// Orchestrates the full flow for generating multiple command alternatives:
/// 1. Gather context (system, directory, history, stdin)
/// 2. Build prompt from context and instruction
/// 3. Create multi-command chat request (requests JSON array response)
/// 4. Call provider chain
/// 5. Parse JSON response to extract commands
///
/// Falls back to single command extraction if JSON parsing fails.
///
/// Pure function after I/O operations - returns immutable Vec<String>
///
/// # Arguments
/// * `config` - Runtime configuration
///
/// # Returns
/// * `Result<Vec<String>>` - Generated commands or error
pub async fn generate_commands(config: &Config) -> Result<Vec<String>> {
    let prompt = build_context_prompt(config)?;
    let (chain, model) = create_provider_chain(config);

    // Build chat request for multiple commands
    let request = build_multi_chat_request(prompt, config.num_options, model);

    // Debug output: show the request that will be sent to AI
    if config.debug {
        eprintln!("\n=== DEBUG: Request to be sent to AI ===");
        eprintln!("Model: {:?}", request.model);
        eprintln!("Temperature: {:?}", request.temperature);
        eprintln!("Max Tokens: {:?}", request.max_tokens);
        eprintln!("Number of options requested: {}", config.num_options);
        eprintln!("\nMessages:");
        for (i, msg) in request.messages.iter().enumerate() {
            eprintln!("  {}. Role: {:?}", i + 1, msg.role);
            eprintln!("     Content: {}", msg.content);
            if i < request.messages.len() - 1 {
                eprintln!();
            }
        }
        eprintln!("=== END DEBUG ===\n");
    }

    // Call provider chain
    let response = chain
        .complete(request)
        .await
        .context("Failed to get response from AI provider")?;

    // Extract commands from JSON response
    let commands = extract_commands(&response.content)
        .map_err(|e| anyhow::anyhow!("Failed to parse AI response: {}", e))?;

    // Ensure we have at least one command
    if commands.is_empty() {
        return Err(anyhow::anyhow!("AI returned no commands"));
    }

    Ok(commands)
}
