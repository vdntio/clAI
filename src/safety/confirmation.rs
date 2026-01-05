use crate::config::Config;
use crate::signals::is_stderr_tty;
use owo_colors::OwoColorize;
use std::io::{self, Write};

/// User decision for dangerous command handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Execute the command
    Execute,
    /// Copy the command to clipboard (or just output it)
    Copy,
    /// Abort and don't execute
    Abort,
}

/// Error types for confirmation handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmationError {
    /// EOF or pipe closed (stdin not available)
    Eof,
    /// Invalid input (not E, C, or A)
    InvalidInput(String),
    /// I/O error reading from stdin
    IoError(String),
}

impl std::fmt::Display for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfirmationError::Eof => write!(f, "EOF: stdin closed or piped"),
            ConfirmationError::InvalidInput(input) => {
                write!(f, "Invalid input: '{}'. Expected E, C, or A", input.trim())
            }
            ConfirmationError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for ConfirmationError {}

/// Handle dangerous command confirmation prompt
///
/// Displays a colored warning on stderr and prompts the user for confirmation.
/// Returns the user's decision: Execute, Copy, or Abort.
///
/// # Arguments
/// * `command` - The dangerous command that was detected
/// * `config` - Runtime configuration (for color settings)
///
/// # Returns
/// * `Result<Decision, ConfirmationError>` - User's decision or error
///
/// # Behavior
/// - Prints warning to stderr (not stdout, following UNIX philosophy)
/// - Prompts: `[E]xecute/[C]opy/[A]bort?`
/// - Reads single character (case-insensitive)
/// - Handles EOF/pipe gracefully (returns Abort)
/// - Respects color settings from config
///
/// # Examples
/// ```
/// use clai::safety::confirmation::{handle_dangerous_confirmation, Decision};
/// use clai::config::Config;
///
/// let config = Config::from_cli(cli);
/// match handle_dangerous_confirmation("rm -rf /", &config) {
///     Ok(Decision::Execute) => println!("Executing..."),
///     Ok(Decision::Copy) => println!("Copying..."),
///     Ok(Decision::Abort) => println!("Aborted"),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn handle_dangerous_confirmation(
    command: &str,
    config: &Config,
) -> Result<Decision, ConfirmationError> {
    // Check if stderr is a TTY (for colored output)
    let use_color = !config.no_color && is_stderr_tty();

    // Print warning to stderr (not stdout - following UNIX philosophy)
    let warning_text = format!("⚠️  DANGEROUS: {}", command);
    if use_color {
        eprintln!("{}", warning_text.yellow().bold());
    } else {
        eprintln!("{}", warning_text);
    }

    // Print prompt to stderr
    let prompt = "[E]xecute/[C]opy/[A]bort? ";
    eprint!("{}", prompt);

    // Flush stderr to ensure prompt is visible
    if let Err(e) = io::stderr().flush() {
        return Err(ConfirmationError::IoError(format!(
            "Failed to flush stderr: {}",
            e
        )));
    }

    // Read user input from stdin
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => {
            // EOF - stdin closed or piped
            // Return Abort as safe default
            eprintln!(); // Newline for clean output
            Ok(Decision::Abort)
        }
        Ok(_) => {
            // Parse input (trim whitespace, take first character, case-insensitive)
            let trimmed = input.trim();
            if trimmed.is_empty() {
                // Empty input - default to Abort
                Ok(Decision::Abort)
            } else {
                match trimmed
                    .chars()
                    .next()
                    .unwrap()
                    .to_uppercase()
                    .next()
                    .unwrap()
                {
                    'E' => Ok(Decision::Execute),
                    'C' => Ok(Decision::Copy),
                    'A' => Ok(Decision::Abort),
                    _ => Err(ConfirmationError::InvalidInput(input.trim().to_string())),
                }
            }
        }
        Err(e) => {
            // I/O error reading stdin
            Err(ConfirmationError::IoError(format!(
                "Failed to read from stdin: {}",
                e
            )))
        }
    }
}

/// Format decision as string for display
///
/// Pure function for converting Decision to string representation.
///
/// # Arguments
/// * `decision` - The decision to format
///
/// # Returns
/// * `&'static str` - String representation
pub fn format_decision(decision: Decision) -> &'static str {
    match decision {
        Decision::Execute => "Execute",
        Decision::Copy => "Copy",
        Decision::Abort => "Abort",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_decision() {
        assert_eq!(format_decision(Decision::Execute), "Execute");
        assert_eq!(format_decision(Decision::Copy), "Copy");
        assert_eq!(format_decision(Decision::Abort), "Abort");
    }

    #[test]
    fn test_confirmation_error_display() {
        let eof = ConfirmationError::Eof;
        assert!(eof.to_string().contains("EOF"));

        let invalid = ConfirmationError::InvalidInput("X".to_string());
        assert!(invalid.to_string().contains("Invalid input"));
        assert!(invalid.to_string().contains("X"));

        let io_err = ConfirmationError::IoError("test error".to_string());
        assert!(io_err.to_string().contains("I/O error"));
        assert!(io_err.to_string().contains("test error"));
    }

    // Note: Integration tests for handle_dangerous_confirmation would require
    // mocking stdin, which is complex. These are better suited for manual testing
    // or using a testing framework that can mock stdin/stdout/stderr.
    // The function is tested manually during development.
}
