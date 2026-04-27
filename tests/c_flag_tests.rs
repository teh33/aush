// Integration tests for -c flag

use std::process::Command;

fn rush_binary() -> String {
    // Use the release binary for realistic testing
    let mut path = std::env::current_dir().unwrap();
    path.push("target");
    path.push("release");
    path.push("aush");
    path.to_string_lossy().to_string()
}

#[test]
fn test_c_flag_echo() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo hello world")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello world\n");
}

#[test]
fn test_c_flag_pwd() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("pwd")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_c_flag_cat() {
    // Create a test file
    let test_content = "test content\nline 2\n";
    std::fs::write("/tmp/aush_test_cat.txt", test_content).unwrap();

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("cat /tmp/aush_test_cat.txt")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), test_content);

    // Cleanup
    std::fs::remove_file("/tmp/aush_test_cat.txt").ok();
}

#[test]
fn test_c_flag_ls() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("ls")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    // Should contain common files like Cargo.toml
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cargo.toml") || stdout.len() > 0);
}

#[test]
fn test_c_flag_pipeline() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo test | cat")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "test\n");
}

#[test]
fn test_c_flag_exit_code_success() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo success")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_c_flag_multiple_commands() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo first && echo second")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("first"));
    assert!(stdout.contains("second"));
}

#[test]
fn test_help_flag() {
    let output = Command::new(rush_binary())
        .arg("--help")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AUSH v0.1.0"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("-c"));
}

#[test]
fn test_h_flag() {
    let output = Command::new(rush_binary())
        .arg("-h")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("AUSH v0.1.0"));
}

#[test]
fn test_c_flag_find() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("find . -name \"Cargo.toml\"")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cargo.toml"));
}

#[test]
fn test_c_flag_grep() {
    // Create a test file
    std::fs::write("/tmp/aush_test_grep.txt", "line 1\nFOUND\nline 3\n").unwrap();

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("grep FOUND /tmp/aush_test_grep.txt")
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("FOUND"));

    // Cleanup
    std::fs::remove_file("/tmp/aush_test_grep.txt").ok();
}
