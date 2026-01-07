use std::process::Command;

fn run_clai(args: &[&str]) -> (String, String, i32) {
    // Use CARGO_BIN_EXE_clai which is set by cargo test to the correct binary path
    let binary_path = env!("CARGO_BIN_EXE_clai");

    let output = Command::new(binary_path)
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
    assert_eq!(
        exit_code, 2,
        "Missing INSTRUCTION should return exit code 2"
    );
}

#[test]
fn test_invalid_flag_returns_exit_2() {
    let (_stdout, _stderr, exit_code) = run_clai(&["--invalid-flag", "test"]);
    assert_eq!(exit_code, 2, "Invalid flag should return exit code 2");
}

#[test]
fn test_help_output() {
    let (stdout, _stderr, exit_code) = run_clai(&["--help"]);
    assert_eq!(exit_code, 0, "Help should return exit code 0");
    assert!(
        stdout.contains("Usage:"),
        "Help should contain usage information"
    );
    assert!(stdout.contains("clai"), "Help should contain binary name");
}

#[test]
fn test_version_output() {
    let (stdout, _stderr, exit_code) = run_clai(&["--version"]);
    assert_eq!(exit_code, 0, "Version should return exit code 0");
    assert!(
        stdout.contains("clai"),
        "Version should contain binary name"
    );
    assert!(
        stdout.contains("0.1.0"),
        "Version should contain version number"
    );
}

#[test]
fn test_offline_not_supported() {
    // --offline is not yet implemented and should return an error
    let (_stdout, stderr, exit_code) = run_clai(&["--offline", "test"]);
    assert_eq!(
        exit_code, 1,
        "Offline mode should return exit code 1 (not supported)"
    );
    assert!(
        stderr.contains("Offline mode is not yet supported"),
        "Should show offline not supported message"
    );
}

// Note: Integration tests that require actual API calls or network access
// are not reliable in CI environments. The unit tests in src/ cover the
// error handling paths. These integration tests focus on CLI argument parsing.
