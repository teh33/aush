use crate::executor::ExecutionResult;
use crate::runtime::Runtime;
use anyhow::{anyhow, Result};
use nu_ansi_term::Color;

struct BuiltinHelp {
    name: &'static str,
    brief: &'static str,
    description: &'static str,
    usage: &'static str,
    examples: &'static [&'static str],
}

const BUILTINS: &[BuiltinHelp] = &[
    BuiltinHelp {
        name: "cd",
        brief: "Change the current directory",
        description: "Change the shell working directory. If no argument is given, changes to the home directory. Supports ~ expansion for home directory and - for previous directory.",
        usage: "cd [dir]",
        examples: &[
            "cd                  # Change to home directory",
            "cd /tmp             # Change to /tmp",
            "cd ~/Documents      # Change to Documents in home",
            "cd ..               # Go up one directory",
        ],
    },
    BuiltinHelp {
        name: "pwd",
        brief: "Print the current working directory",
        description: "Print the absolute pathname of the current working directory.",
        usage: "pwd",
        examples: &[
            "pwd                 # Show current directory",
        ],
    },
    BuiltinHelp {
        name: "echo",
        brief: "Display a line of text",
        description: "Display the given arguments separated by spaces, followed by a newline.",
        usage: "echo [arg ...]",
        examples: &[
            "echo hello          # Output: hello",
            "echo hello world    # Output: hello world",
            "echo $USER          # Output: current username",
        ],
    },
    BuiltinHelp {
        name: "exit",
        brief: "Exit the shell",
        description: "Exit the shell with an optional exit status. If no status is given, exits with 0.",
        usage: "exit [n]",
        examples: &[
            "exit                # Exit with status 0",
            "exit 1              # Exit with status 1",
        ],
    },
    BuiltinHelp {
        name: "exec",
        brief: "Replace shell or modify redirections",
        description: "Replace the shell process with a command, or perform permanent file descriptor redirections. With a command, exec replaces the shell and never returns. Without a command but with redirections, it permanently redirects the shell's file descriptors.",
        usage: "exec [command [args ...]]\nexec [redirections]",
        examples: &[
            "exec ./server        # Replace shell with server",
            "exec > output.log    # Redirect stdout permanently",
            "exec 2>&1            # Redirect stderr to stdout",
            "exec 3> file.txt     # Open fd 3 for writing",
            "exec 3>&-            # Close fd 3",
        ],
    },
    BuiltinHelp {
        name: "export",
        brief: "Set environment variables",
        description: "Set environment variables that will be passed to child processes. Variables must be specified in KEY=value format.",
        usage: "export VAR=value [VAR=value ...]",
        examples: &[
            "export PATH=/usr/bin:$PATH",
            "export EDITOR=vim",
            "export DEBUG=1 VERBOSE=true",
        ],
    },
    BuiltinHelp {
        name: "source",
        brief: "Execute commands from a file",
        description: "Read and execute commands from a file in the current shell environment. Supports ~ expansion.",
        usage: "source <file>",
        examples: &[
            "source ~/.aushrc    # Load shell configuration",
            "source ./setup.sh   # Execute setup script",
        ],
    },
    BuiltinHelp {
        name: "cat",
        brief: "Concatenate and display files",
        description: "Read files sequentially and write them to standard output. Supports reading from stdin when no files are specified or when - is given.",
        usage: "cat [file ...]",
        examples: &[
            "cat file.txt        # Display file contents",
            "cat file1 file2     # Display multiple files",
            "echo hello | cat    # Read from stdin",
        ],
    },
    BuiltinHelp {
        name: "ls",
        brief: "List directory contents",
        description: "List information about files and directories. By default, lists the current directory. Supports colorized output and various formatting options.",
        usage: "ls [options] [path ...]",
        examples: &[
            "ls                  # List current directory",
            "ls -l               # Long format with details",
            "ls -a               # Show hidden files",
            "ls -lah /tmp        # Long format, all files, human-readable sizes",
        ],
    },
    BuiltinHelp {
        name: "mkdir",
        brief: "Create directories",
        description: "Create one or more directories. By default, fails if parent directories don't exist unless -p option is used.",
        usage: "mkdir [options] directory ...",
        examples: &[
            "mkdir mydir         # Create directory",
            "mkdir -p path/to/dir # Create with parents",
            "mkdir dir1 dir2     # Create multiple directories",
        ],
    },
    BuiltinHelp {
        name: "find",
        brief: "Search for files and directories",
        description: "Search for files in a directory hierarchy. Supports pattern matching with -name and gitignore-aware searching.",
        usage: "find [path] [options]",
        examples: &[
            "find . -name '*.rs' # Find all Rust files",
            "find /tmp -name log # Find files named 'log'",
            "find .              # List all files recursively",
        ],
    },
    BuiltinHelp {
        name: "grep",
        brief: "Search for patterns in files",
        description: "Search for lines matching a pattern in files or stdin. Supports regular expressions and various output modes.",
        usage: "grep [options] pattern [file ...]",
        examples: &[
            "grep 'error' log.txt       # Find 'error' in file",
            "grep -i 'warning' *.log    # Case-insensitive search",
            "cat file | grep 'pattern'  # Search stdin",
            "grep -n 'TODO' src/*.rs    # Show line numbers",
        ],
    },
    BuiltinHelp {
        name: "git-status",
        brief: "Show enhanced git repository status",
        description: "Display the working tree status with colorized output. Shows modified, staged, and untracked files.",
        usage: "git-status",
        examples: &[
            "git-status          # Show git status with colors",
        ],
    },
    BuiltinHelp {
        name: "undo",
        brief: "Undo recent file operations",
        description: "Undo recent file modifications tracked by the shell. Shows available undo operations and allows reverting changes.",
        usage: "undo [options]",
        examples: &[
            "undo                # Show available undo operations",
            "undo -1             # Undo last operation",
        ],
    },
    BuiltinHelp {
        name: "jobs",
        brief: "List background jobs",
        description: "Display status of jobs in the current shell. Shows job ID, status, and command for each background job.",
        usage: "jobs [options]",
        examples: &[
            "jobs                # List all jobs",
            "jobs -l             # List with process IDs",
            "jobs -r             # List only running jobs",
            "jobs -s             # List only stopped jobs",
        ],
    },
    BuiltinHelp {
        name: "fg",
        brief: "Bring job to foreground",
        description: "Move a background job to the foreground. If no job is specified, uses the current job (most recent).",
        usage: "fg [job_spec]",
        examples: &[
            "fg                  # Foreground current job",
            "fg %1               # Foreground job 1",
            "fg 1                # Foreground job 1",
        ],
    },
    BuiltinHelp {
        name: "bg",
        brief: "Resume job in background",
        description: "Resume a stopped job in the background. If no job is specified, uses the current job (most recent).",
        usage: "bg [job_spec]",
        examples: &[
            "bg                  # Resume current job in background",
            "bg %1               # Resume job 1 in background",
            "bg 1                # Resume job 1 in background",
        ],
    },
    BuiltinHelp {
        name: "set",
        brief: "Set or display shell options",
        description: "Set or unset shell options. With no arguments, displays current option settings. Use - to enable and + to disable options.",
        usage: "set [options]",
        examples: &[
            "set                 # Display current options",
            "set -e              # Exit on error (errexit)",
            "set -u              # Error on undefined variables (nounset)",
            "set -x              # Print commands before execution (xtrace)",
            "set -o pipefail     # Fail on pipe errors",
            "set +e              # Disable errexit",
        ],
    },
    BuiltinHelp {
        name: "alias",
        brief: "Create command aliases",
        description: "Create or display command aliases. Aliases are shortcuts for longer commands. With no arguments, displays all defined aliases.",
        usage: "alias [name[=value] ...]",
        examples: &[
            "alias               # List all aliases",
            "alias ll='ls -la'   # Create alias for ls -la",
            "alias gs='git status'",
            "alias ..='cd ..'",
        ],
    },
    BuiltinHelp {
        name: "unalias",
        brief: "Remove command aliases",
        description: "Remove one or more command aliases defined with the alias builtin.",
        usage: "unalias name [name ...]",
        examples: &[
            "unalias ll          # Remove ll alias",
            "unalias gs ll       # Remove multiple aliases",
        ],
    },
    BuiltinHelp {
        name: "test",
        brief: "Evaluate conditional expressions",
        description: "Evaluate conditional expressions for use in shell scripts. Returns exit status 0 (true) or 1 (false). Also available as [ expression ].",
        usage: "test expression\n[ expression ]",
        examples: &[
            "test -f file.txt    # Check if file exists",
            "test -d mydir       # Check if directory exists",
            "[ -z \"$var\" ]       # Check if variable is empty",
            "[ \"$a\" = \"$b\" ]     # String equality",
            "[ 5 -gt 3 ]         # Numeric comparison",
        ],
    },
    BuiltinHelp {
        name: "break",
        brief: "Exit from a loop",
        description: "Exit from a for, while, or until loop. If N is specified, break from N enclosing loops. N must be >= 1. If N is greater than the number of enclosing loops, an error is returned.",
        usage: "break [N]",
        examples: &[
            "break               # Exit innermost loop",
            "break 1             # Same as 'break'",
            "break 2             # Exit from 2 nested loops",
        ],
    },
    BuiltinHelp {
        name: "continue",
        brief: "Resume the next iteration of a loop",
        description: "Resume the next iteration of an enclosing for, while, or until loop. If N is specified, resume at the Nth enclosing loop. N must be >= 1. If N is greater than the number of enclosing loops, an error is returned.",
        usage: "continue [N]",
        examples: &[
            "continue            # Skip to next iteration of innermost loop",
            "continue 1          # Same as 'continue'",
            "continue 2          # Skip to next iteration of 2nd enclosing loop",
        ],
    },
    BuiltinHelp {
        name: "return",
        brief: "Return from a function",
        description: "Return from a shell function with exit status N. If N is not specified, the return status is that of the last command executed in the function. Can only be used inside a function or sourced script.",
        usage: "return [N]",
        examples: &[
            "return              # Return with status 0",
            "return 0            # Return with status 0",
            "return 1            # Return with status 1",
            "return $?           # Return with last command's exit code",
        ],
    },
    BuiltinHelp {
        name: "local",
        brief: "Declare local variables in functions",
        description: "Create function-local variables that shadow global variables of the same name. Local variables are automatically cleaned up when the function exits. Can only be used inside a function. Multiple variables can be declared with or without initial values.",
        usage: "local var[=value] [var[=value] ...]",
        examples: &[
            "local x=5            # Declare local with value",
            "local y              # Declare local without value",
            "local a=1 b=2 c=3    # Multiple declarations",
            "local path=$PATH     # Copy global to local",
        ],
    },
    BuiltinHelp {
        name: "type",
        brief: "Display command type information",
        description: "Display information about command type. Shows whether a command is a builtin, alias, function, or external executable.",
        usage: "type [command ...]",
        examples: &[
            "type cd             # Show type of cd command",
            "type ls grep        # Show type of multiple commands",
            "type -a python      # Show all locations of python",
        ],
    },
    BuiltinHelp {
        name: "trap",
        brief: "Catch signals and execute commands",
        description: "Set trap handlers that execute commands when signals are received. Supports signal names (INT, TERM, HUP) and numbers (2, 15, 1). Special traps: EXIT runs on shell exit, ERR runs when a command fails.",
        usage: "trap [-lp] [command signal_spec ...]\ntrap - signal_spec ...    # Reset to default\ntrap '' signal_spec ...   # Ignore signal",
        examples: &[
            "trap                         # List current traps",
            "trap -l                      # List available signals",
            "trap 'echo bye' EXIT         # Run command on exit",
            "trap 'echo caught' INT       # Catch Ctrl-C",
            "trap 'cleanup' EXIT TERM HUP # Multiple signals",
            "trap - INT                   # Reset INT to default",
            "trap '' INT                  # Ignore INT signal",
        ],
    },
    BuiltinHelp {
        name: "kill",
        brief: "Send signals to processes",
        description: "Send a signal to processes or jobs. By default, sends SIGTERM. Supports signal names (INT, TERM, KILL) and numbers (2, 15, 9). Use -l to list available signals.",
        usage: "kill [-s sigspec | -n signum | -sigspec] pid | jobspec ...\nkill -l [sigspec]",
        examples: &[
            "kill 1234                    # Send SIGTERM to PID 1234",
            "kill -9 1234                 # Send SIGKILL to PID 1234",
            "kill -TERM 1234              # Send SIGTERM by name",
            "kill -INT 1234 5678          # Send SIGINT to multiple PIDs",
            "kill %1                      # Send SIGTERM to job 1",
            "kill -l                      # List available signals",
            "kill -l 15                   # Show signal name for number 15",
        ],
    },
    BuiltinHelp {
        name: "eval",
        brief: "Execute arguments as a shell command",
        description: "Concatenates all arguments into a single string, then parses and executes that string as shell commands. This is useful for dynamic command construction. Variables are expanded twice: once by the shell when passing to eval, and once by eval when executing the command string.",
        usage: "eval [arg ...]",
        examples: &[
            "eval echo hello              # Execute 'echo hello'",
            "cmd='echo hello'; eval $cmd  # Execute command in variable",
            "eval 'x=5; echo $x'          # Multiple statements",
            "eval \"echo \\$HOME\"          # Double expansion",
        ],
    },
    BuiltinHelp {
        name: "help",
        brief: "Display help information",
        description: "Display help information about builtin commands. With no arguments, lists all available builtins. With a command name, shows detailed help for that command.",
        usage: "help [command]",
        examples: &[
            "help                # List all builtins",
            "help cd             # Show help for cd command",
            "help set            # Show help for set command",
        ],
    },
    BuiltinHelp {
        name: "read",
        brief: "Read a line from standard input",
        description: "Read a line from standard input and assign the words to variables. If no variable names are given, the line is stored in REPLY. The line is split using IFS (default: space, tab, newline). If there are more words than variables, the last variable gets all remaining words.",
        usage: "read [-r] [-s] [-t timeout] [-p prompt] [name ...]",
        examples: &[
            "read name           # Read line into $name",
            "read first last     # Split into $first and $last",
            "read                # Read into $REPLY",
            "read -r line        # Raw mode (no backslash processing)",
            "read -s pass        # Silent mode (for passwords)",
            "read -t 5 answer    # Timeout after 5 seconds",
            "read -p 'Name: ' n  # Display prompt before reading",
            "echo 'a b' | read x # Read from pipeline",
        ],
    },
    BuiltinHelp {
        name: "shift",
        brief: "Shift positional parameters",
        description: "Shift positional parameters to the left by N positions. The positional parameters from $N+1 onward are renamed to $1, $2, etc. If N is not given, it defaults to 1. The shift command decrements $# by N. Attempting to shift more parameters than exist is an error.",
        usage: "shift [N]",
        examples: &[
            "shift               # Shift by 1: $2 becomes $1, $3 becomes $2, etc.",
            "shift 2             # Shift by 2: $3 becomes $1, $4 becomes $2, etc.",
            "shift 0             # No-op, parameters unchanged",
            "while [ $# -gt 0 ]; do echo $1; shift; done  # Process all args",
        ],
    },
];

