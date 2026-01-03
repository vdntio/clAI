use clai::config::Config;
use clai::context::gatherer::gather_context;

#[test]
fn test_context_gathering_integration() {
    // Create a test config
    let config = Config {
        instruction: "test instruction".to_string(),
        model: None,
        provider: None,
        quiet: false,
        verbose: 0,
        no_color: false,
        interactive: false,
        force: false,
        dry_run: false,
        context: None,
        offline: false,
    };

    // Gather context
    match gather_context(&config) {
        Ok(json_str) => {
            println!("\n=== Context Gathering Test Output ===\n");
            println!("{}", json_str);
            println!("\n=== End of Context Output ===\n");

            // Verify it's valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&json_str)
                .expect("Context should be valid JSON");

            // Verify required fields exist
            assert!(parsed.get("system").is_some(), "System info should be present");
            assert!(parsed.get("cwd").is_some(), "CWD should be present");
            assert!(parsed.get("files").is_some(), "Files should be present");
            assert!(parsed.get("history").is_some(), "History should be present");
            assert!(parsed.get("stdin").is_some(), "Stdin field should be present");

            // Verify system info has expected fields
            let system = parsed.get("system").unwrap().as_object().unwrap();
            assert!(system.contains_key("os_name"), "System should have os_name");
            assert!(system.contains_key("shell"), "System should have shell");
            assert!(system.contains_key("architecture"), "System should have architecture");

            // Verify cwd is a string
            assert!(parsed.get("cwd").unwrap().is_string(), "CWD should be a string");

            // Verify files is an array
            assert!(parsed.get("files").unwrap().is_array(), "Files should be an array");

            // Verify history is an array
            assert!(parsed.get("history").unwrap().is_array(), "History should be an array");

            println!("✅ All context gathering tests passed!");
        }
        Err(e) => {
            panic!("Failed to gather context: {}", e);
        }
    }
}

