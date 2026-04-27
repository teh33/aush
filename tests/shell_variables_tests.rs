use std::env;
use std::fs;
use std::process::Command;

fn rush_binary() -> String {
    env::var("CARGO_BIN_EXE_aush").unwrap_or_else(|_| "target/debug/aush".to_string())
}

#[test]
fn test_shell_variable_set() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo $SHELL")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("aush"),
        "SHELL should contain 'aush': {}",
        stdout
    );
}

#[test]
fn test_ppid_variable() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo $PPID")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // PPID should be a number
    assert!(!stdout.is_empty(), "PPID should not be empty");
    assert!(
        stdout.parse::<u32>().is_ok(),
        "PPID should be a number: {}",
        stdout
    );
}

#[test]
fn test_ppid_readonly() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("PPID=123")
        .output()
        .expect("Failed to execute aush");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("readonly"),
        "Setting PPID should fail as it's readonly: {}",
        stderr
    );
}

#[test]
fn test_shlvl_increments() {
    // Test that SHLVL starts at 1 in the first shell
    let output = Command::new(rush_binary())
        .env_remove("SHLVL")
        .arg("-c")
        .arg("echo $SHLVL")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "1", "SHLVL should be 1 in first shell");
}

#[test]
fn test_shlvl_increments_in_subshell() {
    let output = Command::new(rush_binary())
        .env_remove("SHLVL")
        .arg("-c")
        .arg("(echo $SHLVL)")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "2", "SHLVL should be 2 in subshell");
}

#[test]
fn test_shlvl_nested_subshells() {
    let output = Command::new(rush_binary())
        .env_remove("SHLVL")
        .arg("-c")
        .arg("(( echo $SHLVL ))")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(stdout, "3", "SHLVL should be 3 in nested subshell");
}

#[test]
fn test_pwd_variable() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo $PWD")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // PWD should be the current directory
    assert!(!stdout.is_empty(), "PWD should not be empty");
    assert!(
        std::path::Path::new(&stdout).is_absolute(),
        "PWD should be absolute path: {}",
        stdout
    );
}

#[test]
fn test_pwd_updates_with_cd() {
    // Create a temp directory for testing
    let temp_dir = env::temp_dir();

    let script = format!("cd {} && echo $PWD", temp_dir.to_string_lossy());

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg(&script)
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(
        stdout,
        temp_dir.to_string_lossy(),
        "PWD should update after cd"
    );
}

#[test]
fn test_oldpwd_tracks_previous_directory() {
    // Get current directory
    let original_dir = env::current_dir().unwrap();
    let temp_dir = env::temp_dir();

    let script = format!("cd {} && echo $OLDPWD", temp_dir.to_string_lossy());

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg(&script)
        .current_dir(&original_dir)
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(
        stdout,
        original_dir.to_string_lossy(),
        "OLDPWD should be previous directory"
    );
}

#[test]
fn test_cd_dash_uses_oldpwd() {
    // Create two test directories
    let temp_dir = env::temp_dir();
    let test_dir1 = temp_dir.join("rush_test_dir1");
    let test_dir2 = temp_dir.join("rush_test_dir2");

    fs::create_dir_all(&test_dir1).ok();
    fs::create_dir_all(&test_dir2).ok();

    let script = format!(
        "cd {} && cd {} && cd - && echo $PWD",
        test_dir1.to_string_lossy(),
        test_dir2.to_string_lossy()
    );

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg(&script)
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // cd - should print the directory it's switching to, then the new PWD
    assert!(
        stdout.contains(&test_dir1.to_string_lossy().to_string()),
        "cd - should switch back to first directory: {}",
        stdout
    );

    // Cleanup
    fs::remove_dir_all(&test_dir1).ok();
    fs::remove_dir_all(&test_dir2).ok();
}

