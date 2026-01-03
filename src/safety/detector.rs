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
    use regex::Regex;

    #[test]
    fn test_safe_commands_return_false() {
        let config = FileConfig::default();
        
        assert!(!is_dangerous_command("ls -la", &config));
        assert!(!is_dangerous_command("cd /tmp", &config));
        assert!(!is_dangerous_command("echo hello", &config));
        assert!(!is_dangerous_command("git status", &config));
        assert!(!is_dangerous_command("cargo build", &config));
    }

    #[test]
    fn test_dangerous_commands_return_true() {
        let config = FileConfig::default();
        
        assert!(is_dangerous_command("rm -rf /", &config));
        assert!(is_dangerous_command("sudo rm -rf /", &config));
        assert!(is_dangerous_command("dd if=/dev/zero of=/dev/sda", &config));
    }

    #[test]
    fn test_empty_command_returns_false() {
        let config = FileConfig::default();
        
        assert!(!is_dangerous_command("", &config));
        assert!(!is_dangerous_command("   ", &config));
    }

    #[test]
    fn test_is_dangerous_command_with_regexes() {
        let regexes = vec![
            Regex::new(r"rm\s+-rf").unwrap(),
            Regex::new(r"dd\s+if=").unwrap(),
        ];
        
        assert!(is_dangerous_command_with_regexes("rm -rf /", &regexes));
        assert!(is_dangerous_command_with_regexes("dd if=/dev/zero", &regexes));
        assert!(!is_dangerous_command_with_regexes("ls -la", &regexes));
    }

    #[test]
    fn test_get_matching_pattern() {
        let mut config = FileConfig::default();
        config.safety.dangerous_patterns = vec![
            r"rm\s+-rf".to_string(),
            r"dd\s+if=".to_string(),
        ];
        
        let result = get_matching_pattern("rm -rf /", &config);
        assert!(result.is_some());
        let (index, pattern) = result.unwrap();
        assert_eq!(index, 0);
        assert_eq!(pattern, r"rm\s+-rf");
        
        let result = get_matching_pattern("dd if=/dev/zero", &config);
        assert!(result.is_some());
        let (index, _) = result.unwrap();
        assert_eq!(index, 1);
    }

    #[test]
    fn test_get_matching_pattern_no_match() {
        let config = FileConfig::default();
        
        let result = get_matching_pattern("ls -la", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_whitespace_handling() {
        let config = FileConfig::default();
        
        // Commands with extra whitespace should still be detected
        assert!(is_dangerous_command("  rm -rf /  ", &config));
        assert!(is_dangerous_command("rm   -rf   /", &config));
    }
}
