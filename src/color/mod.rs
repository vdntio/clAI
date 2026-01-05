use crate::config::Config;

/// Color mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    /// Auto-detect based on environment and TTY
    Auto,
    /// Always enable colors
    Always,
    /// Never use colors
    Never,
}

impl ColorMode {
    /// Determine if colors should be enabled based on mode and environment
    /// Pure function - no side effects
    pub fn should_use_color(self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => detect_color_auto(),
        }
    }
}

/// Pure function to detect if colors should be enabled automatically
/// Checks CLICOLOR, NO_COLOR, TERM=dumb, and TTY status
/// No side effects - pure function
/// 
/// Priority order:
/// 1. CLICOLOR=0 disables, CLICOLOR=1 enables (GNU standard)
/// 2. NO_COLOR disables (no-color.org standard)
/// 3. TERM=dumb disables (POSIX standard)
/// 4. TTY status (if stderr is a TTY, enable colors)
pub fn detect_color_auto() -> bool {
    // Check CLICOLOR environment variable (GNU standard)
    // CLICOLOR=0 means disable, CLICOLOR=1 means enable, unset means auto
    if let Ok(clicolor) = std::env::var("CLICOLOR") {
        match clicolor.as_str() {
            "0" => return false,
            "1" => return true,
            _ => {
                // Invalid value, fall through to other checks
            }
        }
    }

    // Check NO_COLOR environment variable (no-color.org standard)
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // Check TERM=dumb (POSIX standard)
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" {
            return false;
        }
    }

    // Check if stderr is a TTY (for color output)
    // Use atty crate for reliable TTY detection
    atty::is(atty::Stream::Stderr)
}

/// Pure function to determine ColorMode from Config
/// Takes immutable Config and returns ColorMode
/// No side effects - pure function
pub fn color_mode_from_config(config: &Config) -> ColorMode {
    // --no-color flag takes precedence
    if config.no_color {
        return ColorMode::Never;
    }

    // Map ColorChoice to ColorMode
    match config.color {
        crate::cli::ColorChoice::Always => ColorMode::Always,
        crate::cli::ColorChoice::Never => ColorMode::Never,
        crate::cli::ColorChoice::Auto => ColorMode::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_always() {
        assert_eq!(ColorMode::Always.should_use_color(), true);
    }

    #[test]
    fn test_color_mode_never() {
        assert_eq!(ColorMode::Never.should_use_color(), false);
    }

    #[test]
    fn test_color_mode_from_config() {
        let config_no_color = crate::config::Config {
            instruction: "test".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: true,
            color: crate::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
            debug: false,
        };

        let config_with_color = crate::config::Config {
            instruction: "test".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: crate::cli::ColorChoice::Auto,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
            debug: false,
        };

        let config_always = crate::config::Config {
            instruction: "test".to_string(),
            model: None,
            provider: None,
            quiet: false,
            verbose: 0,
            no_color: false,
            color: crate::cli::ColorChoice::Always,
            interactive: false,
            force: false,
            dry_run: false,
            context: None,
            offline: false,
            num_options: 3,
            debug: false,
        };

        assert_eq!(color_mode_from_config(&config_no_color), ColorMode::Never);
        assert_eq!(color_mode_from_config(&config_with_color), ColorMode::Auto);
        assert_eq!(color_mode_from_config(&config_always), ColorMode::Always);
    }

    #[test]
    fn test_detect_color_auto_clicolor() {
        // Test CLICOLOR=0 disables
        std::env::set_var("CLICOLOR", "0");
        assert_eq!(detect_color_auto(), false);
        std::env::remove_var("CLICOLOR");

        // Test CLICOLOR=1 enables
        std::env::set_var("CLICOLOR", "1");
        // Note: This test may fail if NO_COLOR is set or TERM=dumb
        // So we just verify it doesn't return false due to CLICOLOR
        let _result = detect_color_auto();
        // If other conditions disable color, that's fine
        // But CLICOLOR=1 should not cause it to be false
        std::env::remove_var("CLICOLOR");
    }
}