#[test]
fn test_cd_dash_prints_directory() {
    // Create test directories
    let temp_dir = env::temp_dir();
    let test_dir1 = temp_dir.join("rush_test_cdprint1");
    let test_dir2 = temp_dir.join("rush_test_cdprint2");

    fs::create_dir_all(&test_dir1).ok();
    fs::create_dir_all(&test_dir2).ok();

    let script = format!(
        "cd {} && cd {} && cd -",
        test_dir1.to_string_lossy(),
        test_dir2.to_string_lossy()
    );

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg(&script)
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // cd - should print the directory it's switching to (bash behavior)
    assert!(
        stdout.contains(&test_dir1.to_string_lossy().to_string()),
        "cd - should print the directory: {}",
        stdout
    );

    // Cleanup
    fs::remove_dir_all(&test_dir1).ok();
    fs::remove_dir_all(&test_dir2).ok();
}

#[test]
fn test_cd_dash_without_oldpwd() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("cd -")
        .output()
        .expect("Failed to execute aush");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("OLDPWD"),
        "cd - without OLDPWD should error: {}",
        stderr
    );
    assert!(!output.status.success(), "cd - without OLDPWD should fail");
}

#[test]
fn test_oldpwd_chain() {
    // Test that OLDPWD is updated correctly across multiple cd commands
    let temp_dir = env::temp_dir();
    let test_dir1 = temp_dir.join("rush_test_chain1");
    let test_dir2 = temp_dir.join("rush_test_chain2");
    let test_dir3 = temp_dir.join("rush_test_chain3");

    fs::create_dir_all(&test_dir1).ok();
    fs::create_dir_all(&test_dir2).ok();
    fs::create_dir_all(&test_dir3).ok();

    let script = format!(
        "cd {} && cd {} && cd {} && echo $OLDPWD",
        test_dir1.to_string_lossy(),
        test_dir2.to_string_lossy(),
        test_dir3.to_string_lossy()
    );

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg(&script)
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(
        stdout,
        test_dir2.to_string_lossy(),
        "OLDPWD should be the immediately previous directory"
    );

    // Cleanup
    fs::remove_dir_all(&test_dir1).ok();
    fs::remove_dir_all(&test_dir2).ok();
    fs::remove_dir_all(&test_dir3).ok();
}

#[test]
fn test_pwd_stays_in_sync() {
    // Test that PWD stays synchronized with actual working directory
    let temp_dir = env::temp_dir();
    let test_dir = temp_dir.join("rush_test_sync");

    fs::create_dir_all(&test_dir).ok();

    let script = format!(
        "cd {} && test $PWD = $(pwd) && echo MATCH",
        test_dir.to_string_lossy()
    );

    let output = Command::new(rush_binary())
        .arg("-c")
        .arg(&script)
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(
        stdout, "MATCH",
        "PWD should match actual directory from pwd command"
    );

    // Cleanup
    fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_all_standard_variables_present() {
    let output = Command::new(rush_binary())
        .arg("-c")
        .arg("echo $SHELL; echo $PPID; echo $SHLVL; echo $PWD")
        .output()
        .expect("Failed to execute aush");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();

    // SHELL should be set (might be empty if not in environment)
    // Check if at least the binary path is present
    let shell = lines.get(0).unwrap_or(&"");
    assert!(
        shell.contains("aush") || shell.is_empty(),
        "SHELL should contain aush binary path or be empty: {}",
        shell
    );

    // PPID should be a non-empty number
    let ppid = lines.get(1).unwrap_or(&"");
    assert!(
        !ppid.is_empty() && ppid.parse::<u32>().is_ok(),
        "PPID should be set to a number: {}",
        ppid
    );

    // SHLVL should be a non-empty number
    let shlvl = lines.get(2).unwrap_or(&"");
    assert!(
        !shlvl.is_empty() && shlvl.parse::<i32>().is_ok(),
        "SHLVL should be set to a number: {}",
        shlvl
    );

    // PWD should be a non-empty path
    let pwd = lines.get(3).unwrap_or(&"");
    assert!(
        !pwd.is_empty() && pwd.starts_with('/'),
        "PWD should be set to an absolute path: {}",
        pwd
    );
}
