use aush::{executor::Executor, lexer::Lexer, parser::Parser};
use tempfile::TempDir;

fn run_result_in(
    dir: &std::path::Path,
    script: &str,
) -> anyhow::Result<aush::executor::ExecutionResult> {
    let tokens = Lexer::tokenize(script)?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(dir.to_path_buf());
    executor.execute(statements)
}

fn run_in(dir: &std::path::Path, script: &str) -> anyhow::Result<String> {
    Ok(run_result_in(dir, script)?.stdout())
}

#[test]
fn process_substitution_feeds_arguments_and_stdin_redirects() -> anyhow::Result<()> {
    let temp = TempDir::new()?;

    let arg_output = run_in(temp.path(), "cat <(echo hi)")?;
    assert_eq!(arg_output, "hi\n");

    let stdin_output = run_in(temp.path(), "cat < <(echo hi)")?;
    assert_eq!(stdin_output, "hi\n");

    Ok(())
}

#[test]
fn process_substitution_preserves_outer_variables() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output = run_in(
        temp.path(),
        "var=\"value\"\ncat <(var=\"updated\"; echo ${var})\necho \"Done.\"\necho \"${var}\"",
    )?;
    assert_eq!(output, "updated\nDone.\nvalue\n");
    Ok(())
}

#[test]
fn invalid_both_append_redirection_matches_shell_failure_status() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let result = run_result_in(
        temp.path(),
        "ls -d . non-existent-dir &>/dev/null\nls -d . non-existent-dir &>>/dev/null",
    )?;
    assert_eq!(result.exit_code, 2);
    assert_eq!(result.stdout(), "");
    assert!(result
        .stderr
        .contains("syntax error near unexpected token `>'"));
    Ok(())
}

#[test]
fn read_write_redirection_feeds_child_stdin() -> anyhow::Result<()> {
    let temp = TempDir::new()?;
    let output = run_in(temp.path(), "echo hi >file.txt\ncat <>file.txt")?;
    assert_eq!(output, "hi\n");
    Ok(())
}
