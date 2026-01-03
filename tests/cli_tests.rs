use std::process::Command;

fn run_clai(args: &[&str]) -> (String, String, i32) {
    let output = Command::new("./target/debug/clai")
        .args(args)
        .output()
        .expect("Failed to execute clai");
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    
    (stdout, stderr, exit_code)
}

#[test]
fn test_missing_instruction_returns_exit_2() {
    let (_stdout, _stderr, exit_code) = run_clai(&[]);
    assert_eq!(exit_code, 2, "Missing INSTRUCTION should return exit code 2");
}

#[test]
fn test_invalid_flag_returns_exit_2() {
    let (_stdout, _stderr, exit_code) = run_clai(&["--invalid-flag", "test"]);
    assert_eq!(exit_code, 2, "Invalid flag should return exit code 2");
}

#[test]
fn test_valid_instruction_parses() {
    let (stdout, _stderr, exit_code) = run_clai(&["list files"]);
    assert_eq!(exit_code, 0, "Valid instruction should return exit code 0");
    assert!(stdout.contains("list files"), "Output should contain instruction");
}

#[test]
fn test_all_flags_parse_correctly() {
    let (stdout, _stderr, exit_code) = run_clai(&[
        "--quiet",
        "--verbose",
        "--no-color",
        "--interactive",
        "--force",
        "--dry-run",
        "--offline",
        "--model", "test-model",
        "--provider", "test-provider",
        "test instruction"
    ]);
    assert_eq!(exit_code, 0, "All flags should parse correctly");
    assert!(stdout.contains("test instruction"), "Instruction should be parsed");
}

#[test]
fn test_help_output() {
    let (stdout, _stderr, exit_code) = run_clai(&["--help"]);
    assert_eq!(exit_code, 0, "Help should return exit code 0");
    assert!(stdout.contains("Usage:"), "Help should contain usage information");
    assert!(stdout.contains("clai"), "Help should contain binary name");
}

#[test]
fn test_version_output() {
    let (stdout, _stderr, exit_code) = run_clai(&["--version"]);
    assert_eq!(exit_code, 0, "Version should return exit code 0");
    assert!(stdout.contains("clai"), "Version should contain binary name");
    assert!(stdout.contains("0.1.0"), "Version should contain version number");
}
