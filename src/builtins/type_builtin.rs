use crate::executor::{ExecutionResult, Output};
use crate::runtime::Runtime;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::env;
use std::path::Path;

pub fn builtin_type(args: &[String], runtime: &mut Runtime) -> Result<ExecutionResult> {
    if args.is_empty() {
        return Err(anyhow!("type: usage: type [-a] name [name ...]"));
    }

    let mut show_all = false;
    let mut names = Vec::new();
    let mut parsing_options = true;

    for arg in args {
        if parsing_options && arg == "--" {
            parsing_options = false;
        } else if parsing_options && arg == "-a" {
            show_all = true;
        } else if parsing_options && arg.starts_with('-') && arg.len() > 1 {
            return Err(anyhow!("type: {}: invalid option", arg));
        } else {
            parsing_options = false;
            names.push(arg);
        }
    }

    if names.is_empty() {
        return Err(anyhow!("type: usage: type [-a] name [name ...]"));
    }

    let mut output = String::new();
    let mut exit_code = 0;

    for name in names {
        let command_types = if show_all {
            get_all_command_types(name, runtime)
        } else {
            get_command_type(name, runtime).into_iter().collect()
        };

        if command_types.is_empty() {
            output.push_str(&format!("{}: not found\n", name));
            exit_code = 1;
            continue;
        }

        for command_type in command_types {
            match command_type {
                CommandType::Builtin => {
                    output.push_str(&format!("{} is a shell builtin\n", name));
                }
                CommandType::Function => {
                    output.push_str(&format!("{} is a function\n", name));
                }
                CommandType::Alias(value) => {
                    output.push_str(&format!("{} is aliased to '{}'\n", name, value));
                }
                CommandType::External(path) => {
                    output.push_str(&format!("{} is {}\n", name, path));
                }
            }
        }
    }

    Ok(ExecutionResult {
        output: Output::Text(output),
        stderr: String::new(),
        exit_code,
        error: None,
    })
}

enum CommandType {
    Builtin,
    Function,
    Alias(String),
    External(String),
}

fn get_all_command_types(name: &str, runtime: &Runtime) -> Vec<CommandType> {
    let mut command_types = Vec::new();

    if is_builtin(name) {
        command_types.push(CommandType::Builtin);
    }

    if let Some(value) = runtime.get_alias(name) {
        command_types.push(CommandType::Alias(value.clone()));
    }

    if runtime.get_function(name).is_some() {
        command_types.push(CommandType::Function);
    }

    for path in find_all_in_path(name) {
        command_types.push(CommandType::External(path));
    }

    command_types
}

fn get_command_type(name: &str, runtime: &Runtime) -> Option<CommandType> {
    get_all_command_types(name, runtime).into_iter().next()
}

fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "cd" | "pwd"
            | "echo"
            | "exit"
            | "export"
            | "source"
            | "cat"
            | "find"
            | "ls"
            | "mkdir"
            | "git-status"
            | "grep"
            | "undo"
            | "jobs"
            | "fg"
            | "bg"
            | "set"
            | "alias"
            | "unalias"
            | "test"
            | "["
            | "help"
            | "type"
            | "shift"
            | "local"
            | "true"
            | "false"
            | "return"
            | "read"
            | "trap"
            | "unset"
            | "printf"
            | "eval"
            | "exec"
            | "kill"
    )
}

fn find_in_path(command: &str) -> Option<String> {
    find_all_in_path(command).into_iter().next()
}

fn find_all_in_path(command: &str) -> Vec<String> {
    // If the command contains a path separator, check if it exists directly
    if command.contains('/') {
        let path = Path::new(command);
        if is_executable_file(path) {
            return vec![command.to_string()];
        }
        return Vec::new();
    }

    let Some(path_env) = env::var("PATH").ok() else {
        return Vec::new();
    };

    let mut matches = Vec::new();
    let mut seen_dirs = HashSet::new();
    let mut seen_paths = HashSet::new();
    for dir in path_env.split(':') {
        let normalized_dir = normalize_path_key(Path::new(dir));
        if !seen_dirs.insert(normalized_dir) {
            continue;
        }
        let full_path = Path::new(dir).join(command);
        if is_executable_file(&full_path) {
            let path_key = normalize_path_key(&full_path);
            if seen_paths.insert(path_key.clone()) {
                matches.push(path_key);
            }
        }
    }

    matches
}

