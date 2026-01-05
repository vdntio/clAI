pub mod directory;
pub mod gatherer;
pub mod history;
pub mod stdin;
pub mod system;

pub use directory::scan_directory;
pub use gatherer::{gather_context, get_context_json, ContextData};
pub use history::{detect_shell, get_history_path, get_shell_history, read_history_tail};
pub use stdin::{is_stdin_piped, read_stdin, read_stdin_default};
pub use system::{format_system_info, get_formatted_system_info, get_system_info, SystemInfo};
