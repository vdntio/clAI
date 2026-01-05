use thiserror::Error;

/// Comprehensive error enum with specific exit codes per FR-7
/// 
/// Maps to exit codes:
/// - General = 1 (unexpected errors)
/// - Usage = 2 (invalid CLI arguments)
/// - Config = 3 (configuration errors)
/// - API = 4 (AI provider/network errors)
/// - Safety = 5 (dangerous command rejected)
#[derive(Debug, Error)]
pub enum ClaiError {
    /// General error (exit code 1)
    /// Catch-all for unexpected errors
    #[error("Error: {0}")]
    General(#[from] anyhow::Error),

    /// Usage error (exit code 2)
    /// Invalid CLI arguments or missing required parameters
    #[error("Usage error: {0}")]
    Usage(String),

    /// Configuration error (exit code 3)
    /// Missing keys, invalid TOML, file permission issues
    #[error("Configuration error: {source}")]
    Config {
        /// Source error from config loading
        #[source]
        source: anyhow::Error,
    },

    /// API error (exit code 4)
    /// Network errors, authentication failures, rate limits
    #[error("API error: {source}")]
    API {
        /// Source error from API provider
        #[source]
        source: anyhow::Error,
        /// Optional HTTP status code for API errors
        status_code: Option<u16>,
    },

    /// Safety error (exit code 5)
    /// Dangerous command rejected by user or safety checks
    #[error("Safety error: {0}")]
    Safety(String),
}

impl ClaiError {
    /// Get the exit code for this error
    /// 
    /// Returns the appropriate exit code per FR-7:
    /// - General = 1
    /// - Usage = 2
    /// - Config = 3
    /// - API = 4
    /// - Safety = 5
    pub fn exit_code(&self) -> u8 {
        match self {
            ClaiError::General(_) => 1,
            ClaiError::Usage(_) => 2,
            ClaiError::Config { .. } => 3,
            ClaiError::API { .. } => 4,
            ClaiError::Safety(_) => 5,
        }
    }

    /// Print error to stderr with optional backtrace
    /// 
    /// Respects verbosity level for backtrace display.
    /// Always prints human-readable error message to stderr.
    /// 
    /// # Arguments
    /// * `verbose` - Verbosity level (0=normal, 1+=show backtrace)
    pub fn print_stderr(&self, verbose: u8) {
        
        
        // Always print the error message
        eprintln!("{}", self);
        
        // Show backtrace if verbose >= 1
        if verbose >= 1 {
            if let Some(backtrace) = self.backtrace() {
                eprintln!("\nBacktrace:\n{}", backtrace);
            }
        }
    }

    /// Get backtrace if available
    /// 
    /// Extracts backtrace from anyhow error chain
    fn backtrace(&self) -> Option<String> {
        match self {
            ClaiError::General(err) | ClaiError::Config { source: err } | ClaiError::API { source: err, .. } => {
                // Try to get backtrace from anyhow error
                let mut backtrace_str = String::new();
                let mut current: &dyn std::error::Error = err.as_ref();
                
                // Build error chain
                backtrace_str.push_str(&format!("Error: {}\n", current));
                while let Some(source) = current.source() {
                    backtrace_str.push_str(&format!("Caused by: {}\n", source));
                    current = source;
                }
                
                if backtrace_str.len() > 0 {
                    Some(backtrace_str)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Convert clap::Error to ClaiError::Usage
impl From<clap::Error> for ClaiError {
    fn from(err: clap::Error) -> Self {
        ClaiError::Usage(err.to_string())
    }
}

/// Convert ConfigLoadError to ClaiError::Config
impl From<crate::config::loader::ConfigLoadError> for ClaiError {
    fn from(err: crate::config::loader::ConfigLoadError) -> Self {
        ClaiError::Config {
            source: anyhow::Error::from(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes() {
        assert_eq!(ClaiError::General(anyhow::anyhow!("test")).exit_code(), 1);
        assert_eq!(ClaiError::Usage("test".to_string()).exit_code(), 2);
        assert_eq!(
            ClaiError::Config {
                source: anyhow::anyhow!("test")
            }
            .exit_code(),
            3
        );
        assert_eq!(
            ClaiError::API {
                source: anyhow::anyhow!("test"),
                status_code: None
            }
            .exit_code(),
            4
        );
        assert_eq!(ClaiError::Safety("test".to_string()).exit_code(), 5);
    }

    #[test]
    fn test_error_display() {
        let err = ClaiError::Usage("Missing required argument".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Usage error"));
        assert!(display.contains("Missing required argument"));
    }

    #[test]
    fn test_clap_error_conversion() {
        use clap::Parser;
        // Try to parse with missing required argument
        let cli = crate::cli::Cli::try_parse_from(["clai"]);
        if let Err(clap_err) = cli {
            let clai_err = ClaiError::from(clap_err);
            assert_eq!(clai_err.exit_code(), 2);
        } else {
            panic!("Expected clap error for missing argument");
        }
    }

    #[test]
    fn test_config_error_conversion() {
        use crate::config::loader::ConfigLoadError;
        let config_err = ConfigLoadError::NotFound("/nonexistent".to_string());
        let clai_err: ClaiError = config_err.into();
        assert_eq!(clai_err.exit_code(), 3);
    }
}

