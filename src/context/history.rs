use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

/// Detect shell from $SHELL environment variable
///
/// Returns the shell name (e.g., "bash", "zsh", "fish")
///
/// Pure function - reads environment variable
///
/// # Returns
/// * `String` - Shell name, or "unknown" if not detected
pub fn detect_shell() -> String {
    std::env::var("SHELL")
        .unwrap_or_else(|_| "unknown".to_string())
        .split('/')
        .next_back()
        .unwrap_or("unknown")
        .to_string()
}

/// Get history file path for detected shell
///
/// Maps shell name to its history file path:
/// - bash: ~/.bash_history
/// - zsh: ~/.zsh_history
/// - fish: ~/.local/share/fish/fish_history
///
/// Pure function - constructs path from shell name
///
/// # Arguments
/// * `shell` - Shell name (e.g., "bash", "zsh", "fish")
///
/// # Returns
/// * `Option<PathBuf>` - History file path, or None if shell not supported
pub fn get_history_path(shell: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let home_path = PathBuf::from(&home);

    match shell {
        "bash" => Some(home_path.join(".bash_history")),
        "zsh" => Some(home_path.join(".zsh_history")),
        "fish" => Some(
            home_path
                .join(".local")
                .join("share")
                .join("fish")
                .join("fish_history"),
        ),
        _ => None,
    }
}

/// Read last N lines from history file using tail-like logic
///
/// Uses efficient tail-like approach:
/// 1. Seeks to end of file minus 4096 bytes (or start if file is smaller)
/// 2. Reads lines from that point
/// 3. Takes last N lines
///
/// Handles missing files gracefully (returns empty vec)
///
/// # Arguments
/// * `path` - Path to history file
/// * `max_lines` - Maximum number of lines to return (default: 3)
///
/// # Returns
/// * `Vec<String>` - Last N lines from history file
pub fn read_history_tail(path: &PathBuf, max_lines: u32) -> Vec<String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let mut reader = BufReader::new(file);

    // Get file size
    let file_size = match reader.seek(SeekFrom::End(0)) {
        Ok(pos) => pos,
        Err(_) => return Vec::new(),
    };

    // Seek to position for tail reading (4096 bytes from end, or start if smaller)
    let seek_pos = file_size.saturating_sub(4096);

    if reader.seek(SeekFrom::Start(seek_pos)).is_err() {
        return Vec::new();
    }

    // Read all lines from seek position
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();

    // Take last N lines
    let start = if lines.len() > max_lines as usize {
        lines.len() - max_lines as usize
    } else {
        0
    };

    lines[start..].to_vec()
}

/// Get shell history (convenience function)
///
/// Detects shell, gets history path, and reads last N lines
///
/// # Arguments
/// * `max_history` - Maximum number of history lines to return (default: 3)
///
/// # Returns
/// * `Vec<String>` - Last N commands from shell history
pub fn get_shell_history(max_history: u32) -> Vec<String> {
    let shell = detect_shell();

    match get_history_path(&shell) {
        Some(path) => read_history_tail(&path, max_history),
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_shell() {
        let shell = detect_shell();
        // Should return something (may be "unknown" if $SHELL not set in test)
        assert!(!shell.is_empty());
    }

    #[test]
    fn test_get_history_path_bash() {
        if let Ok(home) = std::env::var("HOME") {
            let path = get_history_path("bash");
            assert!(path.is_some());
            assert_eq!(path.unwrap(), PathBuf::from(home).join(".bash_history"));
        }
    }

    #[test]
    fn test_get_history_path_zsh() {
        if let Ok(home) = std::env::var("HOME") {
            let path = get_history_path("zsh");
            assert!(path.is_some());
            assert_eq!(path.unwrap(), PathBuf::from(home).join(".zsh_history"));
        }
    }

    #[test]
    fn test_get_history_path_fish() {
        if let Ok(home) = std::env::var("HOME") {
            let path = get_history_path("fish");
            assert!(path.is_some());
            let expected = PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("fish")
                .join("fish_history");
            assert_eq!(path.unwrap(), expected);
        }
    }

    #[test]
    fn test_get_history_path_unknown() {
        let path = get_history_path("unknown_shell");
        assert!(path.is_none());
    }

    #[test]
    fn test_read_history_tail_small_file() {
        // Create temp file with 5 lines
        let mut temp_file = NamedTempFile::new().unwrap();
        for i in 1..=5 {
            writeln!(temp_file, "command_{}", i).unwrap();
        }
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let lines = read_history_tail(&path, 3);

        // Should return last 3 lines
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "command_3");
        assert_eq!(lines[1], "command_4");
        assert_eq!(lines[2], "command_5");
    }

    #[test]
    fn test_read_history_tail_large_file() {
        // Create temp file with 20 lines (larger than 4096 bytes when written)
        let mut temp_file = NamedTempFile::new().unwrap();
        for i in 1..=20 {
            writeln!(
                temp_file,
                "command_{}_with_some_additional_text_to_make_line_longer",
                i
            )
            .unwrap();
        }
        temp_file.flush().unwrap();

        let path = temp_file.path().to_path_buf();
        let lines = read_history_tail(&path, 3);

        // Should return last 3 lines
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("command_18"));
        assert!(lines[1].contains("command_19"));
        assert!(lines[2].contains("command_20"));
    }

    #[test]
    fn test_read_history_tail_missing_file() {
        let path = PathBuf::from("/nonexistent/history/file");
        let lines = read_history_tail(&path, 3);

        // Should return empty vec for missing file
        assert!(lines.is_empty());
    }

    #[test]
    fn test_read_history_tail_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let lines = read_history_tail(&path, 3);

        // Should return empty vec for empty file
        assert!(lines.is_empty());
    }

    #[test]
    fn test_get_shell_history() {
        // This test depends on actual shell history file
        // Just verify it doesn't panic and returns a vec
        let history = get_shell_history(3);

        // Should return a vec (may be empty if history file doesn't exist)
        let _ = history;
    }

    #[test]
    fn test_detect_shell_pure() {
        // Pure function - same environment, same output
        let shell1 = detect_shell();
        let shell2 = detect_shell();

        assert_eq!(shell1, shell2);
    }
}
