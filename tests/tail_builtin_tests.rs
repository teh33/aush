use std::process::Command;

use tempfile::NamedTempFile;

#[test]
fn tail_follow_flag_fails_loudly_for_now() {
    let file = NamedTempFile::new().unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("--no-rc")
        .arg("-c")
        .arg(format!("tail -f {}", file.path().display()))
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("-f/--follow"), "stderr: {stderr}");
    assert!(stderr.contains("not supported"), "stderr: {stderr}");
}

#[test]
fn tail_pipeline_uses_builtin_tail() {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("--no-rc")
        .arg("-c")
        .arg("printf 'one\\ntwo\\nthree\\n' | tail -n 2")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "two\nthree\n");
}
