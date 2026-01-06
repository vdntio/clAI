// Simple example to test context gathering
// Run with: cargo run --example test_context

use clai::config::Config;
use clai::context::gatherer::gather_context;

fn main() {
    println!("Testing Context Gathering...\n");

    // Create a test config
    let config = Config {
        instruction: "test instruction".to_string(),
        model: None,
        provider: None,
        quiet: false,
        verbose: 0,
        no_color: false,
        color: clai::cli::ColorChoice::Auto,
        interactive: false,
        force: false,
        dry_run: false,
        context: None,
        offline: false,
        num_options: 3,
        debug: false,
        debug_log_file: None,
    };

    // Gather context
    match gather_context(&config) {
        Ok(json_str) => {
            println!("✅ Context gathered successfully!\n");
            println!("=== Context JSON Output ===\n");
            println!("{}", json_str);
            println!("\n=== End of Context Output ===\n");

            // Parse and display summary
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                println!("=== Context Summary ===");

                if let Some(system) = parsed.get("system").and_then(|s| s.as_object()) {
                    println!("System:");
                    println!(
                        "  OS: {}",
                        system.get("os_name").unwrap_or(&serde_json::Value::Null)
                    );
                    println!(
                        "  Shell: {}",
                        system.get("shell").unwrap_or(&serde_json::Value::Null)
                    );
                    println!(
                        "  Architecture: {}",
                        system
                            .get("architecture")
                            .unwrap_or(&serde_json::Value::Null)
                    );
                }

                if let Some(cwd) = parsed.get("cwd").and_then(|c| c.as_str()) {
                    println!("Current Directory: {}", cwd);
                }

                if let Some(files) = parsed.get("files").and_then(|f| f.as_array()) {
                    println!("Files in directory: {}", files.len());
                    if files.len() > 0 {
                        println!("  (showing first 5)");
                        for (i, file) in files.iter().take(5).enumerate() {
                            if let Some(f) = file.as_str() {
                                println!("    {}. {}", i + 1, f);
                            }
                        }
                    }
                }

                if let Some(history) = parsed.get("history").and_then(|h| h.as_array()) {
                    println!("Shell History: {} commands", history.len());
                    for (i, cmd) in history.iter().enumerate() {
                        if let Some(c) = cmd.as_str() {
                            println!("  {}. {}", i + 1, c);
                        }
                    }
                }

                if let Some(stdin) = parsed.get("stdin") {
                    if stdin.is_null() {
                        println!("Stdin: (not piped)");
                    } else if let Some(s) = stdin.as_str() {
                        println!("Stdin: {} bytes", s.len());
                        if s.len() > 0 {
                            let preview = if s.len() > 50 {
                                format!("{}...", &s[..50])
                            } else {
                                s.to_string()
                            };
                            println!("  Preview: {}", preview);
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to gather context: {}", e);
            std::process::exit(1);
        }
    }
}
