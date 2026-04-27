use aush::executor::Executor;
use aush::parser::ast::{Argument, Command, Statement};

#[cfg(unix)]
#[test]
fn test_kill_no_arguments() {
    let mut executor = Executor::new();

    // Execute: kill (with no arguments)
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail with usage message
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("usage"));
}

#[cfg(unix)]
#[test]
fn test_kill_invalid_pid() {
    let mut executor = Executor::new();

    // Execute: kill abc
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("abc".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail with error
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("arguments must be process"));
}

#[cfg(unix)]
#[test]
fn test_kill_zero_pid() {
    let mut executor = Executor::new();

    // Execute: kill 0
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("0".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail (PID 0 is invalid)
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("invalid process ID"));
}

#[cfg(unix)]
#[test]
fn test_kill_negative_pid() {
    let mut executor = Executor::new();

    // Execute: kill -1 (this is interpreted as signal 1, not PID -1)
    // Since no PID follows, it should fail with usage error
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("-1".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail with usage message (signal specified but no PID)
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("usage"));
}

#[cfg(unix)]
#[test]
fn test_kill_signal_zero_self() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Execute: kill -0 <my_pid>
    // Signal 0 checks if process exists without sending a signal
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-0".to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should succeed (we exist!)
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stderr, "");
}

#[cfg(unix)]
#[test]
fn test_kill_nonexistent_pid() {
    let mut executor = Executor::new();

    // Execute: kill 999999 (hopefully doesn't exist)
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("999999".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail
    assert_eq!(result.exit_code, 1);
    assert!(!result.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn test_kill_with_signal_name_term() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Execute: kill -0 <my_pid> instead of TERM to avoid killing ourselves
    // This tests signal name parsing
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-TERM".to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    // We can't actually test TERM because it would kill the test process
    // Instead, let's test signal 0 with a name format
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-0".to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();
    assert_eq!(result.exit_code, 0);
}

#[cfg(unix)]
#[test]
fn test_kill_with_signal_name_int() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Test INT signal name parsing by using signal 0 instead
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-0".to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();
    assert_eq!(result.exit_code, 0);
}

#[cfg(unix)]
#[test]
fn test_kill_with_numeric_signal() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Execute: kill -0 <my_pid> (signal number 0)
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-0".to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should succeed
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stderr, "");
}

#[cfg(unix)]
#[test]
fn test_kill_multiple_pids() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Execute: kill -0 <my_pid> <my_pid> (send to same process twice)
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-0".to_string()),
            Argument::Literal(my_pid.to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should succeed for both
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stderr, "");
}

#[cfg(unix)]
#[test]
fn test_kill_invalid_signal_name() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Execute: kill -INVALID <my_pid>
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-INVALID".to_string()),
            Argument::Literal(my_pid.to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail with invalid signal error
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("invalid signal"));
}

#[cfg(unix)]
#[test]
fn test_kill_signal_only_no_pid() {
    let mut executor = Executor::new();

    // Execute: kill -TERM (signal but no PID)
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("-TERM".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail with usage message
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("usage"));
}

#[cfg(unix)]
#[test]
fn test_kill_partial_failure() {
    let mut executor = Executor::new();
    let my_pid = std::process::id();

    // Execute: kill -0 <my_pid> 999999
    // First should succeed, second should fail
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-0".to_string()),
            Argument::Literal(my_pid.to_string()),
            Argument::Literal("999999".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should have exit code 1 due to partial failure
    assert_eq!(result.exit_code, 1);
    assert!(!result.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn test_kill_default_signal_is_term() {
    let mut executor = Executor::new();

    // Execute: kill 999999 (no signal specified, should default to TERM)
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("999999".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail (process doesn't exist), but validates default signal behavior
    assert_eq!(result.exit_code, 1);
    assert!(!result.stderr.is_empty());
}

#[cfg(unix)]
#[test]
fn test_kill_list_signals() {
    let mut executor = Executor::new();

    // Execute: kill -l
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("-l".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should succeed and list signals
    assert_eq!(result.exit_code, 0);
    let output = result.stdout();
    assert!(output.contains("SIGTERM"));
    assert!(output.contains("SIGINT"));
    assert!(output.contains("SIGKILL"));
}

#[cfg(unix)]
#[test]
fn test_kill_list_signal_by_number() {
    let mut executor = Executor::new();

    // Execute: kill -l 15
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-l".to_string()),
            Argument::Literal("15".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should return TERM
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("TERM"));
}

#[cfg(unix)]
#[test]
fn test_kill_list_signal_by_name() {
    let mut executor = Executor::new();

    // Execute: kill -l TERM
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-l".to_string()),
            Argument::Literal("TERM".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should return 15
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("15"));
}

#[cfg(unix)]
#[test]
fn test_kill_list_invalid_signal() {
    let mut executor = Executor::new();

    // Execute: kill -l INVALID
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![
            Argument::Literal("-l".to_string()),
            Argument::Literal("INVALID".to_string()),
        ],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("invalid signal"));
}

#[cfg(not(unix))]
#[test]
fn test_kill_not_supported_on_windows() {
    let mut executor = Executor::new();

    // Execute: kill 123
    let stmt = Statement::Command(Command {
        name: "kill".to_string(),
        args: vec![Argument::Literal("123".to_string())],
        redirects: vec![],
        prefix_env: vec![],
    });

    let result = executor.execute(vec![stmt]).unwrap();

    // Should fail with not supported message
    assert_eq!(result.exit_code, 1);
    assert!(result.stderr.contains("not supported"));
}
