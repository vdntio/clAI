use crate::config::file::FileConfig;
use anyhow::{Context, Result};
use regex::Regex;
use std::sync::OnceLock;

/// Cached compiled dangerous pattern regexes
///
/// Thread-safe lazy initialization using OnceLock.
/// Compiled once on first access, reused for all subsequent checks.
static DANGEROUS_REGEXES: OnceLock<Result<Vec<Regex>, String>> = OnceLock::new();

/// Default dangerous command patterns
///
/// These are safe defaults that catch common destructive commands.
/// Users can override via config file.
fn default_dangerous_patterns() -> Vec<String> {
    vec![
        r"rm\s+-rf\s+/".to_string(),             // rm -rf /
        r"rm\s+-rf\s+/\s*$".to_string(),         // rm -rf / (end of line)
        r"dd\s+if=/dev/zero".to_string(),        // dd if=/dev/zero
        r"mkfs\.\w+\s+/dev/".to_string(),        // mkfs.* /dev/
        r"sudo\s+rm\s+-rf\s+/".to_string(),      // sudo rm -rf /
        r">\s*/dev/".to_string(),                // > /dev/
        r"format\s+[c-z]:".to_string(),          // format C: (Windows)
        r"del\s+/f\s+/s\s+[c-z]:\\".to_string(), // del /f /s C:\ (Windows)
    ]
}

/// Compile dangerous pattern regexes from config
///
/// Pure function that compiles regex patterns from config.
/// Uses lazy static caching - compiled once, reused forever.
///
/// # Arguments
/// * `config` - File configuration containing dangerous patterns
///
/// # Returns
/// * `Result<Vec<Regex>>` - Compiled regex patterns or error
///
/// # Errors
/// * Returns error if any pattern fails to compile as valid regex
pub fn compile_dangerous_regexes(config: &FileConfig) -> Result<Vec<Regex>> {
    // Get patterns from config or use defaults
    let patterns = if config.safety.dangerous_patterns.is_empty() {
        default_dangerous_patterns()
    } else {
        config.safety.dangerous_patterns.clone()
    };

    // Compile each pattern
    let mut regexes = Vec::with_capacity(patterns.len());

    for (index, pattern) in patterns.iter().enumerate() {
        match Regex::new(pattern) {
            Ok(regex) => regexes.push(regex),
            Err(e) => {
                // Log error to stderr but continue with other patterns
                eprintln!(
                    "Warning: Invalid dangerous pattern at index {}: '{}' - {}",
                    index, pattern, e
                );
                // Return error for invalid regex (fail fast for safety)
                return Err(anyhow::anyhow!(
                    "Failed to compile dangerous pattern '{}' at index {}: {}",
                    pattern,
                    index,
                    e
                ))
                .context("Invalid regex pattern in dangerous_patterns config");
            }
        }
    }

    Ok(regexes)
}

/// Get or compile dangerous regexes (lazy initialization)
///
/// Thread-safe function that compiles regexes once on first access.
/// Subsequent calls return the cached compiled regexes.
///
/// # Arguments
/// * `config` - File configuration
///
/// # Returns
/// * `Result<&[Regex]>` - Reference to compiled regexes
pub fn get_dangerous_regexes(config: &FileConfig) -> Result<&'static [Regex]> {
    DANGEROUS_REGEXES
        .get_or_init(|| match compile_dangerous_regexes(config) {
            Ok(regexes) => Ok(regexes),
            Err(e) => Err(e.to_string()),
        })
        .as_ref()
        .map_err(|e| anyhow::anyhow!("Failed to compile dangerous patterns: {}", e))
        .map(|regexes| regexes.as_slice())
}

/// Reset dangerous regex cache (for testing only)
///
/// # Safety
/// This function is only intended for testing.
/// It clears the cache, allowing tests to use different configs.
#[cfg(test)]
pub fn reset_regex_cache() {
    // OnceLock doesn't have a reset method, so we can't actually reset it
    // This is a no-op, but documents the intent for testing
    // In practice, tests should use different configs or test compile_dangerous_regexes directly
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::file::FileConfig;

    #[test]
    fn test_default_patterns_compile() {
        let config = FileConfig::default();
        let regexes = compile_dangerous_regexes(&config).unwrap();
        assert!(!regexes.is_empty());
    }

    #[test]
    fn test_default_patterns_match_rm_rf() {
        let config = FileConfig::default();
        let regexes = compile_dangerous_regexes(&config).unwrap();

        // Test that default patterns match dangerous commands
        assert!(regexes.iter().any(|r| r.is_match("rm -rf /")));
        assert!(regexes.iter().any(|r| r.is_match("sudo rm -rf /")));
        assert!(regexes
            .iter()
            .any(|r| r.is_match("dd if=/dev/zero of=/dev/sda")));
    }

    #[test]
    fn test_custom_patterns() {
        let mut config = FileConfig::default();
        config.safety.dangerous_patterns = vec![
            r"dangerous\s+command".to_string(),
            r"test\s+pattern".to_string(),
        ];

        let regexes = compile_dangerous_regexes(&config).unwrap();
        assert_eq!(regexes.len(), 2);
        assert!(regexes.iter().any(|r| r.is_match("dangerous command")));
        assert!(regexes.iter().any(|r| r.is_match("test pattern")));
    }

    #[test]
    fn test_invalid_regex_returns_error() {
        let mut config = FileConfig::default();
        config.safety.dangerous_patterns = vec![
            r"valid\s+pattern".to_string(),
            r"[invalid regex".to_string(), // Unclosed bracket
        ];

        let result = compile_dangerous_regexes(&config);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Error message should mention the pattern or compilation failure
        assert!(
            error_msg.contains("Failed to compile") || error_msg.contains("Invalid regex pattern")
        );
    }

    #[test]
    fn test_empty_patterns_uses_defaults() {
        let mut config = FileConfig::default();
        config.safety.dangerous_patterns = vec![];

        // Empty vec should use defaults
        let regexes = compile_dangerous_regexes(&config).unwrap();
        assert!(!regexes.is_empty()); // Should have default patterns
    }

    #[test]
    fn test_safe_commands_dont_match() {
        let config = FileConfig::default();
        let regexes = compile_dangerous_regexes(&config).unwrap();

        // Safe commands should not match
        assert!(!regexes.iter().any(|r| r.is_match("ls -la")));
        assert!(!regexes.iter().any(|r| r.is_match("cd /tmp")));
        assert!(!regexes.iter().any(|r| r.is_match("echo hello")));
        assert!(!regexes.iter().any(|r| r.is_match("git status")));
    }
}
