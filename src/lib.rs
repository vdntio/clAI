//! clai - AI-Powered Shell Command Translator
//!
//! A shell-native AI command translator that converts natural language to
//! executable commands. Follows Unix philosophy: simple, composable, privacy-respecting.

pub mod ai;
pub mod cli;
pub mod color;
pub mod config;
pub mod context;
pub mod error;
pub mod locale;
pub mod logging;
pub mod output;
pub mod safety;
pub mod signals;

// Re-export AI handler for convenience
pub use ai::handler::generate_command;

// Re-export commonly used types for convenience
pub use cli::{parse_args, Cli};
pub use color::{color_mode_from_config, detect_color_auto, ColorMode};
pub use config::Config;
pub use error::ClaiError;
pub use locale::{get_language_code, get_locale, is_c_locale};
pub use logging::{LogLevel, Logger};
pub use output::{format_config_debug, format_output, print_command};
pub use signals::{
    is_interactive, is_piped, is_stderr_tty, is_stdin_tty, is_stdout_tty, setup_signal_handlers,
    ExitCode,
};