pub fn builtin_help(args: &[String], _runtime: &mut Runtime) -> Result<ExecutionResult> {
    if args.is_empty() {
        // Show all builtins with brief descriptions
        Ok(ExecutionResult::success(show_all_builtins()))
    } else {
        // Show detailed help for specific command
        let command = &args[0];
        match find_builtin(command) {
            Some(builtin) => Ok(ExecutionResult::success(show_detailed_help(builtin))),
            None => Err(anyhow!(
                "help: no help topics match `{}`. Try `help` to see available commands.",
                command
            )),
        }
    }
}

fn show_all_builtins() -> String {
    let mut output = String::new();

    let title_style = Color::Cyan.bold();
    let name_style = Color::Green.bold();

    output.push_str(&title_style.paint("AUSH Shell Builtins").to_string());
    output.push_str("\n\n");
    output.push_str("Type 'help <command>' for more information on a specific command.\n\n");

    // Find the longest name for alignment
    let max_name_len = BUILTINS.iter().map(|b| b.name.len()).max().unwrap_or(0);

    for builtin in BUILTINS {
        let colored_name = name_style.paint(builtin.name).to_string();
        let padding = " ".repeat(max_name_len - builtin.name.len() + 2);
        output.push_str(&format!("  {}{}{}\n", colored_name, padding, builtin.brief));
    }

    output.push('\n');
    output
}

