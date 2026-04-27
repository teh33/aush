use aush::executor::Executor;
use aush::parser::ast::{Argument, Command, Statement};

/// Test exec with no arguments is a no-op
#[test]
fn test_exec_no_arguments() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]).unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout(), "");
}

/// Test exec with builtin command fails
#[test]
fn test_exec_builtin_error() {
    let mut executor = Executor::new();

    // exec cd /tmp should fail because cd is a builtin
    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![
            Argument::Literal("cd".to_string()),
            Argument::Literal("/tmp".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("cannot execute builtin"),
        "Expected 'cannot execute builtin' error, got: {}",
        err_msg
    );
}

/// Test exec with nonexistent command fails
#[test]
fn test_exec_nonexistent_command() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![Argument::Literal(
            "nonexistent_command_xyz12345".to_string(),
        )],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("command not found"),
        "Expected 'command not found' error, got: {}",
        err_msg
    );
}

/// Test exec with absolute path to nonexistent file
#[test]
fn test_exec_absolute_path_not_found() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![Argument::Literal(
            "/nonexistent/path/to/command".to_string(),
        )],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
}

/// Test exec errors when trying to execute a builtin (echo)
#[test]
fn test_exec_echo_builtin_error() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![
            Argument::Literal("echo".to_string()),
            Argument::Literal("hello".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot execute builtin"));
}

/// Test exec errors when trying to execute true builtin
#[test]
fn test_exec_true_builtin_error() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![Argument::Literal("true".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot execute builtin"));
}

/// Test exec errors when trying to execute false builtin
#[test]
fn test_exec_false_builtin_error() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![Argument::Literal("false".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot execute builtin"));
}

/// Test exec errors when trying to execute pwd builtin
#[test]
fn test_exec_pwd_builtin_error() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![Argument::Literal("pwd".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot execute builtin"));
}

/// Test exec errors when trying to execute test builtin
#[test]
fn test_exec_test_builtin_error() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![
            Argument::Literal("test".to_string()),
            Argument::Literal("-f".to_string()),
            Argument::Literal("/etc/passwd".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot execute builtin"));
}

/// Test exec errors when trying to execute set builtin
#[test]
fn test_exec_set_builtin_error() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![
            Argument::Literal("set".to_string()),
            Argument::Literal("-e".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("cannot execute builtin"));
}

/// Test exec with relative path to nonexistent file
#[test]
fn test_exec_relative_path_not_found() {
    let mut executor = Executor::new();

    let cmd = Statement::Command(Command {
        name: "exec".to_string(),
        args: vec![Argument::Literal("./nonexistent_script.sh".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![cmd]);
    assert!(result.is_err());
}

// Note: We cannot test actual process replacement in unit tests
// because exec replaces the test process itself and would cause
// the test runner to exit. Actual exec functionality with process
// replacement should be tested manually or via end-to-end tests
// that spawn a separate shell process.
//
// The following behaviors cannot be tested in integration tests:
// - exec echo hello (would replace test process with echo)
// - exec ls (would replace test process with ls)
// - exec with redirections (would affect test process's FDs)
// - exec env FOO=bar printenv FOO (would replace test process)
//
// These behaviors are tested in the unit tests in exec.rs where
// we can test the individual functions without actually calling exec.

#[cfg(unix)]
mod subprocess_tests {
    use std::process::Command;

    /// Test exec actually replaces shell with external command
    /// This test spawns a subprocess to verify exec works
    #[test]
    fn test_exec_replaces_shell() {
        // Build the aush binary path
        let rush_binary = env!("CARGO_BIN_EXE_aush");

        // Run aush with a command that uses exec to run /bin/echo
        // If exec works, the shell is replaced and we get the output from echo
        let output = Command::new(rush_binary)
            .arg("-c")
            .arg("exec /bin/echo hello from exec")
            .output()
            .expect("Failed to execute aush");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("hello from exec"),
            "Expected 'hello from exec' in output, got: {}",
            stdout
        );
    }

    /// Test exec with arguments passes arguments correctly
    #[test]
    fn test_exec_with_arguments() {
        let rush_binary = env!("CARGO_BIN_EXE_aush");

        let output = Command::new(rush_binary)
            .arg("-c")
            .arg("exec /bin/echo arg1 arg2 arg3")
            .output()
            .expect("Failed to execute aush");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("arg1 arg2 arg3"),
            "Expected 'arg1 arg2 arg3' in output, got: {}",
            stdout
        );
    }

    /// Test exec failure doesn't crash the shell for nonexistent command
    #[test]
    fn test_exec_failure_nonexistent() {
        let rush_binary = env!("CARGO_BIN_EXE_aush");

        let output = Command::new(rush_binary)
            .arg("-c")
            .arg("exec nonexistent_command_xyz; echo should not print")
            .output()
            .expect("Failed to execute aush");

        // The shell should error out on exec failure
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("command not found") || stderr.contains("not found"),
            "Expected error message in stderr, got: {}",
            stderr
        );
    }
}
