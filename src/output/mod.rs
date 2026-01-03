use crate::config::Config;
use crate::signals::is_stdout_tty;
use std::io::{self, Write};

/// Pure function to format output message
/// Takes immutable Config and returns formatted string
/// No side effects - pure function
pub fn format_output(config: &Config) -> String {
    format!("Command would be generated for: {}", config.instruction)
}

/// Print command to stdout with proper piped handling
/// 
/// If stdout is piped (not a TTY), prints without trailing newline.
/// If stdout is a TTY, prints with trailing newline.
/// 
/// This follows UNIX philosophy: piped output should be clean for further processing.
/// 
/// # Arguments
/// * `command` - The command string to print
/// 
/// # Side Effects
/// * Writes to stdout (this is the only function with side effects in this module)
pub fn print_command(command: &str) -> io::Result<()> {
    let is_piped = !is_stdout_tty();
    
    if is_piped {
        // Piped output: no newline (clean for further processing)
        print!("{}", command.trim());
        io::stdout().flush()
    } else {
        // TTY output: with newline (user-friendly)
        println!("{}", command.trim());
        Ok(())
    }
}

/// Pure function to format debug/config output
/// Returns formatted string representation of config
/// No side effects - pure function
pub fn format_config_debug(config: &Config) -> String {
    format!("Parsed config: {:?}", config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_output_pure() {
        let config = Config {
            instruction: "test instruction".to_string(),
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
        };

        let output = format_output(&config);
        assert_eq!(output, "Command would be generated for: test instruction");
        
        // Verify pure function - same input, same output
        let output2 = format_output(&config);
        assert_eq!(output, output2);
    }

    #[test]
    fn test_format_config_debug_pure() {
        let config = Config {
            instruction: "debug test".to_string(),
            model: Some("model".to_string()),
            provider: None,
            quiet: true,
            verbose: 1,
            no_color: true,
            color: crate::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
        };

        let debug = format_config_debug(&config);
        assert!(debug.contains("debug test"));
        assert!(debug.contains("model"));
        
        // Verify pure function - same input, same output
        let debug2 = format_config_debug(&config);
        assert_eq!(debug, debug2);
    }

    // Note: print_command tests would require mocking stdout/TTY state
    // which is complex. Integration tests are better suited for this.
}

