use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;
use sysinfo::System;

/// Cached system information structure
/// Immutable snapshot of system info, cached per run
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub architecture: String,
    pub shell: String,
    pub user: String,
    pub total_memory: u64,
}

/// Global cached system information
/// Lazy-initialized, thread-safe cache
static SYSTEM_INFO_CACHE: Lazy<RwLock<Option<SystemInfo>>> = Lazy::new(|| RwLock::new(None));

/// Get system information (cached per run)
///
/// This function collects system information on first access and caches it.
/// Subsequent calls return the cached information.
///
/// Pure function after first call - returns cached immutable data
/// First call has I/O side effects (reading system info)
///
/// # Returns
/// * `SystemInfo` - Immutable system information snapshot
pub fn get_system_info() -> SystemInfo {
    // Check cache
    {
        let cache = SYSTEM_INFO_CACHE.read().unwrap();
        if let Some(ref info) = *cache {
            return info.clone();
        }
    }

    // Collect system information
    let mut system = System::new();
    system.refresh_all();

    // Extract OS information
    // sysinfo 0.37: name() and os_version() are associated functions (static methods)
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());

    // Get architecture
    let architecture = std::env::consts::ARCH.to_string();

    // Get shell from environment
    let shell = std::env::var("SHELL")
        .unwrap_or_else(|_| "unknown".to_string())
        .split('/')
        .next_back()
        .unwrap_or("unknown")
        .to_string();

    // Get user from environment
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    // Get total memory
    let total_memory = system.total_memory();

    let info = SystemInfo {
        os_name,
        os_version,
        architecture,
        shell,
        user,
        total_memory,
    };

    // Cache the result
    {
        let mut cache = SYSTEM_INFO_CACHE.write().unwrap();
        *cache = Some(info.clone());
    }

    info
}

/// Format system information as a structured map for prompt context
///
/// Pure function - takes immutable SystemInfo and returns formatted map
/// No side effects
///
/// # Arguments
/// * `info` - System information to format
///
/// # Returns
/// * `HashMap<String, String>` - Formatted system information
pub fn format_system_info(info: &SystemInfo) -> HashMap<String, String> {
    let mut map = HashMap::new();

    map.insert("os_name".to_string(), info.os_name.clone());
    map.insert("os_version".to_string(), info.os_version.clone());
    map.insert("architecture".to_string(), info.architecture.clone());
    map.insert("shell".to_string(), info.shell.clone());
    map.insert("user".to_string(), info.user.clone());
    map.insert(
        "total_memory_mb".to_string(),
        format!("{}", info.total_memory / 1024 / 1024),
    );

    map
}

/// Get formatted system information (convenience function)
///
/// Combines get_system_info() and format_system_info()
///
/// # Returns
/// * `HashMap<String, String>` - Formatted system information
pub fn get_formatted_system_info() -> HashMap<String, String> {
    let info = get_system_info();
    format_system_info(&info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_info_cached() {
        // First call should collect info
        let info1 = get_system_info();

        // Second call should return cached info
        let info2 = get_system_info();

        // Should be equal (cached)
        assert_eq!(info1, info2);
    }

    #[test]
    fn test_format_system_info() {
        let info = SystemInfo {
            os_name: "Linux".to_string(),
            os_version: "5.15.0".to_string(),
            architecture: "x86_64".to_string(),
            shell: "bash".to_string(),
            user: "testuser".to_string(),
            total_memory: 8 * 1024 * 1024 * 1024, // 8 GB
        };

        let formatted = format_system_info(&info);

        assert_eq!(formatted.get("os_name"), Some(&"Linux".to_string()));
        assert_eq!(formatted.get("os_version"), Some(&"5.15.0".to_string()));
        assert_eq!(formatted.get("architecture"), Some(&"x86_64".to_string()));
        assert_eq!(formatted.get("shell"), Some(&"bash".to_string()));
        assert_eq!(formatted.get("user"), Some(&"testuser".to_string()));
        assert_eq!(formatted.get("total_memory_mb"), Some(&"8192".to_string()));
    }

    #[test]
    fn test_format_system_info_pure() {
        let info = SystemInfo {
            os_name: "Test".to_string(),
            os_version: "1.0".to_string(),
            architecture: "test".to_string(),
            shell: "test".to_string(),
            user: "test".to_string(),
            total_memory: 1024,
        };

        // Pure function - same input, same output
        let formatted1 = format_system_info(&info);
        let formatted2 = format_system_info(&info);

        assert_eq!(formatted1, formatted2);
    }

    #[test]
    fn test_system_info_has_required_fields() {
        let info = get_system_info();

        // Verify all fields are populated (not empty)
        assert!(!info.os_name.is_empty());
        assert!(!info.architecture.is_empty());
        // shell and user might be "unknown" but should not be empty
        assert!(!info.shell.is_empty());
        assert!(!info.user.is_empty());
    }
}
