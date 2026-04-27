//! Command, function, redirection, subshell, and background execution.

use super::*;
use crate::ai::tools::confirm;
use crate::brand;
use crate::command_metadata::{metadata_for_command, CommandMetadata};
use crate::effects::RiskLevel;
use crate::receipts::{append_default_receipt_jsonl, ApprovalDecision, CommandReceipt};
use anyhow::{anyhow, Result};
use nix::unistd::{getpid, setpgid};
use std::io::IsTerminal;
use std::os::unix::process::CommandExt;
use std::process::Command as StdCommand;
use std::thread;
use std::time::{Duration, Instant};

fn format_command_preview(command_name: &str, args: &[String]) -> String {
    if args.is_empty() {
        command_name.to_string()
    } else {
        format!("{} {}", command_name, args.join(" "))
    }
}

fn approval_prompt(command_name: &str, args: &[String], metadata: &CommandMetadata) -> String {
    let rendered_command = format_command_preview(command_name, args);

    format!(
        "{}\n\nCommand: {}\nProceed?",
        metadata.render_human_summary(),
        rendered_command
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApprovalMode {
    Off,
    High,
    Medium,
}

impl ApprovalMode {
    fn from_env() -> Self {
        match brand::env_var("AUSH_APPROVAL_MODE") {
            Some(value) => parse_approval_mode(&value),
            None => Self::High,
        }
    }

    fn requires_confirmation(self, risk: RiskLevel) -> bool {
        match self {
            Self::Off => false,
            Self::High => risk >= RiskLevel::High,
            Self::Medium => risk >= RiskLevel::Medium,
        }
    }
}

fn parse_approval_mode(value: &str) -> ApprovalMode {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" | "never" | "false" | "0" => ApprovalMode::Off,
        "medium" => ApprovalMode::Medium,
        "high" | "on" | "true" | "1" | "" => ApprovalMode::High,
        _ => ApprovalMode::High,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_metadata::metadata_for_command;

    #[test]
    fn approval_prompt_uses_human_labels() {
        let metadata = metadata_for_command("rm").expect("rm metadata should exist");
        let prompt = approval_prompt("rm", &["old.log".to_string()], metadata);

        assert!(prompt.contains("Remove files or directories"));
        assert!(prompt.contains("High risk"));
        assert!(prompt.contains("Delete files"));
        assert!(prompt.contains("Command: rm old.log"));
        assert!(!prompt.contains("delete_file"));
    }

    #[test]
    fn confirmation_only_applies_to_high_risk_interactive_commands() {
        let executor = Executor::new_embedded();
        let rm = metadata_for_command("rm").expect("rm metadata should exist");
        let ls = metadata_for_command("ls").expect("ls metadata should exist");

        assert!(!executor.should_confirm_effects(rm));
        assert!(!executor.should_confirm_effects(ls));
    }

    #[test]
    fn approval_mode_parser_supports_off_high_and_medium() {
        assert_eq!(parse_approval_mode("off"), ApprovalMode::Off);
        assert_eq!(parse_approval_mode("0"), ApprovalMode::Off);
        assert_eq!(parse_approval_mode("medium"), ApprovalMode::Medium);
        assert_eq!(parse_approval_mode("high"), ApprovalMode::High);
        assert_eq!(parse_approval_mode("unexpected"), ApprovalMode::High);
    }

    #[test]
    fn approval_mode_thresholds_match_risk_levels() {
        assert!(!ApprovalMode::Off.requires_confirmation(RiskLevel::High));
        assert!(!ApprovalMode::High.requires_confirmation(RiskLevel::Medium));
        assert!(ApprovalMode::High.requires_confirmation(RiskLevel::High));
        assert!(ApprovalMode::Medium.requires_confirmation(RiskLevel::Medium));
        assert!(ApprovalMode::Medium.requires_confirmation(RiskLevel::High));
    }
}

impl Executor {
    pub(crate) fn execute_command(&mut self, command: Command) -> Result<ExecutionResult> {
        if self.runtime.options.xtrace {
            let args_str = command
                .args
                .iter()
                .map(|arg| match arg {
                    Argument::Literal(s)
                    | Argument::SingleQuoted(s)
                    | Argument::Variable(s)
                    | Argument::BracedVariable(s)
                    | Argument::CommandSubstitution(s)
                    | Argument::Flag(s)
                    | Argument::Path(s)
                    | Argument::Glob(s) => s.clone(),
                    Argument::DoubleQuoted(parts) => parts
                        .iter()
                        .map(|p| match p {
                            ArgumentPart::Literal(s) => s.clone(),
                            ArgumentPart::Variable(s)
                            | ArgumentPart::BracedVariable(s)
                            | ArgumentPart::CommandSubstitution(s) => s.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(""),
                })
                .collect::<Vec<_>>()
                .join(" ");
            if args_str.is_empty() {
                eprintln!("+ {}", command.name);
            } else {
                eprintln!("+ {} {}", command.name, args_str);
            }
        }

        let saved_env: Vec<(String, Option<String>)> = command
            .prefix_env
            .iter()
            .map(|(k, _)| (k.clone(), self.runtime.get_variable(k)))
            .collect();

        for (key, value) in &command.prefix_env {
            let expanded_value = self.expand_string_value(value)?;
            self.runtime
                .set_variable(key.clone(), expanded_value.clone());
            self.runtime.set_env(key, &expanded_value);
        }

        let (command_name, command_args) =
            if let Some(alias_value) = self.runtime.get_alias(&command.name) {
                let parts: Vec<&str> = alias_value.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(anyhow!("Empty alias expansion for '{}'", command.name));
                }

                let new_name = parts[0].to_string();
                let mut new_args = Vec::new();
                for part in parts.iter().skip(1) {
                    new_args.push(Argument::Literal(part.to_string()));
                }
                new_args.extend(command.args.clone());

                (new_name, new_args)
            } else {
                (command.name.clone(), command.args.clone())
            };

        if self.runtime.get_function(&command_name).is_some() {
            let args = self.expand_and_resolve_arguments(&command_args)?;
            if let Some(last) = args.last() {
                self.runtime.set_last_arg(last.clone());
            }
            let result = self.execute_user_function(&command_name, args);
            self.restore_prefix_env(&saved_env);
            return result;
        }

        let args = self.expand_and_resolve_arguments(&command_args)?;
        if let Some(last) = args.last() {
            self.runtime.set_last_arg(last.clone());
        }

        match self.maybe_confirm_effects(&command_name, &args)? {
            ApprovalDecision::Denied => {
                self.restore_prefix_env(&saved_env);
                return Ok(ExecutionResult::error(format!("Cancelled {}\n", command_name)));
            }
            ApprovalDecision::Approved | ApprovalDecision::NotRequired => {}
        }

        if self.builtins.is_builtin(&command_name) {
            let stdin_content = self.extract_stdin_content(&command.redirects)?;
            let piped_stdin = self.runtime.get_piped_stdin().map(|s| s.to_vec());

            let builtin_result_to_stderr =
                |res: Result<ExecutionResult>, cmd_name: &str| -> Result<ExecutionResult> {
                    match res {
                        Ok(r) => Ok(r),
                        Err(e) => {
                            if crate::executor::flow_signals::is_flow_control_signal(&e) {
                                return Err(e);
                            }
                            if matches!(cmd_name, "command" | "exec" | "local" | "shift") {
                                return Err(e);
                            }
                            Ok(ExecutionResult::error(format!("{}: {}\n", cmd_name, e)))
                        }
                    }
                };

            let mut result = if let Some(ref stdin_data) = stdin_content {
                builtin_result_to_stderr(
                    self.builtins.execute_with_stdin(
                        &command_name,
                        args,
                        &mut self.runtime,
                        Some(stdin_data.as_bytes()),
                    ),
                    &command_name,
                )?
            } else if let Some(ref piped_data) = piped_stdin {
                if command_name == "read" {
                    let line_end = piped_data
                        .iter()
                        .position(|&b| b == b'\n')
                        .map(|p| p + 1)
                        .unwrap_or(piped_data.len());

                    let result = builtin_result_to_stderr(
                        self.builtins.execute_with_stdin(
                            &command_name,
                            args,
                            &mut self.runtime,
                            Some(&piped_data[..line_end]),
                        ),
                        &command_name,
                    )?;

                    if line_end < piped_data.len() {
                        self.runtime
                            .set_piped_stdin(piped_data[line_end..].to_vec());
                    } else {
                        let _ = self.runtime.take_piped_stdin();
                    }

                    result
                } else {
                    builtin_result_to_stderr(
                        self.builtins.execute_with_stdin(
                            &command_name,
                            args,
                            &mut self.runtime,
                            Some(piped_data),
                        ),
                        &command_name,
                    )?
                }
            } else {
                builtin_result_to_stderr(
                    self.builtins
                        .execute(&command_name, args, &mut self.runtime),
                    &command_name,
                )?
            };

            if !command.redirects.is_empty() {
                result = self.apply_redirects(result, &command.redirects)?;
            }

            self.runtime.set_last_exit_code(result.exit_code);
            self.restore_prefix_env(&saved_env);
            return Ok(result);
        }

        let mut expanded_command = command;
        expanded_command.name = command_name;
        expanded_command.args = command_args;
        let result = self.execute_external_command(expanded_command)?;
        self.runtime.set_last_exit_code(result.exit_code);
        self.restore_prefix_env(&saved_env);
        Ok(result)
    }

    fn restore_prefix_env(&mut self, saved: &[(String, Option<String>)]) {
        for (key, old_value) in saved {
            match old_value {
                Some(val) => {
                    self.runtime.set_variable(key.clone(), val.clone());
                    self.runtime.set_env(key, val);
                }
                None => {
                    self.runtime.remove_variable(key);
                    std::env::remove_var(key);
                }
            }
        }
    }

    pub(crate) fn apply_redirects(
        &self,
        mut result: ExecutionResult,
        redirects: &[Redirect],
    ) -> Result<ExecutionResult> {
        use std::fs::{File, OpenOptions};
        use std::io::Write;
        use std::path::Path;

        let resolve_path = |target: &str| -> std::path::PathBuf {
            let path = Path::new(target);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.runtime.get_cwd().join(target)
            }
        };

        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::Stdout => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let mut file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        file.write_all(result.stdout().as_bytes())
                            .map_err(|e| anyhow!("Failed to write to '{}': {}", target, e))?;
                        result.clear_stdout();
                    }
                }
                RedirectKind::StdoutAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let mut file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&resolved)
                            .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?;
                        file.write_all(result.stdout().as_bytes())
                            .map_err(|e| anyhow!("Failed to write to '{}': {}", target, e))?;
                        result.clear_stdout();
                    }
                }
                RedirectKind::Stdin => {}
                RedirectKind::Stderr => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let mut file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        file.write_all(result.stderr.as_bytes())
                            .map_err(|e| anyhow!("Failed to write to '{}': {}", target, e))?;
                        result.stderr.clear();
                    }
                }
                RedirectKind::StderrToStdout => {
                    result.push_stdout(&result.stderr.clone());
                    result.stderr.clear();
                }
                RedirectKind::Both => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let mut file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        file.write_all(result.stdout().as_bytes())
                            .map_err(|e| anyhow!("Failed to write to '{}': {}", target, e))?;
                        file.write_all(result.stderr.as_bytes())
                            .map_err(|e| anyhow!("Failed to write to '{}': {}", target, e))?;
                        result.clear_stdout();
                        result.stderr.clear();
                    }
                }
                RedirectKind::HereDoc | RedirectKind::HereDocLiteral => {}
            }
        }

        Ok(result)
    }

    pub(crate) fn execute_user_function(
        &mut self,
        name: &str,
        args: Vec<String>,
    ) -> Result<ExecutionResult> {
        let func = self
            .runtime
            .get_function(name)
            .ok_or_else(|| anyhow!("Function '{}' not found", name))?
            .clone();

        self.runtime
            .push_call(name.to_string())
            .map_err(|e| anyhow!(e))?;
        self.call_stack.push(name.to_string());
        self.runtime.push_scope();

        for (i, param) in func.params.iter().enumerate() {
            let arg_value = args.get(i).cloned().unwrap_or_default();
            self.runtime.set_variable(param.name.clone(), arg_value);
        }

        self.runtime.push_positional_scope(args.clone());
        self.runtime.enter_function_context();

        let mut last_result = ExecutionResult::default();
        for statement in func.body {
            match self.execute_statement(statement) {
                Ok(stmt_result) => {
                    last_result.push_stdout(&stmt_result.stdout());
                    last_result.stderr.push_str(&stmt_result.stderr);
                    last_result.exit_code = stmt_result.exit_code;
                }
                Err(e) => {
                    if let Some(return_signal) =
                        e.downcast_ref::<crate::builtins::return_builtin::ReturnSignal>()
                    {
                        last_result.exit_code = return_signal.exit_code;
                        break;
                    } else {
                        self.runtime.exit_function_context();
                        self.runtime.pop_positional_scope();
                        self.runtime.pop_scope();
                        self.runtime.pop_call();
                        self.call_stack.pop();
                        return Err(e);
                    }
                }
            }
        }

        self.runtime.exit_function_context();
        self.runtime.pop_positional_scope();
        self.runtime.pop_scope();
        self.runtime.pop_call();
        self.call_stack.pop();

        Ok(last_result)
    }

    pub(crate) fn execute_external_command(&mut self, command: Command) -> Result<ExecutionResult> {
        let args = self.expand_and_resolve_arguments(&command.args)?;

        if let Some(last) = args.last() {
            self.runtime.set_last_arg(last.clone());
        }

        let mut cmd = StdCommand::new(&command.name);
        cmd.args(&args)
            .current_dir(self.runtime.get_cwd())
            .envs(self.runtime.get_env());

        use std::fs::{File, OpenOptions};
        use std::path::Path;
        use std::process::Stdio;

        let mut stdout_redirect = false;
        let mut stderr_redirect = false;
        let mut stderr_to_stdout = false;
        let mut stdin_redirect = false;

        let resolve_path = |target: &str| -> std::path::PathBuf {
            let path = Path::new(target);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.runtime.get_cwd().join(target)
            }
        };

        for redirect in &command.redirects {
            match &redirect.kind {
                RedirectKind::Stdout => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        cmd.stdout(Stdio::from(file));
                        stdout_redirect = true;
                    }
                }
                RedirectKind::StdoutAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&resolved)
                            .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?;
                        cmd.stdout(Stdio::from(file));
                        stdout_redirect = true;
                    }
                }
                RedirectKind::Stdin => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = File::open(&resolved)
                            .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?;
                        cmd.stdin(Stdio::from(file));
                        stdin_redirect = true;
                    }
                }
                RedirectKind::Stderr => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        cmd.stderr(Stdio::from(file));
                        stderr_redirect = true;
                    }
                }
                RedirectKind::StderrToStdout => {
                    stderr_to_stdout = true;
                }
                RedirectKind::Both => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        cmd.stdout(Stdio::from(
                            file.try_clone()
                                .map_err(|e| anyhow!("Failed to clone file descriptor: {}", e))?,
                        ));
                        cmd.stderr(Stdio::from(file));
                        stdout_redirect = true;
                        stderr_redirect = true;
                    }
                }
                RedirectKind::HereDoc | RedirectKind::HereDocLiteral => {
                    cmd.stdin(Stdio::piped());
                    stdin_redirect = true;
                }
            }
        }

        let heredoc_body: Option<String> = {
            let mut body = None;
            for redirect in &command.redirects {
                match &redirect.kind {
                    RedirectKind::HereDoc => {
                        if let Some(b) = &redirect.target {
                            body = Some(self.expand_heredoc_body(b)?);
                        }
                    }
                    RedirectKind::HereDocLiteral => {
                        if let Some(b) = &redirect.target {
                            body = Some(b.clone());
                        }
                    }
                    _ => {}
                }
            }
            body
        };

        if !stdin_redirect {
            cmd.stdin(Stdio::inherit());
        }

        let should_inherit_io = self.show_progress
            && !stdout_redirect
            && !stderr_redirect
            && command.redirects.is_empty()
            && std::io::stdout().is_terminal();

        if !stdout_redirect {
            if should_inherit_io {
                cmd.stdout(Stdio::inherit());
            } else {
                cmd.stdout(Stdio::piped());
            }
        }
        if !stderr_redirect && !stderr_to_stdout {
            if should_inherit_io {
                cmd.stderr(Stdio::inherit());
            } else {
                cmd.stderr(Stdio::piped());
            }
        } else if stderr_to_stdout && !stderr_redirect {
            cmd.stderr(Stdio::piped());
        }

        unsafe {
            cmd.pre_exec(|| {
                let pid = getpid();
                setpgid(pid, pid).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("setpgid failed: {}", e))
                })?;
                Ok(())
            });
        }

        let builtin_names: Vec<String> = self.builtins.builtin_names();
        let alias_names: Vec<String> = self.runtime.get_all_aliases().keys().cloned().collect();
        let history_commands: Vec<String> = self
            .runtime
            .history()
            .entries()
            .iter()
            .rev()
            .take(100)
            .map(|e| e.command.clone())
            .collect();
        let current_dir = self.runtime.get_cwd().to_path_buf();
        let command_name = command.name.clone();

        let mut child = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                let mut result = if e.kind() == std::io::ErrorKind::NotFound {
                    let suggestions = self.suggestion_engine.suggest_command(
                        &command_name,
                        &builtin_names,
                        &alias_names,
                        &history_commands,
                        &current_dir,
                    );

                    let mut error_msg = format!("Command not found: '{}'", command_name);

                    if !suggestions.is_empty() {
                        error_msg.push_str("\n\nDid you mean:\n");
                        for suggestion in suggestions.iter().take(3) {
                            error_msg.push_str(&format!("  {}\n", suggestion.text));
                        }
                    }

                    ExecutionResult {
                        output: Output::Text(String::new()),
                        stderr: format!("{}\n", error_msg),
                        exit_code: 127,
                        error: None,
                    }
                } else {
                    ExecutionResult {
                        output: Output::Text(String::new()),
                        stderr: format!("Failed to execute '{}': {}\n", command_name, e),
                        exit_code: 126,
                        error: None,
                    }
                };

                if !command.redirects.is_empty() {
                    result = self.apply_redirects(result, &command.redirects)?;
                }

                return Ok(result);
            }
        };

        if should_inherit_io {
            let child_pgid = nix::unistd::Pid::from_raw(child.id() as i32);
            let _ = self.terminal_control.give_terminal_to(child_pgid);
        }

        if let Some(body) = heredoc_body {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin
                    .write_all(body.as_bytes())
                    .map_err(|e| anyhow!("Failed to write here-document to stdin: {}", e))?;
                drop(stdin);
            }
        }

        let (stdout_str, stderr_str, exit_code) = if should_inherit_io {
            loop {
                if let Some(handler) = &self.signal_handler {
                    if handler.should_shutdown() {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(anyhow!("Command interrupted by signal"));
                    }
                }

                match child.try_wait() {
                    Ok(Some(status)) => {
                        break (String::new(), String::new(), status.code().unwrap_or(1));
                    }
                    Ok(None) => {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "Failed to check status for '{}': {}",
                            command.name,
                            e
                        ));
                    }
                }
            }
        } else {
            let output = child
                .wait_with_output()
                .map_err(|e| anyhow!("Failed to wait for '{}': {}", command.name, e))?;

            let mut stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();

            if stderr_to_stdout && !stderr_str.is_empty() {
                stdout_str.push_str(&stderr_str);
            }

            (
                stdout_str,
                if stderr_to_stdout {
                    String::new()
                } else {
                    stderr_str
                },
                output.status.code().unwrap_or(1),
            )
        };

        if should_inherit_io {
            let _ = self.terminal_control.reclaim_terminal();
        }

        Ok(ExecutionResult {
            output: Output::Text(stdout_str),
            stderr: stderr_str,
            exit_code,
            error: None,
        })
    }

    pub(crate) fn extract_stdin_content(
        &mut self,
        redirects: &[Redirect],
    ) -> Result<Option<String>> {
        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::HereDoc => {
                    if let Some(body) = &redirect.target {
                        return Ok(Some(self.expand_heredoc_body(body)?));
                    }
                }
                RedirectKind::HereDocLiteral => {
                    if let Some(body) = &redirect.target {
                        return Ok(Some(body.clone()));
                    }
                }
                RedirectKind::Stdin => {
                    if let Some(target) = &redirect.target {
                        let path = std::path::Path::new(target);
                        let resolved = if path.is_absolute() {
                            path.to_path_buf()
                        } else {
                            self.runtime.get_cwd().join(target)
                        };
                        let content = std::fs::read_to_string(&resolved)
                            .map_err(|e| anyhow!("Failed to read '{}': {}", target, e))?;
                        return Ok(Some(content));
                    }
                }
                _ => {}
            }
        }
        Ok(None)
    }

    pub(crate) fn execute_subshell(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<ExecutionResult> {
        let mut child_runtime = self.runtime.clone();

        let current_shlvl = child_runtime
            .get_variable("SHLVL")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(1);
        child_runtime.set_variable("SHLVL".to_string(), (current_shlvl + 1).to_string());

        let mut child_executor = Executor {
            runtime: child_runtime,
            builtins: self.builtins.clone(),
            corrector: self.corrector.clone(),
            suggestion_engine: self.suggestion_engine.clone(),
            signal_handler: None,
            show_progress: self.show_progress,
            terminal_control: self.terminal_control.clone(),
            call_stack: CallStack::new(),
            profile_data: None,
            enable_profiling: false,
        };

        let result = match child_executor.execute(statements) {
            Ok(r) => r,
            Err(e) => {
                if let Some(exit_sig) =
                    e.downcast_ref::<crate::builtins::exit_builtin::ExitSignal>()
                {
                    ExecutionResult {
                        output: Output::Text(String::new()),
                        stderr: String::new(),
                        exit_code: exit_sig.exit_code,
                        error: None,
                    }
                } else {
                    return Err(e);
                }
            }
        };

        Ok(result)
    }

    pub(crate) fn execute_brace_group(
        &mut self,
        statements: Vec<Statement>,
    ) -> Result<ExecutionResult> {
        self.execute(statements)
    }

    pub(crate) fn is_exec_command(statement: &Statement) -> bool {
        match statement {
            Statement::Command(cmd) => cmd.name == "exec",
            Statement::ConditionalAnd(cond) => Self::is_exec_command(&cond.right),
            Statement::ConditionalOr(cond) => Self::is_exec_command(&cond.right),
            _ => false,
        }
    }

    fn should_confirm_effects(&self, metadata: &CommandMetadata) -> bool {
        self.show_progress
            && !self.runtime.agent_mode()
            && ApprovalMode::from_env().requires_confirmation(metadata.risk)
            && std::io::stdin().is_terminal()
            && std::io::stdout().is_terminal()
    }

    fn maybe_confirm_effects(&self, command_name: &str, args: &[String]) -> Result<ApprovalDecision> {
        let Some(metadata) = metadata_for_command(command_name) else {
            return Ok(ApprovalDecision::NotRequired);
        };

        if !self.should_confirm_effects(metadata) {
            return Ok(ApprovalDecision::NotRequired);
        }

        let prompt = approval_prompt(command_name, args, metadata);
        let started_at = chrono::Utc::now();
        let timer = Instant::now();
        let approved = confirm(&prompt)?;
        let finished_at = chrono::Utc::now();
        let decision = if approved {
            ApprovalDecision::Approved
        } else {
            ApprovalDecision::Denied
        };

        self.record_approval_receipt(
            command_name,
            args,
            started_at,
            finished_at,
            timer.elapsed(),
            decision,
        );

        Ok(decision)
    }

    fn record_approval_receipt(
        &self,
        command_name: &str,
        args: &[String],
        started_at: chrono::DateTime<chrono::Utc>,
        finished_at: chrono::DateTime<chrono::Utc>,
        elapsed: Duration,
        approval: ApprovalDecision,
    ) {
        let exit_code = match approval {
            ApprovalDecision::Denied => 1,
            ApprovalDecision::Approved | ApprovalDecision::NotRequired => 0,
        };
        let receipt = CommandReceipt::new(
            format_command_preview(command_name, args),
            self.runtime.get_cwd().clone(),
            started_at,
            finished_at,
            exit_code,
        )
        .with_approval(approval);

        if let Err(error) = append_default_receipt_jsonl(&receipt) {
            eprintln!(
                "aush: warning: failed to write approval receipt after {} ms: {}",
                elapsed.as_millis(),
                error
            );
        }
    }

    pub(crate) fn execute_background(&mut self, statement: Statement) -> Result<ExecutionResult> {
        use std::process::Stdio;

        let command_str = self.statement_to_string(&statement);

        match statement {
            Statement::Command(command) => {
                if self.builtins.is_builtin(&command.name) {
                    return Err(anyhow!("Builtin commands cannot be run in background"));
                }

                let args: Result<Vec<String>> = command
                    .args
                    .iter()
                    .map(|arg| self.resolve_argument(arg))
                    .collect();

                let args = args?;

                let mut cmd = StdCommand::new(&command.name);
                cmd.args(&args)
                    .current_dir(self.runtime.get_cwd())
                    .envs(self.runtime.get_env())
                    .stdin(Stdio::null())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());

                unsafe {
                    cmd.pre_exec(|| {
                        let pid = getpid();
                        setpgid(pid, pid).map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("setpgid failed: {}", e),
                            )
                        })?;
                        Ok(())
                    });
                }

                let child = cmd.spawn().map_err(|e| {
                    anyhow!(
                        "Failed to spawn background process '{}': {}",
                        command.name,
                        e
                    )
                })?;

                let pid = child.id();
                let job_id = self.runtime.job_manager().add_job(pid, command_str);
                self.runtime.set_last_bg_pid(pid);

                Ok(ExecutionResult::success(format!("[{}] {}\n", job_id, pid)))
            }
            Statement::Pipeline(_) | Statement::Subshell(_) => {
                self.execute_background_via_sh(&command_str)
            }
            _ => Err(anyhow!(
                "Only simple commands and pipelines can be run in background"
            )),
        }
    }

    pub(crate) fn execute_background_via_sh(
        &mut self,
        command_str: &str,
    ) -> Result<ExecutionResult> {
        use std::process::{Command as StdCommand, Stdio};

        let mut cmd = StdCommand::new("sh");
        cmd.arg("-c")
            .arg(command_str)
            .current_dir(self.runtime.get_cwd())
            .envs(self.runtime.get_env())
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        unsafe {
            cmd.pre_exec(|| {
                let pid = getpid();
                setpgid(pid, pid).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("setpgid failed: {}", e))
                })?;
                Ok(())
            });
        }

        let child = cmd
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn background process: {}", e))?;

        let pid = child.id();
        let job_id = self
            .runtime
            .job_manager()
            .add_job(pid, command_str.to_string());
        self.runtime.set_last_bg_pid(pid);

        Ok(ExecutionResult::success(format!("[{}] {}\n", job_id, pid)))
    }
}
