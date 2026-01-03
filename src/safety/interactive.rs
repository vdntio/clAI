use crate::config::Config;
use crate::signals::is_stderr_tty;
use crossterm::cursor::{MoveToColumn, MoveUp};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::ExecutableCommand;
use owo_colors::OwoColorize;
use std::io::{self, Write};

/// User action for command handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    /// Execute the command directly
    Execute,
    /// Output the command (for user to edit/run manually)
    Output,
    /// Abort and don't do anything
    Abort,
}

/// Error types for interactive command handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractiveError {
    /// EOF or pipe closed (stdin not available)
    Eof,
    /// I/O error reading from terminal
    IoError(String),
    /// Terminal not available (not a TTY)
    NotTty,
    /// No commands provided
    NoCommands,
}

impl std::fmt::Display for InteractiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InteractiveError::Eof => write!(f, "EOF: stdin closed or piped"),
            InteractiveError::IoError(msg) => write!(f, "I/O error: {}", msg),
            InteractiveError::NotTty => write!(f, "Not a TTY: interactive mode requires a terminal"),
            InteractiveError::NoCommands => write!(f, "No commands provided"),
        }
    }
}

impl std::error::Error for InteractiveError {}

/// Prompt user to select from command options with Tab cycling
/// 
/// Shows the generated command(s) and prompts for action:
/// - Tab: Cycle to next command option (inline replacement)
/// - Enter: Execute the currently selected command
/// - Ctrl+C or Esc: Abort
/// 
/// Uses crossterm for raw mode terminal input to read single keypresses.
/// 
/// # Arguments
/// * `commands` - Slice of command options (at least one required)
/// * `config` - Runtime configuration (for color settings)
/// 
/// # Returns
/// * `Result<(CommandAction, String), InteractiveError>` - User's action and selected command
/// 
/// # Behavior
/// - Prints command to stderr (not stdout, following UNIX philosophy)
/// - Tab cycles through options, replacing the command inline
/// - Shows indicator `[1/3]` when multiple options exist
/// - Enter executes the currently selected command
/// - Handles EOF/pipe gracefully (returns Output with first command)
/// - Respects color settings from config
/// 
/// # Examples
/// ```ignore
/// use clai::safety::interactive::{prompt_command_action, CommandAction};
/// use clai::config::Config;
/// 
/// let commands = vec!["ls -la".to_string(), "ls -lah".to_string()];
/// match prompt_command_action(&commands, &config) {
///     Ok((CommandAction::Execute, cmd)) => println!("Executing: {}", cmd),
///     Ok((CommandAction::Output, cmd)) => println!("{}", cmd),
///     Ok((CommandAction::Abort, _)) => println!("Aborted"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn prompt_command_action(
    commands: &[String],
    config: &Config,
) -> Result<(CommandAction, String), InteractiveError> {
    // Validate input
    if commands.is_empty() {
        return Err(InteractiveError::NoCommands);
    }
    
    // Check if stderr is a TTY (required for interactive mode)
    if !is_stderr_tty() {
        // Not a TTY - default to output first command (safe for piping)
        return Ok((CommandAction::Output, commands[0].clone()));
    }

    let use_color = !config.no_color;
    let total = commands.len();
    let mut selected_index: usize = 0;

    // Get stderr for crossterm commands
    let mut stderr = io::stderr();

    // Build the prompt text (used for redraw)
    let prompt = if total > 1 {
        "Press Tab to cycle, Enter to execute, or Ctrl+C to cancel: "
    } else {
        "Press Enter to execute, or Ctrl+C to cancel: "
    };

    // Helper to format command text
    let format_command = |cmd: &str, idx: usize| -> String {
        if total > 1 {
            format!("Command [{}/{}]: {}", idx + 1, total, cmd)
        } else {
            format!("Command: {}", cmd)
        }
    };

    // Display initial command and prompt
    let initial_text = format_command(&commands[selected_index], selected_index);
    if use_color {
        eprintln!("{}", initial_text.cyan());
    } else {
        eprintln!("{}", initial_text);
    }
    eprint!("{}", prompt);
    stderr.flush().map_err(|e| InteractiveError::IoError(format!("Failed to flush: {}", e)))?;

    // Enable raw mode to read single keypresses
    enable_raw_mode().map_err(|e| {
        InteractiveError::IoError(format!("Failed to enable raw mode: {}", e))
    })?;

    // Read keypresses in a loop
    let result = loop {
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code,
                modifiers,
                kind: KeyEventKind::Press,
                ..
            })) => {
                // Check for Ctrl+C first
                if modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
                    && code == KeyCode::Char('c')
                {
                    break Ok((CommandAction::Abort, String::new()));
                }
                
                // Handle other keys
                match code {
                    KeyCode::Tab => {
                        // Cycle to next command
                        selected_index = (selected_index + 1) % total;
                        
                        // Use crossterm commands to update display:
                        // 1. Move up one line (to the command line)
                        // 2. Move to column 0
                        // 3. Clear the entire line
                        // 4. Print new command
                        // 5. Move to next line
                        // 6. Clear prompt line
                        // 7. Reprint prompt
                        
                        let _ = stderr.execute(MoveUp(1));
                        let _ = stderr.execute(MoveToColumn(0));
                        let _ = stderr.execute(Clear(ClearType::CurrentLine));
                        
                        let cmd_text = format_command(&commands[selected_index], selected_index);
                        if use_color {
                            eprintln!("{}", cmd_text.cyan());
                        } else {
                            eprintln!("{}", cmd_text);
                        }
                        
                        // Clear current line (prompt line) and reprint
                        let _ = stderr.execute(MoveToColumn(0));
                        let _ = stderr.execute(Clear(ClearType::CurrentLine));
                        eprint!("{}", prompt);
                        let _ = stderr.flush();
                        
                        continue;
                    }
                    KeyCode::Enter => {
                        break Ok((CommandAction::Execute, commands[selected_index].clone()));
                    }
                    KeyCode::Esc => {
                        break Ok((CommandAction::Abort, String::new()));
                    }
                    _ => {
                        // Ignore other keys, keep waiting
                        continue;
                    }
                }
            }
            Ok(_) => {
                // Ignore non-key events
                continue;
            }
            Err(e) => {
                break Err(InteractiveError::IoError(format!("Failed to read keypress: {}", e)));
            }
        }
    };

    // Disable raw mode
    if let Err(e) = disable_raw_mode() {
        eprintln!("\nWarning: Failed to disable raw mode: {}", e);
    }

    // Print newline for clean output
    eprintln!();

    result
}

