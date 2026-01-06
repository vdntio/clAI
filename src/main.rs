use clai::ai::handler::{generate_command, generate_commands};
use clai::ai::providers::openrouter::init_file_logger;
use clai::cli::parse_args;
use clai::config::{get_file_config, Config};
use clai::error::ClaiError;
use clai::logging::{FileLogger, Logger};
use clai::output::print_command;
use clai::safety::{
    execute_command, handle_dangerous_confirmation, is_dangerous_command, prompt_command_action,
    should_prompt, CommandAction, Decision,
};
use clai::signals::{is_interactive, is_interrupted, setup_signal_handlers, ExitCode};
use regex::Regex;
use std::process;
use std::sync::Arc;

/// Main entry point - orchestrates pure function composition
/// I/O side effects are isolated to this function
/// Signal handling and exit codes follow UNIX conventions
///
/// Uses Result-based error handling with ClaiError for proper exit codes
#[tokio::main]
async fn main() {
    // Setup signal handlers early (SIGINT, SIGTERM, SIGPIPE)
    let interrupt_flag = setup_signal_handlers();

    // Check for interruption before starting
    if is_interrupted(&interrupt_flag) {
        process::exit(ExitCode::Interrupted.as_i32());
    }

    // Function composition: parse_args() |> build_config() |> handle_cli()
    let result = run_main(&interrupt_flag).await;

    // Check for interruption before handling result
    if is_interrupted(&interrupt_flag) {
        process::exit(ExitCode::Interrupted.as_i32());
    }

    // Handle result and exit with appropriate code
    match result {
        Ok(()) => process::exit(ExitCode::Success.as_i32()),
        Err(err) => {
            // Log error to file if file logging is enabled
            err.log_to_file();

            // Get verbosity level from parsed CLI args
            // Parse args again just to get verbosity (lightweight operation)
            let verbose = parse_args().map(|cli| cli.verbose).unwrap_or(0);

            // Print error to stderr with optional backtrace
            err.print_stderr(verbose);
            process::exit(err.exit_code() as i32);
        }
    }
}

/// Extract HTTP status code from error message
///
/// Looks for patterns like "(401)", "(429)", etc. in error messages
/// Returns the status code if found, None otherwise
fn extract_status_code(error_msg: &str) -> Option<u16> {
    // Pattern: "(401)", "(429)", etc.
    static STATUS_CODE_RE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"\((\d{3})\)").unwrap());

    STATUS_CODE_RE
        .captures(error_msg)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u16>().ok())
}

/// Core main logic with Result-based error handling
///
/// Returns Result<(), ClaiError> for proper error propagation
async fn run_main(interrupt_flag: &Arc<std::sync::atomic::AtomicBool>) -> Result<(), ClaiError> {
    // Parse CLI arguments - convert clap::Error to ClaiError::Usage
    let cli = parse_args().map_err(ClaiError::from)?;

    // Check for offline mode first
    if cli.offline {
        return Err(ClaiError::General(anyhow::anyhow!(
            "Offline mode is not yet supported. Please remove --offline flag or configure a local provider (e.g., Ollama)."
        )));
    }

    // Load file config (lazy-loaded, cached after first access)
    // Missing config files are non-fatal (use defaults)
    // Parse/permission errors are fatal (exit code 3)
    let (file_config, was_config_missing) = match get_file_config(&cli) {
        Ok(config) => (config, false),
        Err(e) => {
            // Check if it's a non-fatal error (file not found)
            match &e {
                clai::config::loader::ConfigLoadError::NotFound(_) => {
                    // Missing config file is non-fatal - use defaults
                    (clai::config::FileConfig::default(), true)
                }
                _ => {
                    // Parse errors, permission errors, etc. are fatal
                    // Convert to ClaiError::Config (exit code 3)
                    return Err(ClaiError::from(e));
                }
            }
        }
    };

    // Create runtime config from CLI (CLI flags take precedence over file config)
    let config = Config::from_cli(cli);

    // Initialize file logger if enabled
    if let Some(ref log_path) = config.debug_log_file {
        match FileLogger::new(log_path.clone()) {
            Ok(logger) => {
                init_file_logger(Arc::new(logger));
                if config.verbose >= 1 {
                    eprintln!("Debug logging enabled: {}", log_path.display());
                }
            }
            Err(e) => {
                // Non-fatal: warn but continue
                eprintln!("Warning: Could not initialize debug log: {}", e);
            }
        }
    }

    // Log missing config file info if verbose
    if was_config_missing && config.verbose >= 1 {
        eprintln!("Info: No config file found, using defaults");
    }

    // Handle CLI logic - convert errors appropriately
    handle_cli(config, file_config, interrupt_flag).await?;

    Ok(())
}

