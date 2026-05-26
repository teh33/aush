use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use tempfile::NamedTempFile;

#[test]
fn tail_follow_flag_outputs_initial_tail_and_appended_data() {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "one\ntwo\n").unwrap();
    file.flush().unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("--no-rc")
        .arg("-c")
        .arg(format!("tail -f {}", file.path().display()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_millis(500));
    write!(file, "three\n").unwrap();
    file.flush().unwrap();
    thread::sleep(Duration::from_millis(500));

    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("one\ntwo\n"), "stdout: {stdout}");
    assert!(stdout.contains("three\n"), "stdout: {stdout}");
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
