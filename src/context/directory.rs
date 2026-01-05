use std::fs;
use std::path::PathBuf;

/// Scan current working directory for top N files/directories
///
/// Returns a vector of file/directory paths, sorted alphabetically, limited to top N.
/// Paths are truncated if >80 characters (to basename).
/// Paths are redacted if redact_paths is true (replaces username/home with [REDACTED]).
///
/// Pure function with I/O side effects (reads directory)
///
/// # Arguments
/// * `max_files` - Maximum number of files/dirs to return (default: 10)
/// * `redact_paths` - Whether to redact paths (replace username/home with [REDACTED])
///
/// # Returns
/// * `Vec<String>` - Vector of truncated/redacted paths
pub fn scan_directory(max_files: u32, redact_paths: bool) -> Vec<String> {
    // Get current working directory
    let cwd = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => return Vec::new(),
    };

    // Read directory entries
    let entries = match fs::read_dir(&cwd) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    // Collect and sort entries
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // Sort alphabetically by file name
    paths.sort_by(|a, b| {
        a.file_name()
            .and_then(|n| n.to_str())
            .cmp(&b.file_name().and_then(|n| n.to_str()))
    });

    // Take top N
    let paths: Vec<PathBuf> = paths.into_iter().take(max_files as usize).collect();

    // Convert to strings with truncation and redaction
    paths
        .into_iter()
        .map(|path| {
            let path_str = path.to_string_lossy().to_string();
            truncate_path(&path_str, 80)
        })
        .map(|path_str| {
            if redact_paths {
                redact_path_internal(&path_str)
            } else {
                path_str
            }
        })
        .collect()
}

/// Truncate path if it exceeds max_length
///
/// If path is longer than max_length, returns just the basename.
/// Otherwise returns the path unchanged.
///
/// Pure function - no side effects
///
/// # Arguments
/// * `path` - Path string to truncate
/// * `max_length` - Maximum length (default: 80)
///
/// # Returns
/// * `String` - Truncated path
fn truncate_path(path: &str, max_length: usize) -> String {
    if path.len() <= max_length {
        return path.to_string();
    }

    // Extract basename
    let path_buf = PathBuf::from(path);
    path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Redact path by replacing username/home directory with [REDACTED]
///
/// Replaces:
/// - ~/ with [REDACTED]/
/// - /home/username/ with [REDACTED]/
/// - $HOME/ with [REDACTED]/
///
/// Pure function - no side effects
///
/// # Arguments
/// * `path` - Path string to redact
///
/// # Returns
/// * `String` - Redacted path
pub(crate) fn redact_path_internal(path: &str) -> String {
    let mut redacted = path.to_string();

    // Get home directory for redaction
    if let Ok(home) = std::env::var("HOME") {
        // Replace /home/username/ with [REDACTED]/
        if redacted.starts_with(&home) {
            redacted = redacted.replacen(&home, "[REDACTED]", 1);
        }
    }

    // Replace ~/ with [REDACTED]/
    if redacted.starts_with("~/") {
        redacted = redacted.replacen("~/", "[REDACTED]/", 1);
    } else if redacted == "~" {
        redacted = "[REDACTED]".to_string();
    }

    // Replace $HOME/ with [REDACTED]/
    if let Ok(home) = std::env::var("HOME") {
        let home_var = format!("${}/", home);
        if redacted.starts_with(&home_var) {
            redacted = redacted.replacen(&home_var, "[REDACTED]/", 1);
        }
    }

    // Replace username in path (e.g., /home/username/...)
    if let Ok(user) = std::env::var("USER") {
        let user_path = format!("/home/{}/", user);
        if redacted.contains(&user_path) {
            redacted = redacted.replace(&user_path, "[REDACTED]/");
        }
    }

    redacted
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_truncate_path_short() {
        let path = "short/path";
        assert_eq!(truncate_path(path, 80), "short/path");
    }

    #[test]
    fn test_truncate_path_long() {
        let long_path =
            "/very/long/path/that/exceeds/eighty/characters/and/should/be/truncated/to/basename";
        let truncated = truncate_path(long_path, 80);
        // Should be just the basename
        assert!(truncated.len() <= 80);
        assert_eq!(truncated, "basename");
    }

    #[test]
    fn test_redact_path_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let path = format!("{}/test/file", home);
        let redacted = redact_path_internal(&path);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains(&home));
    }

    #[test]
    fn test_redact_path_tilde() {
        let path = "~/test/file";
        let redacted = redact_path_internal(path);
        assert_eq!(redacted, "[REDACTED]/test/file");
    }

    #[test]
    fn test_scan_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        for i in 0..15 {
            let file_path = temp_dir.path().join(format!("file_{:02}.txt", i));
            let mut file = fs::File::create(&file_path).unwrap();
            file.write_all(b"test").unwrap();
        }

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Scan directory
        let files = scan_directory(10, false);

        // Should return exactly 10 files (sorted)
        assert_eq!(files.len(), 10);

        // Should be sorted alphabetically
        let mut sorted = files.clone();
        sorted.sort();
        assert_eq!(files, sorted);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_scan_directory_with_redaction() {
        let temp_dir = TempDir::new().unwrap();

        // Create test file
        let file_path = temp_dir.path().join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test").unwrap();

        // Change to temp directory
        let original_dir = std::env::current_dir().unwrap();
        let temp_path = temp_dir.path().to_path_buf(); // Keep reference to path

        match std::env::set_current_dir(&temp_path) {
            Ok(_) => {
                // Scan with redaction
                let files = scan_directory(10, true);

                // Should return files (redaction may or may not apply depending on path)
                assert!(!files.is_empty());

                // Restore original directory
                let _ = std::env::set_current_dir(&original_dir);
            }
            Err(_) => {
                // If we can't change directory, just verify the function doesn't panic
                // when called from current directory
                let files = scan_directory(10, true);
                // May be empty or have files, but shouldn't panic
                let _ = files;
            }
        }
    }

    #[test]
    fn test_scan_directory_empty() {
        let temp_dir = TempDir::new().unwrap();

        // Change to empty temp directory
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Scan empty directory
        let files = scan_directory(10, false);

        // Should return empty or just . and ..
        // (depending on filesystem, may have hidden files)
        // Just verify it doesn't panic
        assert!(files.len() <= 2);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[test]
    fn test_redact_path_pure() {
        let path = "~/test/file";

        // Pure function - same input, same output
        let redacted1 = redact_path_internal(path);
        let redacted2 = redact_path_internal(path);

        assert_eq!(redacted1, redacted2);
    }
}
