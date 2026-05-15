use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn run_aush(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aush"))
        .args(args)
        .output()
        .unwrap()
}

fn run_aush_with_home(home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aush"))
        .env("HOME", home)
        .env_remove("SHELL")
        .args(args)
        .output()
        .unwrap()
}

fn write_home_file(home: &Path, name: &str, contents: &str) -> PathBuf {
    let path = home.join(name);
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn test_source_builtin() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test_config.aush");

    // Create a config file
    let mut file = fs::File::create(&config_file).unwrap();
    writeln!(file, "echo hello").unwrap();
    drop(file);

    // Test sourcing the file
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg(format!("source {}", config_file.display()))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
}

#[test]
fn test_dot_builtin() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test_config.aush");

    // Create a config file
    let mut file = fs::File::create(&config_file).unwrap();
    writeln!(file, "echo hello_from_dot").unwrap();
    drop(file);

    // Test sourcing the file with dot command (POSIX syntax)
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg(format!(". {}", config_file.display()))
        .output()
        .unwrap();

    assert!(output.status.success(), "dot command should succeed");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "hello_from_dot"
    );
}

#[test]
fn test_dot_nonexistent_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg(". /nonexistent/file.aush")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No such file"));
}

#[test]
fn test_source_with_tilde() {
    // Create a test file in temp directory
    let home = dirs::home_dir().unwrap();
    let test_file = home.join(".aush_test_source");

    // Create test file
    let mut file = fs::File::create(&test_file).unwrap();
    writeln!(file, "echo tilde_success").unwrap();
    drop(file);

    // Test sourcing with ~ expansion
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg("source ~/.aush_test_source")
        .output()
        .unwrap();

    // Cleanup
    fs::remove_file(test_file).ok();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "tilde_success"
    );
}

#[test]
fn test_source_nonexistent_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg("source /nonexistent/file.aush")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No such file"));
}

#[test]
fn test_environment_variables_set() {
    let output = run_aush(&["-c", "printf '[%s]\\n' \"$TERM\" \"$USER\" \"$HOME\""]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "expected environment-backed variables to print"
    );
}

#[test]
fn test_term_variable_set() {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg("echo $TERM")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "TERM should be set");
}

#[test]
fn test_user_variable_set() {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg("echo $USER")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "USER should be set");
}

#[test]
fn test_home_variable_set() {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg("echo $HOME")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty(), "HOME should be set");
}

#[test]
fn test_login_flag() {
    let temp_dir = TempDir::new().unwrap();
    let home = temp_dir.path();
    write_home_file(home, ".aush_profile", "echo from_profile\n");

    let output = run_aush_with_home(home, &["--login", "-c", "echo test"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test"));
    assert!(
        !stdout.contains("from_profile"),
        "-c fast path should not source login startup files: {stdout}"
    );
}

#[test]
fn test_no_rc_flag() {
    let temp_dir = TempDir::new().unwrap();
    let home = temp_dir.path();
    write_home_file(home, ".aushrc", "echo should_not_load\n");

    let output = run_aush_with_home(home, &["--no-rc", "-c", "echo test_output"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("should_not_load"));
    assert!(stdout.contains("test_output"));
}

#[test]
fn test_fast_path_c_mode_skips_rc_files_even_with_login_flag() {
    let temp_dir = TempDir::new().unwrap();
    let home = temp_dir.path();
    write_home_file(home, ".aush_profile", "echo from_profile\n");
    write_home_file(home, ".aushrc", "echo from_rc\n");

    let output = run_aush_with_home(home, &["--login", "-c", "echo command_only"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command_only"));
    assert!(!stdout.contains("from_profile"), "stdout: {stdout}");
    assert!(!stdout.contains("from_rc"), "stdout: {stdout}");
}

#[test]
fn test_source_skips_zsh_completion_files() {
    let temp_dir = TempDir::new().unwrap();
    let completion_file = temp_dir.path().join("_bun");

    let mut file = fs::File::create(&completion_file).unwrap();
    writeln!(file, "#compdef bun").unwrap();
    writeln!(file, "_bun_completion() {{").unwrap();
    writeln!(file, "    _alternative 'files:file:_files'").unwrap();
    writeln!(file, "}}").unwrap();
    drop(file);

    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg(format!(
            "[ -s {} ] && source {}",
            completion_file.display(),
            completion_file.display()
        ))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "expected zsh completion source to be a no-op, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Parse error"), "stderr: {stderr}");
    assert!(!stderr.contains("_alternative"), "stderr: {stderr}");
}

#[test]
fn test_source_with_comments() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test_comments.aush");

    // Create a config file with comments
    let mut file = fs::File::create(&config_file).unwrap();
    writeln!(file, "# This is a comment").unwrap();
    writeln!(file, "echo value1").unwrap();
    writeln!(file, "# Another comment").unwrap();
    writeln!(file, "echo value2").unwrap();
    writeln!(file, "").unwrap(); // Empty line
    writeln!(file, "echo value3").unwrap();
    drop(file);

    // Test sourcing the file
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg(format!("source {}", config_file.display()))
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("value1"));
    assert!(stdout.contains("value2"));
    assert!(stdout.contains("value3"));
}

#[test]
fn test_source_with_error_continues() {
    let temp_dir = TempDir::new().unwrap();
    let config_file = temp_dir.path().join("test_error.aush");

    // Create a config file with an error in the middle
    let mut file = fs::File::create(&config_file).unwrap();
    writeln!(file, "echo before_error").unwrap();
    writeln!(file, "nonexistent_command_that_will_fail").unwrap();
    writeln!(file, "echo after_error").unwrap();
    drop(file);

    // Test sourcing the file - should continue after error
    let output = run_aush(&["-c", &format!("source {}", config_file.display())]);

    // Should complete successfully despite the error
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("before_error"));
    assert!(stdout.contains("after_error"));
    assert!(
        stderr.contains("nonexistent_command_that_will_fail"),
        "stderr should name the failing command: {stderr}"
    );
    assert!(
        !stderr.contains("after_error"),
        "startup errors should not swallow later successful output into stderr: {stderr}"
    );
}
