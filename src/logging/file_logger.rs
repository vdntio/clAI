//! File-based debug logger for clai
//!
//! Provides opt-in file logging for debugging and troubleshooting.
//! Writes structured JSON Lines format for easy parsing.

use super::LogLevel;
use anyhow::Result;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum log file size before truncation (10 MB)
const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;

/// File-based debug logger
///
/// Writes structured JSON log entries to a file.
/// Thread-safe via interior mutability with Mutex.
pub struct FileLogger {
    writer: Mutex<BufWriter<File>>,
    path: PathBuf,
}

/// Log entry structure for JSON serialization
#[derive(Debug, Serialize)]
struct LogEntry<'a> {
    ts: String,
    level: &'a str,
    event: &'a str,
    #[serde(flatten)]
    data: serde_json::Value,
}

impl FileLogger {
    /// Create a new file logger
    ///
    /// Creates parent directories if needed.
    /// Truncates file if it exceeds MAX_LOG_SIZE.
    pub fn new(path: PathBuf) -> Result<Self> {
        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Check file size and truncate if needed
        if path.exists() {
            let metadata = std::fs::metadata(&path)?;
            if metadata.len() > MAX_LOG_SIZE {
                // Truncate by removing and recreating
                std::fs::remove_file(&path)?;
            }
        }

        // Open file in append mode
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        let writer = BufWriter::new(file);

        Ok(Self {
            writer: Mutex::new(writer),
            path,
        })
    }

    /// Get the log file path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Log an event with structured data
    pub fn log(&self, level: LogLevel, event: &str, data: serde_json::Value) {
        let entry = LogEntry {
            ts: iso8601_timestamp(),
            level: level_str(level),
            event,
            data,
        };

        if let Ok(json) = serde_json::to_string(&entry) {
            if let Ok(mut guard) = self.writer.lock() {
                let _ = writeln!(guard, "{}", json);
                let _ = guard.flush();
            }
        }
    }

    /// Log AI request with full message content
    pub fn log_request(
        &self,
        model: Option<&str>,
        messages: &[crate::ai::types::ChatMessage],
        temperature: Option<f64>,
        max_tokens: Option<u32>,
    ) {
        let messages_data: Vec<serde_json::Value> = messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": format!("{:?}", m.role).to_lowercase(),
                    "content": m.content
                })
            })
            .collect();

        self.log(
            LogLevel::Debug,
            "ai_request",
            serde_json::json!({
                "model": model,
                "messages": messages_data,
                "temperature": temperature,
                "max_tokens": max_tokens
            }),
        );
    }

    /// Log AI response
    pub fn log_response(
        &self,
        model: Option<&str>,
        status: u16,
        content: &str,
        usage: Option<&crate::ai::types::Usage>,
    ) {
        self.log(
            LogLevel::Debug,
            "ai_response",
            serde_json::json!({
                "model": model,
                "status": status,
                "content": content,
                "usage": usage.map(|u| serde_json::json!({
                    "prompt_tokens": u.prompt_tokens,
                    "completion_tokens": u.completion_tokens,
                    "total_tokens": u.total_tokens
                }))
            }),
        );
    }

    /// Log error with context
    pub fn log_error(&self, event: &str, error: &str, context: Option<serde_json::Value>) {
        let mut data = serde_json::json!({
            "error": error
        });

        if let Some(ctx) = context {
            if let serde_json::Value::Object(ref mut map) = data {
                if let serde_json::Value::Object(ctx_map) = ctx {
                    map.extend(ctx_map);
                }
            }
        }

        self.log(LogLevel::Error, event, data);
    }
}

/// Generate ISO 8601 timestamp without external dependencies
fn iso8601_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let total_secs = duration.as_secs();
    let millis = duration.subsec_millis();

    // Calculate date/time components (UTC)
    let days_since_epoch = total_secs / 86400;
    let secs_today = total_secs % 86400;

    let hours = secs_today / 3600;
    let minutes = (secs_today % 3600) / 60;
    let seconds = secs_today % 60;

    // Calculate year, month, day from days since 1970-01-01
    let (year, month, day) = days_to_ymd(days_since_epoch);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hours, minutes, seconds, millis
    )
}

/// Convert days since Unix epoch to year, month, day
fn days_to_ymd(days: u64) -> (i32, u32, u32) {
    // Algorithm based on Howard Hinnant's date algorithms
    let z = days as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    (y as i32, m, d)
}

/// Convert LogLevel to string for JSON output
fn level_str(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "ERROR",
        LogLevel::Warning => "WARN",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
        LogLevel::Trace => "TRACE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_logger_creation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.log");

        let logger = FileLogger::new(path.clone()).unwrap();
        logger.log(
            LogLevel::Info,
            "test_event",
            serde_json::json!({"key": "value"}),
        );

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("test_event"));
        assert!(contents.contains("INFO"));
        assert!(contents.contains("key"));
        assert!(contents.contains("value"));
    }

    #[test]
    fn test_iso8601_timestamp_format() {
        let ts = iso8601_timestamp();
        // Should match pattern: YYYY-MM-DDTHH:MM:SS.mmmZ
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
        assert_eq!(ts.len(), 24); // 2024-01-05T10:30:00.123Z
    }

    #[test]
    fn test_log_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("error.log");

        let logger = FileLogger::new(path.clone()).unwrap();
        logger.log_error(
            "test_error",
            "Something went wrong",
            Some(serde_json::json!({"status": 500})),
        );

        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("ERROR"));
        assert!(contents.contains("test_error"));
        assert!(contents.contains("Something went wrong"));
        assert!(contents.contains("500"));
    }

    #[test]
    fn test_creates_parent_directories() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join("test.log");

        let logger = FileLogger::new(path.clone()).unwrap();
        logger.log(LogLevel::Debug, "test", serde_json::json!({}));

        assert!(path.exists());
    }
}