/// Execute a command directly using std::process::Command
/// 
/// Spawns the command as a child process and waits for it to complete.
/// Returns the exit code of the command.
/// 
/// # Arguments
/// * `command` - The command to execute (will be parsed by shell)
/// 
/// # Returns
/// * `Result<i32, String>` - Exit code of command or error message
pub fn execute_command(command: &str) -> Result<i32, String> {
    use std::process::Command;

    // Detect shell from environment
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    // Execute command using shell
    let status = Command::new(&shell)
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    Ok(status.code().unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_command_simple() {
        // Test executing a simple command
        let result = execute_command("echo test");
        // Should succeed (exit code 0)
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_execute_command_failure() {
        // Test executing a failing command
        let result = execute_command("false");
        // Should return non-zero exit code
        assert!(result.is_ok());
        assert_ne!(result.unwrap(), 0);
    }

    #[test]
    fn test_empty_commands_returns_error() {
        use clap::Parser;
        let cli = crate::cli::Cli::parse_from(["clai", "test instruction"]);
        let config = crate::config::Config::from_cli(cli);
        
        let commands: Vec<String> = vec![];
        let result = prompt_command_action(&commands, &config);
        
        assert!(result.is_err());
        match result {
            Err(InteractiveError::NoCommands) => (),
            _ => panic!("Expected NoCommands error"),
        }
    }

    // Note: Integration tests for prompt_command_action with TTY interaction
    // would require a TTY and user interaction, which is complex to test automatically.
    // These are better suited for manual testing.
}