fn normalize_path_key(path: &Path) -> String {
    std::fs::canonicalize(path)
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn is_executable_file(path: &Path) -> bool {
    if !(path.exists() && path.is_file()) {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ast::FunctionDef;
    use crate::runtime::Runtime;

    #[test]
    fn test_type_builtin() {
        let mut runtime = Runtime::new();

        let result = builtin_type(&["cd".to_string()], &mut runtime).unwrap();
        assert_eq!(result.stdout(), "cd is a shell builtin\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_alias() {
        let mut runtime = Runtime::new();

        // Create an alias
        runtime.set_alias("ll".to_string(), "ls -la".to_string());

        let result = builtin_type(&["ll".to_string()], &mut runtime).unwrap();
        assert_eq!(result.stdout(), "ll is aliased to 'ls -la'\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_external() {
        let mut runtime = Runtime::new();

        // Test with a common external command that should exist on most systems
        let result = builtin_type(&["sh".to_string()], &mut runtime).unwrap();
        // sh should be found as an external command
        assert!(result.stdout().contains("sh is /"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_function() {
        let mut runtime = Runtime::new();

        // Define a test function
        let func = FunctionDef {
            name: "myfunc".to_string(),
            params: vec![],
            body: vec![],
        };
        runtime.define_function(func);

        let result = builtin_type(&["myfunc".to_string()], &mut runtime).unwrap();
        assert_eq!(result.stdout(), "myfunc is a function\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_not_found() {
        let mut runtime = Runtime::new();

        let result = builtin_type(&["nonexistent_command_xyz".to_string()], &mut runtime).unwrap();
        assert_eq!(result.stdout(), "nonexistent_command_xyz: not found\n");
        assert_eq!(result.exit_code, 1);
    }

    #[test]
    fn test_type_multiple_args() {
        let mut runtime = Runtime::new();

        let result = builtin_type(&["cd".to_string(), "pwd".to_string()], &mut runtime).unwrap();
        assert!(result.stdout().contains("cd is a shell builtin"));
        assert!(result.stdout().contains("pwd is a shell builtin"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_mixed_args() {
        let mut runtime = Runtime::new();

        // Set up different command types
        runtime.set_alias("ll".to_string(), "ls -la".to_string());
        let func = FunctionDef {
            name: "myfunc".to_string(),
            params: vec![],
            body: vec![],
        };
        runtime.define_function(func);

        let result = builtin_type(
            &["cd".to_string(), "ll".to_string(), "myfunc".to_string()],
            &mut runtime,
        )
        .unwrap();

        assert!(result.stdout().contains("cd is a shell builtin"));
        assert!(result.stdout().contains("ll is aliased to 'ls -la'"));
        assert!(result.stdout().contains("myfunc is a function"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_priority_builtin_over_alias() {
        let mut runtime = Runtime::new();

        // Try to create an alias with the same name as a builtin
        runtime.set_alias("cd".to_string(), "echo fake cd".to_string());

        // type should report cd as a builtin (builtins have priority)
        let result = builtin_type(&["cd".to_string()], &mut runtime).unwrap();
        assert_eq!(result.stdout(), "cd is a shell builtin\n");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_all_reports_multiple_matches() {
        let mut runtime = Runtime::new();
        runtime.set_alias("cd".to_string(), "echo fake cd".to_string());

        let result = builtin_type(&["-a".to_string(), "cd".to_string()], &mut runtime).unwrap();

        let stdout = result.stdout();
        assert!(stdout.contains("cd is a shell builtin\n"));
        assert!(stdout.contains("cd is aliased to 'echo fake cd'\n"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_all_reports_function_and_external_matches() {
        let mut runtime = Runtime::new();
        let func = FunctionDef {
            name: "sh".to_string(),
            params: vec![],
            body: vec![],
        };
        runtime.define_function(func);

        let result = builtin_type(&["-a".to_string(), "sh".to_string()], &mut runtime).unwrap();

        assert!(result.stdout().contains("sh is a function\n"));
        assert!(result.stdout().contains("sh is /"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_type_invalid_option() {
        let mut runtime = Runtime::new();

        let result = builtin_type(&["-z".to_string(), "cd".to_string()], &mut runtime);

        assert!(result.is_err());
    }

    #[test]
    fn test_type_no_args() {
        let mut runtime = Runtime::new();

        let result = builtin_type(&[], &mut runtime);
        assert!(result.is_err());
    }
}
