//! Locale detection and formatting utilities
//!
//! Provides locale-aware formatting for dates, numbers, and messages.
//! Detects locale from LANG environment variable.

/// Get the current locale from environment
///
/// Returns the locale string (e.g., "en_US.UTF-8", "C", "fr_FR")
/// Defaults to "en_US" if LANG is not set.
///
/// Pure function - no side effects
pub fn get_locale() -> String {
    std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string())
}

/// Get the locale language code (e.g., "en", "fr", "de")
///
/// Extracts the language part from locale string.
/// Examples:
/// - "en_US.UTF-8" -> "en"
/// - "fr_FR" -> "fr"
/// - "C" -> "C"
///
/// Pure function - no side effects
pub fn get_language_code() -> String {
    let locale = get_locale();

    // Extract language code (first part before underscore or dot)
    locale
        .split('_')
        .next()
        .unwrap_or(&locale)
        .split('.')
        .next()
        .unwrap_or(&locale)
        .to_string()
}

/// Check if locale is set to C (POSIX locale)
///
/// The C locale typically means no locale-specific formatting.
///
/// Pure function - no side effects
pub fn is_c_locale() -> bool {
    let locale = get_locale();
    locale == "C" || locale == "POSIX"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_code() {
        // Test various locale formats
        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(get_language_code(), "en");
        std::env::remove_var("LANG");

        std::env::set_var("LANG", "fr_FR");
        assert_eq!(get_language_code(), "fr");
        std::env::remove_var("LANG");

        std::env::set_var("LANG", "C");
        assert_eq!(get_language_code(), "C");
        std::env::remove_var("LANG");
    }

    #[test]
    fn test_is_c_locale() {
        std::env::set_var("LANG", "C");
        assert_eq!(is_c_locale(), true);
        std::env::remove_var("LANG");

        std::env::set_var("LANG", "POSIX");
        assert_eq!(is_c_locale(), true);
        std::env::remove_var("LANG");

        std::env::set_var("LANG", "en_US.UTF-8");
        assert_eq!(is_c_locale(), false);
        std::env::remove_var("LANG");
    }

    #[test]
    fn test_get_locale_default() {
        // Remove LANG to test default
        std::env::remove_var("LANG");
        let locale = get_locale();
        assert_eq!(locale, "en_US");
    }
}
