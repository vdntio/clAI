use clap::{Parser, ValueEnum};

/// Color mode for output
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ColorChoice {
    /// Auto-detect based on environment
    Auto,
    /// Always enable colors
    Always,
    /// Never use colors
    Never,
}

/// AI-Powered Shell Command Translator
/// Converts natural language to executable commands
#[derive(Parser, Debug, Clone)]
#[command(name = "clai")]
#[command(version)]
#[command(about = "AI-powered shell command translator", long_about = None)]
pub struct Cli {
    /// Natural language instruction to convert to a command
    #[arg(required = true)]
    pub instruction: String,

    /// Override the AI model to use
    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// Override the AI provider to use
    #[arg(short = 'p', long = "provider")]
    pub provider: Option<String>,

    /// Suppress non-essential output
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Increase verbosity (can be used multiple times)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Disable colored output (deprecated: use --color=never)
    #[arg(long = "no-color")]
    pub no_color: bool,

    /// Control colored output: auto (default), always, or never
    #[arg(long = "color", default_value = "auto")]
    pub color: ColorChoice,

    /// Interactive mode: prompt for execute/copy/abort on dangerous commands
    #[arg(short = 'i', long = "interactive")]
    pub interactive: bool,

    /// Skip dangerous command confirmation
    #[arg(short = 'f', long = "force")]
    pub force: bool,

    /// Show command without execution prompt
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Additional context file
    #[arg(short = 'c', long = "context")]
    pub context: Option<String>,

    /// Offline mode (fail gracefully if no local model available)
    #[arg(long = "offline")]
    pub offline: bool,

    /// Number of command options to generate (default: 3)
    #[arg(short = 'o', long = "options", default_value = "3")]
    pub num_options: u8,

    /// Show the prompt that will be sent to the AI (for debugging)
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,
}

/// Pure function to parse CLI arguments into Cli struct
/// Returns Result with clap::Error on parse failure
/// No side effects - pure function
pub fn parse_args() -> Result<Cli, clap::Error> {
    Cli::try_parse()
}
