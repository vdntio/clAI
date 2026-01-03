pub mod confirmation;
pub mod detector;
pub mod interactive;
pub mod patterns;
pub mod prompt;

pub use confirmation::{format_decision, handle_dangerous_confirmation, ConfirmationError, Decision};
pub use detector::{get_matching_pattern, is_dangerous_command, is_dangerous_command_with_regexes};
pub use interactive::{execute_command, prompt_command_action, CommandAction, InteractiveError};
pub use patterns::{compile_dangerous_regexes, get_dangerous_regexes};
pub use prompt::{is_interactive_mode, is_piped_output, should_prompt};
