use crate::executor::{ExecutionResult, Output};
use crate::runtime::Runtime;
use anyhow::{anyhow, Result};

pub fn builtin_history(args: &[String], runtime: &mut Runtime) -> Result<ExecutionResult> {
    let history = runtime.history_mut();

    if args.is_empty() {
        // Show all history (or recent entries)
        let entries = history.last_n(100); // Show last 100 by default
        let mut output = String::new();

        for (i, entry) in entries.iter().enumerate() {
            if history.config().show_timestamps {
                output.push_str(&format!(
                    "{:5} {} {}\n",
                    i + 1,
                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    entry.command
                ));
            } else {
                output.push_str(&format!("{:5} {}\n", i + 1, entry.command));
            }
        }

        return Ok(ExecutionResult::success(output));
    }

    match args[0].as_str() {
        "search" => {
            if args.len() < 2 {
                return Err(anyhow!("Usage: history search <query>"));
            }

            let query = args[1..].join(" ");
            let results = history.search(&query, 20);

            if results.is_empty() {
                return Ok(ExecutionResult::success(
                    "No matching commands found\n".to_string(),
                ));
            }

            let mut output = String::new();
            for result in results {
                if history.config().show_timestamps {
                    output.push_str(&format!(
                        "[score: {}] {} {}\n",
                        result.score,
                        result.entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        result.entry.command
                    ));
                } else {
                    output.push_str(&format!(
                        "[score: {}] {}\n",
                        result.score, result.entry.command
                    ));
                }
            }

            Ok(ExecutionResult::success(output))
        }
        "clear" => {
            history.clear()?;
            Ok(ExecutionResult::success("History cleared\n".to_string()))
        }
        _ => {
            // Try to parse as a number
            if let Ok(n) = args[0].parse::<usize>() {
                let entries = history.last_n(n);
                let mut output = String::new();

                for (i, entry) in entries.iter().enumerate() {
                    if history.config().show_timestamps {
                        output.push_str(&format!(
                            "{:5} {} {}\n",
                            i + 1,
                            entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                            entry.command
                        ));
                    } else {
                        output.push_str(&format!("{:5} {}\n", i + 1, entry.command));
                    }
                }

                Ok(ExecutionResult::success(output))
            } else {
                Err(anyhow!("Usage: history [N | search <query> | clear]"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::History;

    fn setup_test_runtime() -> Runtime {
        let mut runtime = Runtime::new();
        let history = runtime.history_mut();

        history.add("echo hello".to_string()).unwrap();
        history.add("ls -la".to_string()).unwrap();
        history.add("git status".to_string()).unwrap();
        history.add("cargo build".to_string()).unwrap();
        history.add("cargo test".to_string()).unwrap();

        runtime
    }

    #[test]
    fn test_history_all() {
        let mut runtime = setup_test_runtime();
        let result = builtin_history(&[], &mut runtime).unwrap();
        assert!(result.stdout().contains("echo hello"));
        assert!(result.stdout().contains("cargo test"));
    }

    #[test]
    fn test_history_n() {
        let mut runtime = setup_test_runtime();
        let result = builtin_history(&["3".to_string()], &mut runtime).unwrap();
        let stdout = result.stdout();
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn test_history_search() {
        let mut runtime = setup_test_runtime();
        let result =
            builtin_history(&["search".to_string(), "cargo".to_string()], &mut runtime).unwrap();
        assert!(result.stdout().contains("cargo build"));
        assert!(result.stdout().contains("cargo test"));
    }

    #[test]
    fn test_history_search_no_results() {
        let mut runtime = setup_test_runtime();
        let result = builtin_history(
            &["search".to_string(), "nonexistent".to_string()],
            &mut runtime,
        )
        .unwrap();
        assert!(result.stdout().contains("No matching commands found"));
    }

    #[test]
    fn test_history_clear() {
        let mut runtime = setup_test_runtime();
        assert!(runtime.history().len() > 0);

        let result = builtin_history(&["clear".to_string()], &mut runtime).unwrap();
        assert!(result.stdout().contains("History cleared"));
        assert_eq!(runtime.history().len(), 0);
    }
}
