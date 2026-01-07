use crate::config::file::FileConfig;
use crate::safety::patterns::get_dangerous_regexes;
use regex::Regex;

/// Check if a command matches any dangerous pattern
///
/// Pure function - no side effects, thread-safe.
/// Checks the command against all compiled dangerous regex patterns.
///
/// # Arguments
/// * `command` - The command string to check
/// * `config` - File configuration containing dangerous patterns
///
/// # Returns
/// * `bool` - `true` if command matches any dangerous pattern, `false` otherwise
///
/// # Examples
/// ```
/// use clai::config::file::FileConfig;
/// use clai::safety::detector::is_dangerous_command;
///
/// let config = FileConfig::default();
/// assert!(is_dangerous_command("rm -rf /", &config));
/// assert!(!is_dangerous_command("ls -la", &config));
/// ```
pub fn is_dangerous_command(command: &str, config: &FileConfig) -> bool {
    // Get compiled regexes (lazy-initialized, cached)
    let regexes = match get_dangerous_regexes(config) {
        Ok(regexes) => regexes,
        Err(_) => {
            // If regex compilation failed, fail safe - don't allow command
            // This is a safety measure: if we can't check, we should be cautious
            return true;
        }
    };

    // Check if command matches any pattern
    regexes.iter().any(|regex| regex.is_match(command))
}

/// Check if a command matches any dangerous pattern (with explicit regexes)
///
/// Lower-level function that takes pre-compiled regexes directly.
/// Useful for testing or when you already have compiled regexes.
///
/// # Arguments
/// * `command` - The command string to check
/// * `regexes` - Slice of compiled regex patterns
///
/// # Returns
/// * `bool` - `true` if command matches any pattern, `false` otherwise
///
/// # Examples
/// ```
/// use regex::Regex;
/// use clai::safety::detector::is_dangerous_command_with_regexes;
///
/// let regexes = vec![
///     Regex::new(r"rm\s+-rf\s+/").unwrap(),
/// ];
/// assert!(is_dangerous_command_with_regexes("rm -rf /", &regexes));
/// assert!(!is_dangerous_command_with_regexes("ls -la", &regexes));
/// ```
pub fn is_dangerous_command_with_regexes(command: &str, regexes: &[Regex]) -> bool {
    regexes.iter().any(|regex| regex.is_match(command))
}

/// Get the first matching dangerous pattern (for logging/debugging)
///
/// Returns the index and pattern string of the first matching regex.
/// Useful for verbose logging to show which pattern matched.
///
/// # Arguments
/// * `command` - The command string to check
/// * `config` - File configuration containing dangerous patterns
///
/// # Returns
/// * `Option<(usize, String)>` - Index and pattern string if match found, `None` otherwise
pub fn get_matching_pattern(command: &str, config: &FileConfig) -> Option<(usize, String)> {
    let regexes = get_dangerous_regexes(config).ok()?;

    for (index, regex) in regexes.iter().enumerate() {
        if regex.is_match(command) {
            // Get the original pattern from config (for display)
            let pattern = config
                .safety
                .dangerous_patterns
                .get(index)
                .cloned()
                .unwrap_or_else(|| format!("pattern_{}", index));
            return Some((index, pattern));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::file::FileConfig;
    use crate::safety::patterns::compile_dangerous_regexes;
    use regex::Regex;

    // Helper to check if command is dangerous using freshly compiled regexes
    // (avoids OnceLock cache issues in tests)
    fn is_dangerous_fresh(command: &str, config: &FileConfig) -> bool {
        let regexes = compile_dangerous_regexes(config).unwrap();
        is_dangerous_command_with_regexes(command, &regexes)
    }

    #[test]
    fn test_safe_commands_return_false() {
        let config = FileConfig::default();

        assert!(!is_dangerous_fresh("ls -la", &config));
        assert!(!is_dangerous_fresh("cd /tmp", &config));
        assert!(!is_dangerous_fresh("echo hello", &config));
        assert!(!is_dangerous_fresh("git status", &config));
        assert!(!is_dangerous_fresh("cargo build", &config));
    }

    #[test]
    fn test_dangerous_commands_return_true() {
        let config = FileConfig::default();

        assert!(is_dangerous_fresh("rm -rf /", &config));
        assert!(is_dangerous_fresh("sudo rm -rf /", &config));
        assert!(is_dangerous_fresh("dd if=/dev/zero of=/dev/sda", &config));
    }

    #[test]
    fn test_empty_command_returns_false() {
        let config = FileConfig::default();

        assert!(!is_dangerous_fresh("", &config));
        assert!(!is_dangerous_fresh("   ", &config));
    }

    #[test]
    fn test_is_dangerous_command_with_regexes() {
        let regexes = vec![
            Regex::new(r"rm\s+-rf").unwrap(),
            Regex::new(r"dd\s+if=").unwrap(),
        ];

        assert!(is_dangerous_command_with_regexes("rm -rf /", &regexes));
        assert!(is_dangerous_command_with_regexes(
            "dd if=/dev/zero",
            &regexes
        ));
        assert!(!is_dangerous_command_with_regexes("ls -la", &regexes));
    }

    #[test]
    fn test_get_matching_pattern() {
        // Test get_matching_pattern with default config
        let config = FileConfig::default();

        // Test rm -rf / matches and returns pattern info
        let result = get_matching_pattern("rm -rf /", &config);
        assert!(result.is_some());
        let (index, _pattern) = result.unwrap();
        // Verify we got a valid match (index >= 0)
        assert!(index < config.safety.dangerous_patterns.len());

        // Test dd if= matches
        let result = get_matching_pattern("dd if=/dev/zero of=/dev/sda", &config);
        assert!(result.is_some());

        // Test safe command returns None
        let result = get_matching_pattern("ls -la", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_regex_matching_indices() {
        // Test that regex matching returns correct indices
        let regexes = vec![
            Regex::new(r"rm\s+-rf").unwrap(),
            Regex::new(r"dd\s+if=").unwrap(),
        ];

        // Test rm -rf matches first pattern (index 0)
        let matched = regexes
            .iter()
            .enumerate()
            .find(|(_, r)| r.is_match("rm -rf /"));
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().0, 0);

        // Test dd if= matches second pattern (index 1)
        let matched = regexes
            .iter()
            .enumerate()
            .find(|(_, r)| r.is_match("dd if=/dev/zero"));
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().0, 1);
    }

    #[test]
    fn test_compile_dangerous_regexes_no_match() {
        // Verify that compiled regexes correctly identify safe commands
        let config = FileConfig::default();
        let regexes = compile_dangerous_regexes(&config).unwrap();

        // Safe command should not match any pattern
        let matched = regexes.iter().any(|r| r.is_match("ls -la"));
        assert!(!matched);
    }

    #[test]
    fn test_whitespace_handling() {
        // Use explicit regex that handles leading/trailing whitespace
        let regexes = vec![Regex::new(r"rm\s+-rf\s+/").unwrap()];

        // Standard spacing works
        assert!(is_dangerous_command_with_regexes("rm -rf /", &regexes));

        // Multiple spaces between args works (because \s+ matches multiple)
        assert!(is_dangerous_command_with_regexes("rm   -rf   /", &regexes));

        // Note: Leading whitespace requires trimming or pattern adjustment
        // The pattern "rm\s+-rf\s+/" doesn't match "  rm -rf /" because
        // the pattern expects to start with "rm", not whitespace
        let trimmed = "  rm -rf /  ".trim();
        assert!(is_dangerous_command_with_regexes(trimmed, &regexes));
    }
}
