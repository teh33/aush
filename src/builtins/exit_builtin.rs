use crate::executor::ExecutionResult;
use crate::runtime::Runtime;
use anyhow::Result;

/// Special error type for exit from subshells.
///
/// When `exit` is called inside a subshell, we don't want to terminate the
/// entire process. Instead, we throw this signal which gets caught by
/// `execute_subshell()` and converted to an ExecutionResult with the
/// appropriate exit code.
///
/// At the top level (main.rs), this signal is caught and converted to
/// `std::process::exit(code)` so that `exit` still terminates the shell
/// when not inside a subshell.
///
/// This follows the same pattern as ReturnSignal and BreakSignal.
#[derive(Debug)]
pub struct ExitSignal {
    pub exit_code: i32,
}

impl std::fmt::Display for ExitSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "exit {}", self.exit_code)
    }
}

impl std::error::Error for ExitSignal {}

/// Implementation of the `exit` builtin
///
/// Usage:
///   exit [N]
///
/// Exit the shell (or subshell) with exit code N (default 0).
///
/// In a subshell context, this only exits the subshell, not the parent.
/// At top level, this exits the entire shell process.
///
/// Examples:
///   exit        # Exit with code 0
///   exit 1      # Exit with code 1
///   exit $?     # Exit with last command's exit code
pub fn builtin_exit(args: &[String], runtime: &mut Runtime) -> Result<ExecutionResult> {
    let code = if args.is_empty() {
        runtime.get_last_exit_code()
    } else {
        args[0].parse::<i32>().unwrap_or(0)
    };

    Err(anyhow::Error::new(ExitSignal { exit_code: code }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_no_args() {
        let mut runtime = crate::runtime::Runtime::new();
        runtime.set_last_exit_code(5);
        let result = builtin_exit(&[], &mut runtime);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let signal = err.downcast_ref::<ExitSignal>().unwrap();
        assert_eq!(signal.exit_code, 5);
    }

    #[test]
    fn test_exit_with_code() {
        let mut runtime = crate::runtime::Runtime::new();
        let result = builtin_exit(&["5".to_string()], &mut runtime);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let signal = err.downcast_ref::<ExitSignal>().unwrap();
        assert_eq!(signal.exit_code, 5);
    }

    #[test]
    fn test_exit_with_invalid_code() {
        let mut runtime = crate::runtime::Runtime::new();
        let result = builtin_exit(&["not_a_number".to_string()], &mut runtime);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let signal = err.downcast_ref::<ExitSignal>().unwrap();
        assert_eq!(signal.exit_code, 0);
    }
}
