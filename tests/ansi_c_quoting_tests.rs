use std::process::Command;

fn run_aush(script: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_aush"))
        .arg("-c")
        .arg(script)
        .output()
        .expect("Failed to execute aush");
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_ansi_c_newline() {
    let result = run_aush(r#"echo $'Hello\nWorld'"#);
    assert_eq!(result, "Hello\nWorld\n");
}

#[test]
fn test_ansi_c_tab() {
    let result = run_aush(r#"echo $'Tab:\there'"#);
    assert_eq!(result, "Tab:\there\n");
}

#[test]
fn test_ansi_c_hex() {
    let result = run_aush(r#"echo $'\x41\x42\x43'"#);
    assert_eq!(result, "ABC\n");
}

#[test]
fn test_ansi_c_backslash() {
    let result = run_aush(r#"echo $'back\\slash'"#);
    assert_eq!(result, "back\\slash\n");
}

#[test]
fn test_ansi_c_single_quote() {
    let result = run_aush(r#"echo $'it\'s working'"#);
    assert_eq!(result, "it's working\n");
}

#[test]
fn test_ansi_c_carriage_return() {
    let result = run_aush(r#"echo $'line\r'"#);
    assert_eq!(result, "line\r\n");
}

#[test]
fn test_ansi_c_escape_sequence() {
    let result = run_aush(r#"echo $'\e[31m'"#);
    assert_eq!(result, "\x1b[31m\n");
}

#[test]
fn test_ansi_c_octal() {
    // \101 = 'A' in octal
    let result = run_aush(r#"echo $'\101\102\103'"#);
    assert_eq!(result, "ABC\n");
}

#[test]
fn test_ansi_c_bell() {
    let result = run_aush(r#"echo $'\a'"#);
    assert_eq!(result, "\x07\n");
}

#[test]
fn test_ansi_c_unicode() {
    // \u0041 = 'A'
    let result = run_aush(r#"echo $'\u0041'"#);
    assert_eq!(result, "A\n");
}
