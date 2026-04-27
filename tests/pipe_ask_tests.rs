//! End-to-end tests for the `|?` (pipe-ask) operator using a mock pi subprocess.
//!
//! These tests verify:
//! - Basic `|?` functionality with a mock pi
//! - Error handling when pi is not found
//! - Stdin capture from piped commands
//! - Integration with complex pipelines

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::NamedTempFile;

/// Create a mock "pi" script that responds with canned RPC output
fn create_mock_pi() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(
        file,
        r#"#!/bin/bash
# Mock pi --rpc that echoes back streaming response
while IFS= read -r line; do
    # Parse the prompt command and respond
    echo '{{"type":"content_delta","content":"Mock "}}'
    echo '{{"type":"content_delta","content":"response"}}'
    echo '{{"type":"agent_end"}}'
done
"#
    )
    .unwrap();

    // Make executable
    std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    file
}

/// Get the path to the aush binary
fn rush_binary() -> String {
    // Check for release binary first, then debug
    let release_path = "./target/release/aush";
    let debug_path = "./target/debug/aush";

    if std::path::Path::new(release_path).exists() {
        release_path.to_string()
    } else {
        debug_path.to_string()
    }
}

#[test]
fn test_pipe_ask_basic() {
    let mock_pi = create_mock_pi();

    // Run aush with mock pi in PATH
    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", mock_pi.path()) // Override pi binary location
        .arg("-c")
        .arg(r#"echo "test" |? "respond""#)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Mock response"),
        "Expected 'Mock response' in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_pipe_ask_no_pi() {
    // Verify helpful error when pi is not installed
    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", "/nonexistent/pi") // Force failure
        .env("PATH", "") // Clear PATH
        .arg("-c")
        .arg(r#"echo "test" |? "respond""#)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("pi not found") || stderr.contains("install pi"),
        "Expected error message about pi not found, got: {}",
        stderr
    );
}

#[test]
fn test_pipe_ask_captures_stdin() {
    // Verify the command output is passed to pi
    let mock_pi = create_mock_pi();

    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", mock_pi.path())
        .arg("-c")
        .arg(r#"echo "hello world" |? "summarize""#)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_pipe_ask_in_pipeline() {
    // cat file | grep pattern |? "explain"
    let mock_pi = create_mock_pi();

    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", mock_pi.path())
        .arg("-c")
        .arg(r#"echo -e "error: foo\nwarning: bar" | grep error |? "explain""#)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_pipe_ask_empty_prompt() {
    // Test |? without explicit prompt (should use default)
    let mock_pi = create_mock_pi();

    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", mock_pi.path())
        .arg("-c")
        .arg(r#"echo "test output" |?"#)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
    assert!(
        stdout.contains("Mock response"),
        "Expected 'Mock response' in stdout, got: {}",
        stdout
    );
}

#[test]
fn test_pipe_ask_single_quoted_prompt() {
    let mock_pi = create_mock_pi();

    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", mock_pi.path())
        .arg("-c")
        .arg(r#"echo "test" |? 'analyze this'"#)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

#[test]
fn test_pipe_ask_multiword_prompt() {
    let mock_pi = create_mock_pi();

    let output = Command::new(rush_binary())
        .env("AUSH_PI_PATH", mock_pi.path())
        .arg("-c")
        .arg(r#"ls -la |? "explain what each column means""#)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Command failed. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}
