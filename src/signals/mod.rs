use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Exit codes following UNIX conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Success (0)
    Success = 0,
    /// Invalid arguments (2)
    InvalidArgs = 2,
    /// Command interrupted by SIGINT (130)
    Interrupted = 130,
    /// General error (1)
    GeneralError = 1,
    /// User aborted dangerous command (5)
    Aborted = 5,
}

impl ExitCode {
    /// Convert to i32 for process::exit()
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Initialize signal handlers
/// Sets up handlers for SIGINT, SIGTERM, and SIGPIPE
/// Returns an Arc<AtomicBool> that can be checked for interruption
pub fn setup_signal_handlers() -> Arc<AtomicBool> {
    let interrupted = Arc::new(AtomicBool::new(false));

    // Handle SIGINT (Ctrl+C) - exit with code 130
    {
        let flag = Arc::clone(&interrupted);
        signal_hook::flag::register(signal_hook::consts::SIGINT, flag.clone())
            .expect("Failed to register SIGINT handler");
    }

    // Handle SIGTERM - clean shutdown
    {
        let flag = Arc::clone(&interrupted);
        signal_hook::flag::register(signal_hook::consts::SIGTERM, flag.clone())
            .expect("Failed to register SIGTERM handler");
    }

    // Handle SIGPIPE - silently ignore (common for pipe operations)
    // SIGPIPE is automatically ignored in Rust by default on Unix systems
    // On Windows, broken pipes are handled via errors, not signals
    // No explicit handler needed - Rust's default behavior is correct

    interrupted
}

/// Check if the process was interrupted by a signal
/// Pure function - reads atomic state
pub fn is_interrupted(flag: &Arc<AtomicBool>) -> bool {
    flag.load(Ordering::Relaxed)
}

/// Check if stdout is a TTY (for interactive behavior detection)
/// Pure function - no side effects
pub fn is_stdout_tty() -> bool {
    atty::is(atty::Stream::Stdout)
}

/// Check if stdin is a TTY (for interactive behavior detection)
/// Pure function - no side effects
pub fn is_stdin_tty() -> bool {
    atty::is(atty::Stream::Stdin)
}

/// Check if stderr is a TTY (for color output)
/// Pure function - no side effects
pub fn is_stderr_tty() -> bool {
    atty::is(atty::Stream::Stderr)
}

/// Determine if the process is running in interactive mode
/// Interactive = both stdin and stdout are TTYs
/// Pure function - no side effects
pub fn is_interactive() -> bool {
    is_stdin_tty() && is_stdout_tty()
}

/// Determine if output is being piped
/// Piped = stdout is not a TTY
/// Pure function - no side effects
pub fn is_piped() -> bool {
    !is_stdout_tty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success.as_i32(), 0);
        assert_eq!(ExitCode::InvalidArgs.as_i32(), 2);
        assert_eq!(ExitCode::Interrupted.as_i32(), 130);
        assert_eq!(ExitCode::GeneralError.as_i32(), 1);
        assert_eq!(ExitCode::Aborted.as_i32(), 5);
    }

    #[test]
    fn test_tty_detection_pure() {
        // These are pure functions - they should return consistent results
        // in the same environment
        let result1 = is_stdout_tty();
        let result2 = is_stdout_tty();
        assert_eq!(result1, result2, "TTY detection should be consistent");
    }

    #[test]
    fn test_is_interactive_pure() {
        // Pure function - same input (environment), same output
        let result1 = is_interactive();
        let result2 = is_interactive();
        assert_eq!(result1, result2, "Interactive detection should be consistent");
    }

    #[test]
    fn test_is_piped_pure() {
        // Pure function - same input (environment), same output
        let result1 = is_piped();
        let result2 = is_piped();
        assert_eq!(result1, result2, "Pipe detection should be consistent");
    }
}