/// Async function to handle CLI logic
/// Takes immutable Config and returns Result<(), ClaiError>
/// Side effects (I/O) are isolated to this function
/// Strict stdout/stderr separation: stdout = commands only, stderr = logs/warnings
/// Checks for signal interruption during execution
/// Integrates safety checks for dangerous commands
///
/// Converts errors to appropriate ClaiError variants:
/// - AI/API errors -> ClaiError::API
/// - Safety rejections -> ClaiError::Safety
/// - I/O errors -> ClaiError::General
async fn handle_cli(
    config: Config,
    file_config: clai::config::FileConfig,
    interrupt_flag: &Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), ClaiError> {
    // Check for interruption before processing
    if is_interrupted(interrupt_flag) {
        return Err(ClaiError::General(anyhow::anyhow!("Interrupted by signal")));
    }

    // Create logger from config (handles verbosity and color detection)
    let logger = Logger::from_config(&config);

    // Debug output to stderr only (respects quiet/verbose flags)
    if config.verbose >= 2 {
        logger.debug(&format!("Parsed config: {:?}", config));
    } else if config.verbose >= 1 {
        logger.info(&format!("Parsed config: {:?}", config));
    }

    // Check for interruption after logging
    if is_interrupted(interrupt_flag) {
        return Err(ClaiError::General(anyhow::anyhow!("Interrupted by signal")));
    }

    // Generate commands using AI
    // Use multi-command generation if num_options > 1 and interactive mode
    let commands_result = if config.num_options > 1 && config.interactive {
        generate_commands(&config).await
    } else {
        // Single command mode - wrap in vec for uniform handling
        generate_command(&config).await.map(|cmd| vec![cmd])
    };

    // Generate commands - convert AI errors to ClaiError::API
    // Extract HTTP status code from error message if available
    let commands = commands_result.map_err(|e| {
        let error_str = e.to_string();
        let status_code = extract_status_code(&error_str);

        ClaiError::API {
            source: anyhow::Error::from(e).context("Failed to generate command from AI provider"),
            status_code,
        }
    })?;

    // Check for interruption before output
    if is_interrupted(interrupt_flag) {
        return Err(ClaiError::General(anyhow::anyhow!("Interrupted by signal")));
    }

    // Process commands
    // Get first command for non-interactive modes
    let first_command = commands.first().cloned().unwrap_or_default();

    // Handle --dry-run flag: always print and exit (bypass safety checks)
    if config.dry_run {
        // Main output to stdout ONLY (clean for piping)
        // For dry-run, output all commands (one per line)
        // Use print_command for proper piped handling
        for (i, cmd) in commands.iter().enumerate() {
            if i > 0 {
                // Add newline between commands when multiple
                print!("\n");
            }
            print_command(cmd).map_err(|e| {
                ClaiError::General(
                    anyhow::Error::from(e).context("Failed to write command to stdout"),
                )
            })?;
        }
        // Ensure final newline for dry-run (user-friendly)
        if !commands.is_empty() {
            println!();
        }
        return Ok(());
    }

    // Check if first command is dangerous (for safety flow)
    let is_dangerous = is_dangerous_command(&first_command, &file_config);

    // Check if we're in interactive mode (TTY + interactive flag)
    let is_interactive_mode = config.interactive && is_interactive();

    // Handle dangerous commands
    if is_dangerous {
        // Check if we should prompt (TTY + config enabled + not forced)
        let should_prompt_user = should_prompt(
            &clai::cli::Cli {
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
            },
            &file_config,
        );

        if should_prompt_user {
            // Prompt user for confirmation (dangerous command)
            // Use first command for dangerous prompt (safety takes priority)
            match handle_dangerous_confirmation(&first_command, &config) {
                Ok(Decision::Execute) => {
                    // User chose to execute - print to stdout
                    print_command(&first_command).map_err(|e| {
                        ClaiError::General(
                            anyhow::Error::from(e).context("Failed to write command to stdout"),
                        )
                    })?;
                    Ok(())
                }
                Ok(Decision::Copy) => {
                    // User chose to copy - print to stdout (clipboard support can be added later)
                    print_command(&first_command).map_err(|e| {
                        ClaiError::General(
                            anyhow::Error::from(e).context("Failed to write command to stdout"),
                        )
                    })?;
                    Ok(())
                }
                Ok(Decision::Abort) => {
                    // User chose to abort - return Safety error
                    Err(ClaiError::Safety("Command rejected by user".to_string()))
                }
                Err(e) => {
                    // Error during confirmation (e.g., EOF) - default to abort
                    Err(ClaiError::Safety(format!(
                        "Error during confirmation: {}. Command rejected.",
                        e
                    )))
                }
            }
        } else {
            // Not prompting (piped, force, or config disabled) - print to stdout
            // Following UNIX philosophy: when piped, output goes to stdout
            print_command(&first_command).map_err(|e| {
                ClaiError::General(
                    anyhow::Error::from(e).context("Failed to write command to stdout"),
                )
            })?;
            Ok(())
        }
    } else if is_interactive_mode {
        // Safe command(s) in interactive mode - prompt for action with Tab cycling
        match prompt_command_action(&commands, &config) {
            Ok((CommandAction::Execute, selected_command)) => {
                // User pressed Enter - execute the selected command
                let exit_code = execute_command(&selected_command).map_err(|e| {
                    ClaiError::General(anyhow::Error::msg(e).context("Failed to execute command"))
                })?;

                if exit_code == 0 {
                    Ok(())
                } else {
                    Err(ClaiError::General(anyhow::anyhow!(
                        "Command exited with code {}",
                        exit_code
                    )))
                }
            }
            Ok((CommandAction::Output, selected_command)) => {
                // User chose to output - print to stdout (they can edit/run manually)
                print_command(&selected_command).map_err(|e| {
                    ClaiError::General(
                        anyhow::Error::from(e).context("Failed to write command to stdout"),
                    )
                })?;
                Ok(())
            }
            Ok((CommandAction::Abort, _)) => {
                // User chose to abort (Ctrl+C or Esc)
                Err(ClaiError::Safety("Command rejected by user".to_string()))
            }
            Err(e) => {
                // Error during prompt (e.g., not TTY) - default to output first
                eprintln!("Warning: {}. Outputting command.", e);
                print_command(&first_command).map_err(|e| {
                    ClaiError::General(
                        anyhow::Error::from(e).context("Failed to write command to stdout"),
                    )
                })?;
                Ok(())
            }
        }
    } else {
        // Command is safe and not interactive - print first command to stdout
        print_command(&first_command).map_err(|e| {
            ClaiError::General(anyhow::Error::from(e).context("Failed to write command to stdout"))
        })?;
        Ok(())
    }
}
