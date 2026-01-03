use crate::color::{color_mode_from_config, ColorMode};
use crate::config::Config;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error messages only
    Error,
    /// Warning messages (default)
    Warning,
    /// Informational messages
    Info,
    /// Debug messages
    Debug,
    /// Trace messages (most verbose)
    Trace,
}

impl LogLevel {
    /// Get log level from verbosity count
    /// Pure function - no side effects
    pub fn from_verbose_count(count: u8) -> Self {
        match count {
            0 => LogLevel::Warning, // Default
            1 => LogLevel::Info,
            2 => LogLevel::Debug,
            _ => LogLevel::Trace,
        }
    }

    /// Get log level considering quiet flag
    /// Pure function - no side effects
    pub fn from_config(config: &Config) -> Self {
        if config.quiet {
            LogLevel::Error
        } else {
            Self::from_verbose_count(config.verbose)
        }
    }
}

/// Pure function to format log message
/// Takes log level, message, and color mode, returns formatted string
/// No side effects - pure function
pub fn format_log(level: LogLevel, message: &str, color_mode: ColorMode) -> String {
    let use_color = color_mode.should_use_color();
    
    if use_color {
        match level {
            LogLevel::Error => format!("{} {}", colorize("ERROR", "red"), message),
            LogLevel::Warning => format!("{} {}", colorize("WARN", "yellow"), message),
            LogLevel::Info => format!("{} {}", colorize("INFO", "blue"), message),
            LogLevel::Debug => format!("{} {}", colorize("DEBUG", "cyan"), message),
            LogLevel::Trace => format!("{} {}", colorize("TRACE", "magenta"), message),
        }
    } else {
        // No color - just prefix with level
        match level {
            LogLevel::Error => format!("ERROR: {}", message),
            LogLevel::Warning => format!("WARN: {}", message),
            LogLevel::Info => format!("INFO: {}", message),
            LogLevel::Debug => format!("DEBUG: {}", message),
            LogLevel::Trace => format!("TRACE: {}", message),
        }
    }
}

/// Pure function to colorize text (returns ANSI codes)
/// No side effects - pure function
fn colorize(text: &str, color: &str) -> String {
    use owo_colors::OwoColorize;
    
    match color {
        "red" => text.red().to_string(),
        "yellow" => text.yellow().to_string(),
        "blue" => text.blue().to_string(),
        "cyan" => text.cyan().to_string(),
        "magenta" => text.magenta().to_string(),
        _ => text.to_string(),
    }
}

/// Logger struct for managing logging state
#[derive(Debug, Clone)]
pub struct Logger {
    level: LogLevel,
    color_mode: ColorMode,
}

impl Logger {
    /// Create new Logger from Config
    /// Pure function - no side effects
    pub fn from_config(config: &Config) -> Self {
        Self {
            level: LogLevel::from_config(config),
            color_mode: color_mode_from_config(config),
        }
    }

    /// Check if a log level should be displayed
    /// Pure function - no side effects
    pub fn should_log(&self, level: LogLevel) -> bool {
        level <= self.level
    }

    /// Format a log message (pure function)
    /// No side effects - returns formatted string
    pub fn format_message(&self, level: LogLevel, message: &str) -> String {
        format_log(level, message, self.color_mode)
    }

    /// Log to stderr (side effect - but isolated)
    /// This is the only function with side effects in this module
    pub fn log(&self, level: LogLevel, message: &str) {
        if self.should_log(level) {
            eprintln!("{}", self.format_message(level, message));
        }
    }

    /// Convenience methods
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_verbose_count() {
        assert_eq!(LogLevel::from_verbose_count(0), LogLevel::Warning);
        assert_eq!(LogLevel::from_verbose_count(1), LogLevel::Info);
        assert_eq!(LogLevel::from_verbose_count(2), LogLevel::Debug);
        assert_eq!(LogLevel::from_verbose_count(3), LogLevel::Trace);
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warning);
        assert!(LogLevel::Warning < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }

    #[test]
    fn test_format_log_pure() {
        let message = "test message";
        let formatted1 = format_log(LogLevel::Error, message, ColorMode::Never);
        let formatted2 = format_log(LogLevel::Error, message, ColorMode::Never);
        
        // Pure function - same input, same output
        assert_eq!(formatted1, formatted2);
        assert!(formatted1.contains("ERROR"));
        assert!(formatted1.contains(message));
    }

    #[test]
    fn test_logger_should_log() {
        let logger = Logger {
            level: LogLevel::Info,
            color_mode: ColorMode::Never,
        };

        assert!(logger.should_log(LogLevel::Error));
        assert!(logger.should_log(LogLevel::Warning));
        assert!(logger.should_log(LogLevel::Info));
        assert!(!logger.should_log(LogLevel::Debug));
        assert!(!logger.should_log(LogLevel::Trace));
    }
}

