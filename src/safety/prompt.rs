use crate::cli::Cli;
use crate::config::file::FileConfig;
use crate::signals::{is_stdin_tty, is_stdout_tty};

/// Determine if we should prompt the user for dangerous command confirmation
///
/// Pure function that checks all conditions for interactive prompting:
/// - Must be in a TTY (stdin and stdout)
/// - Config must have confirm_dangerous enabled
/// - CLI must not have --force flag
///
/// # Arguments
/// * `cli` - CLI arguments
/// * `config` - File configuration
///
/// # Returns
/// * `bool` - `true` if we should prompt, `false` otherwise
///
/// # Examples
/// ```ignore
/// use clap::Parser;
/// use clai::cli::Cli;
/// use clai::config::file::FileConfig;
/// use clai::safety::prompt::should_prompt;
///
/// // Cli is a clap-derived struct; construct via parse_from.
/// // See crate::cli::Cli for full field definitions.
/// let cli = Cli::parse_from(&["clai", "your instruction here"]);
/// let config = FileConfig::default();
/// // Result depends on TTY state
/// let result = should_prompt(&cli, &config);
/// ```
pub fn should_prompt(cli: &Cli, config: &FileConfig) -> bool {
    // Check if we're in a TTY (both stdin and stdout)
    let is_tty = is_stdin_tty() && is_stdout_tty();

    // Check config setting
    let confirm_enabled = config.safety.confirm_dangerous;

    // Check if --force flag is set (bypasses prompting)
    let force_bypass = cli.force;

    // Should prompt if: TTY && confirm enabled && not forced
    is_tty && confirm_enabled && !force_bypass
}

/// Check if we're in interactive mode (TTY)
///
/// Pure function that checks if both stdin and stdout are TTYs.
///
/// # Returns
/// * `bool` - `true` if interactive (TTY), `false` if piped
pub fn is_interactive_mode() -> bool {
    is_stdin_tty() && is_stdout_tty()
}

/// Check if output is piped (not a TTY)
///
/// Pure function that checks if stdout is not a TTY.
///
/// # Returns
/// * `bool` - `true` if piped, `false` if TTY
pub fn is_piped_output() -> bool {
    !is_stdout_tty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::file::FileConfig;
    use clap::Parser;

    fn create_test_cli(force: bool) -> crate::cli::Cli {
        // Create a minimal Cli for testing
        if force {
            crate::cli::Cli::parse_from(&["clai", "--force", "test instruction"])
        } else {
            crate::cli::Cli::parse_from(&["clai", "test instruction"])
        }
    }

    #[test]
    fn test_should_prompt_requires_tty() {
        let cli = create_test_cli(false);
        let mut config = FileConfig::default();
        config.safety.confirm_dangerous = true;

        // Result depends on actual TTY state, but logic is correct
        let result = should_prompt(&cli, &config);
        // If we're in a TTY, should prompt; if piped, should not
        // This test verifies the logic, not the TTY state
        assert_eq!(
            result,
            is_interactive_mode() && config.safety.confirm_dangerous && !cli.force
        );
    }

    #[test]
    fn test_should_prompt_respects_force_flag() {
        let cli_forced = create_test_cli(true);
        let cli_not_forced = create_test_cli(false);
        let mut config = FileConfig::default();
        config.safety.confirm_dangerous = true;

        let result_forced = should_prompt(&cli_forced, &config);
        let result_not_forced = should_prompt(&cli_not_forced, &config);

        // Force should always disable prompting
        assert!(!result_forced);
        // Not forced should respect other conditions
        assert_eq!(
            result_not_forced,
            is_interactive_mode() && config.safety.confirm_dangerous
        );
    }

    #[test]
    fn test_should_prompt_respects_config() {
        let cli = create_test_cli(false);
        let mut config_enabled = FileConfig::default();
        config_enabled.safety.confirm_dangerous = true;

        let mut config_disabled = FileConfig::default();
        config_disabled.safety.confirm_dangerous = false;

        let result_enabled = should_prompt(&cli, &config_enabled);
        let result_disabled = should_prompt(&cli, &config_disabled);

        // If disabled, should never prompt
        assert!(!result_disabled);
        // If enabled, depends on TTY and force
        assert_eq!(result_enabled, is_interactive_mode() && !cli.force);
    }

    #[test]
    fn test_is_interactive_mode() {
        // This test verifies the function works (actual value depends on test environment)
        let result = is_interactive_mode();
        // Should be consistent with should_prompt logic
        assert_eq!(result, is_stdin_tty() && is_stdout_tty());
    }

    #[test]
    fn test_is_piped_output() {
        // This test verifies the function works (actual value depends on test environment)
        let result = is_piped_output();
        // Should be opposite of stdout TTY
        assert_eq!(result, !is_stdout_tty());
    }
}