fn show_detailed_help(builtin: &BuiltinHelp) -> String {
    let mut output = String::new();

    let title_style = Color::Cyan.bold();
    let heading_style = Color::Yellow.bold();
    let usage_style = Color::Green;
    let example_style = Color::Blue;

    // Title
    output.push_str(&title_style.paint(builtin.name).to_string());
    output.push_str(&format!(" - {}\n\n", builtin.brief));

    // Description
    output.push_str(&heading_style.paint("DESCRIPTION").to_string());
    output.push_str("\n  ");
    output.push_str(builtin.description);
    output.push_str("\n\n");

    // Usage
    output.push_str(&heading_style.paint("USAGE").to_string());
    output.push_str("\n  ");
    output.push_str(&usage_style.paint(builtin.usage).to_string());
    output.push_str("\n\n");

    // Examples
    if !builtin.examples.is_empty() {
        output.push_str(&heading_style.paint("EXAMPLES").to_string());
        output.push_str("\n");
        for example in builtin.examples {
            output.push_str("  ");
            output.push_str(&example_style.paint(*example).to_string());
            output.push_str("\n");
        }
        output.push('\n');
    }

    output
}

fn find_builtin(name: &str) -> Option<&'static BuiltinHelp> {
    BUILTINS.iter().find(|b| b.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_no_args() {
        let mut runtime = Runtime::new();
        let result = builtin_help(&[], &mut runtime).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout().contains("AUSH Shell Builtins"));
        assert!(result.stdout().contains("cd"));
        assert!(result.stdout().contains("echo"));
        assert!(result.stdout().contains("help"));
    }

    #[test]
    fn test_help_specific_command() {
        let mut runtime = Runtime::new();
        let result = builtin_help(&["cd".to_string()], &mut runtime).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout().contains("cd"));
        assert!(result.stdout().contains("DESCRIPTION"));
        assert!(result.stdout().contains("USAGE"));
        assert!(result.stdout().contains("EXAMPLES"));
        assert!(result.stdout().contains("Change the current directory"));
    }

    #[test]
    fn test_help_all_commands() {
        let mut runtime = Runtime::new();

        // Test help for each builtin
        for builtin in BUILTINS {
            let result = builtin_help(&[builtin.name.to_string()], &mut runtime).unwrap();
            assert_eq!(result.exit_code, 0);
            assert!(result.stdout().contains(builtin.name));
            assert!(result.stdout().contains("DESCRIPTION"));
        }
    }

    #[test]
    fn test_help_invalid_command() {
        let mut runtime = Runtime::new();
        let result = builtin_help(&["nonexistent".to_string()], &mut runtime);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no help topics match"));
    }

    #[test]
    fn test_find_builtin() {
        assert!(find_builtin("cd").is_some());
        assert!(find_builtin("help").is_some());
        assert!(find_builtin("nonexistent").is_none());
    }

    #[test]
    fn test_help_examples() {
        let mut runtime = Runtime::new();
        let result = builtin_help(&["set".to_string()], &mut runtime).unwrap();
        assert!(result.stdout().contains("EXAMPLES"));
        assert!(result.stdout().contains("set -e"));
        assert!(result.stdout().contains("Exit on error"));
    }

    #[test]
    fn test_help_local() {
        let mut runtime = Runtime::new();
        let result = builtin_help(&["local".to_string()], &mut runtime).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout().contains("local"));
        assert!(result.stdout().contains("DESCRIPTION"));
        assert!(result.stdout().contains("function-local"));
        assert!(result.stdout().contains("EXAMPLES"));
        assert!(result.stdout().contains("local x=5"));
    }
}
