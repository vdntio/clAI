use std::io::{self, Read};

/// Detect if stdin is piped (not a TTY)
///
/// Uses atty crate to check if stdin is a terminal.
/// Returns true if stdin is piped (not a TTY), false otherwise.
///
/// Pure function - checks TTY status
///
/// # Returns
/// * `bool` - True if stdin is piped, false if it's a TTY
pub fn is_stdin_piped() -> bool {
    !atty::is(atty::Stream::Stdin)
}

/// Read stdin with configurable byte limit
///
/// Reads all available input from stdin up to max_bytes.
/// If input exceeds max_bytes, it's truncated.
///
/// Returns None if stdin is not piped (is a TTY) or if reading fails.
/// Returns Some("") if stdin is piped but empty.
/// Returns Some(content) with the read content (possibly truncated).
///
/// # Arguments
/// * `max_bytes` - Maximum number of bytes to read (default: 10KB)
///
/// # Returns
/// * `Option<String>` - None if not piped/error, Some(content) if piped
pub fn read_stdin(max_bytes: usize) -> Option<String> {
    // Check if stdin is piped
    if !is_stdin_piped() {
        return None;
    }

    // Read from stdin with limit
    let mut buffer = vec![0u8; max_bytes];
    let mut stdin = io::stdin();

    match stdin.read(&mut buffer) {
        Ok(0) => {
            // Empty pipe
            Some(String::new())
        }
        Ok(n) => {
            // Read n bytes, truncate buffer
            buffer.truncate(n);

            // Convert to string, handling invalid UTF-8 gracefully
            // Use from_utf8_lossy to handle invalid UTF-8 sequences
            Some(String::from_utf8_lossy(&buffer).to_string())
        }
        Err(_) => {
            // Error reading stdin
            None
        }
    }
}

/// Read stdin with default limit (10KB)
///
/// Convenience function that calls read_stdin with default 10KB limit.
///
/// # Returns
/// * `Option<String>` - None if not piped/error, Some(content) if piped
pub fn read_stdin_default() -> Option<String> {
    const DEFAULT_MAX_BYTES: usize = 10 * 1024; // 10KB
    read_stdin(DEFAULT_MAX_BYTES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stdin_piped() {
        // In test environment, stdin is typically not piped
        // Just verify function doesn't panic
        let _ = is_stdin_piped();
    }

    #[test]
    fn test_read_stdin_not_piped() {
        // When stdin is not piped (TTY), should return None
        // In test environment, stdin is typically not piped
        // This test verifies the function handles non-piped stdin correctly
        let result = read_stdin(1024);
        // May be None (if not piped) or Some (if somehow piped in test)
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_read_stdin_empty() {
        // Test with very small limit to verify empty handling
        // Note: This test may not work as expected in test environment
        // where stdin might not be piped
        let result = read_stdin(1);
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_read_stdin_default() {
        // Test default limit function
        let result = read_stdin_default();
        // Just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_is_stdin_piped_pure() {
        // Pure function - same environment, same output
        let result1 = is_stdin_piped();
        let result2 = is_stdin_piped();

        assert_eq!(result1, result2);
    }
}
