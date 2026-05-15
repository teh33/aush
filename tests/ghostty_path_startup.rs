use std::process::Command;

#[test]
fn ghostty_path_startup_uses_default_path_when_path_is_empty() {
    let aush = env!("CARGO_BIN_EXE_aush");

    let output = Command::new(aush)
        .env_clear()
        .env("PATH", "")
        .env("TERM", "xterm-ghostty")
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        )
        .env("USER", "aush-test")
        .arg("-c")
        .arg("uname")
        .output()
        .expect("failed to run aush");

    assert!(
        output.status.success(),
        "aush should resolve external commands with an empty startup PATH; stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "uname should produce output"
    );
}

#[test]
fn ghostty_path_startup_extends_incomplete_path() {
    let aush = env!("CARGO_BIN_EXE_aush");

    let output = Command::new(aush)
        .env_clear()
        .env("PATH", "/tmp")
        .env("TERM", "xterm-ghostty")
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()),
        )
        .env("USER", "aush-test")
        .arg("-c")
        .arg("uname")
        .output()
        .expect("failed to run aush");

    assert!(
        output.status.success(),
        "aush should append standard system paths when startup PATH is incomplete; stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
