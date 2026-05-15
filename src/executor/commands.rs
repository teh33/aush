//! Command, function, redirection, subshell, and background execution.

use super::*;
use crate::ai::tools::confirm;
use crate::brand;
use crate::command_metadata::{metadata_for_command, CommandMetadata};
use crate::effects::RiskLevel;
use crate::receipts::{append_default_receipt_jsonl, ApprovalDecision, CommandReceipt};
use crate::runtime::PermanentFd;
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

enum StreamTarget {
    Capture,
    File(std::path::PathBuf, bool),
    RawFd(i32),
    Closed,
}

fn clone_stream_target(target: Option<&StreamTarget>) -> StreamTarget {
    match target {
        Some(StreamTarget::File(path, append)) => StreamTarget::File(path.clone(), *append),
        Some(StreamTarget::RawFd(fd)) => StreamTarget::RawFd(*fd),
        Some(StreamTarget::Closed) => StreamTarget::Closed,
        Some(StreamTarget::Capture) | None => StreamTarget::Capture,
    }
}

fn move_captured_stream(result: &mut ExecutionResult, from_fd: u32, to_fd: u32) {
    match (from_fd, to_fd) {
        (1, 2) => {
            result.stderr.push_str(&result.stdout());
            result.clear_stdout();
        }
        (2, 1) => {
            result.push_stdout(&result.stderr.clone());
            result.stderr.clear();
        }
        _ => {}
    }
}

fn process_substitution_output_path(command: &str) -> Result<String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let path = std::env::temp_dir().join(format!(
        "aush-process-subst-out-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    Ok(format!("{}::{}", path.to_string_lossy(), command))
}

pub(crate) fn process_substitution_argument_path(command: &str, output: bool) -> Result<String> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let prefix = if output {
        "aush-process-subst-out"
    } else {
        "aush-process-subst"
    };
    let path = std::env::temp_dir().join(format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    let marker = format!(
        "__AUSH_PROCESS_SUBST_ARG__{}::{}",
        path.to_string_lossy().replace('/', "~s"),
        command
    );
    Ok(marker)
}

pub(crate) fn split_process_substitution_argument(target: &str) -> Option<(String, String)> {
    target
        .strip_prefix("__AUSH_PROCESS_SUBST_ARG__")
        .and_then(|rest| rest.split_once("::"))
        .map(|(path, cmd)| (path.replace("~s", "/"), cmd.to_string()))
}

pub(crate) fn materialize_process_substitution_argument(target: &str) -> Option<String> {
    let (path, command) = split_process_substitution_argument(target)?;
    if path.contains("process-subst-out") {
        return Some(path);
    }
    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .output()
        .ok()?;
    std::fs::write(&path, output.stdout).ok()?;
    Some(path)
}

fn split_process_substitution_output(target: &str) -> Option<(&str, &str)> {
    target
        .strip_prefix("__AUSH_PROCESS_SUBST_OUT__")
        .and_then(|rest| rest.split_once("::"))
}

fn finalize_process_substitution_outputs(
    redirects: &[Redirect],
    args: &[Argument],
    runtime: &crate::runtime::Runtime,
) -> Result<()> {
    let mut targets: Vec<String> = redirects
        .iter()
        .filter_map(|redirect| redirect.target.clone())
        .collect();
    targets.extend(args.iter().filter_map(|arg| match arg {
        Argument::Literal(value) => Some(value.clone()),
        _ => None,
    }));

    for raw_target in targets {
        let output = if let Some((path, command)) = split_process_substitution_output(&raw_target) {
            Some((path.to_string(), command.to_string()))
        } else if let Some((path, command)) = split_process_substitution_argument(&raw_target) {
            if path.contains("process-subst-out") {
                Some((path, command))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((path, command)) = output {
            let input = std::fs::File::open(&path)?;
            let result = std::process::Command::new("/bin/sh")
                .arg("-c")
                .arg(command)
                .current_dir(runtime.get_cwd())
                .stdin(std::process::Stdio::from(input))
                .output()?;
            let stderr = String::from_utf8_lossy(&result.stderr);
            if !result.status.success() || !stderr.is_empty() {
                eprint!("{}", stderr);
            }
            if result.status.success() {
                print!("{}", String::from_utf8_lossy(&result.stdout));
            }
        }
    }
    Ok(())
}

fn open_output_fd(path: &std::path::Path, append: bool) -> Result<i32> {
    use std::fs::OpenOptions;
    use std::os::fd::IntoRawFd;

    let file = if append {
        OpenOptions::new().create(true).append(true).open(path)?
    } else {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?
    };
    Ok(file.into_raw_fd())
}

fn apply_stream_target(
    fd: u32,
    result: &mut ExecutionResult,
    target: Option<&StreamTarget>,
) -> Result<()> {
    use std::fs::{File, OpenOptions};
    use std::io::Write;

    let content = match fd {
        1 => result.stdout(),
        2 => result.stderr.clone(),
        _ => String::new(),
    };

    match target.unwrap_or(&StreamTarget::Capture) {
        StreamTarget::Capture => {}
        StreamTarget::Closed => match fd {
            2 => result.stderr.clear(),
            _ => {}
        },
        StreamTarget::RawFd(raw_fd) => {
            if !content.is_empty() {
                let bytes = content.as_bytes();
                let written = unsafe {
                    libc::write(*raw_fd, bytes.as_ptr().cast::<libc::c_void>(), bytes.len())
                };
                if written < 0 {
                    return Err(anyhow!("Failed to write to fd {}", raw_fd));
                }
            }
            match fd {
                1 => result.clear_stdout(),
                2 => result.stderr.clear(),
                _ => {}
            }
        }
        StreamTarget::File(path, append) => {
            let mut file = if *append {
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .map_err(|e| anyhow!("Failed to open '{}': {}", path.display(), e))?
            } else {
                File::create(path)
                    .map_err(|e| anyhow!("Failed to create '{}': {}", path.display(), e))?
            };
            file.write_all(content.as_bytes())
                .map_err(|e| anyhow!("Failed to write to '{}': {}", path.display(), e))?;
            match fd {
                1 => result.clear_stdout(),
                2 => result.stderr.clear(),
                _ => {}
            }
        }
    }

    Ok(())
}

fn process_substitution_fd_path(fd: i32) -> String {
    #[cfg(target_os = "macos")]
    {
        format!("/dev/fd/{}", fd)
    }
    #[cfg(not(target_os = "macos"))]
    {
        format!("/proc/self/fd/{}", fd)
    }
}

impl Executor {
    pub(crate) fn execute_command(&mut self, mut command: Command) -> Result<ExecutionResult> {
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
            let expanded_value = self.expand_assignment_value(value)?;
            self.runtime
                .set_variable(key.clone(), expanded_value.clone());
            self.runtime.set_env(key, &expanded_value);
        }

        let raw_command_name = self.expand_command_name(&command.name)?;
        if raw_command_name.is_empty() {
            self.restore_prefix_env(&saved_env);
            return Ok(ExecutionResult::success(String::new()));
        }
        let command_args = self.expand_command_name_args(&command.name, command.args.clone());
        self.prepare_command_substitution_redirects(&command.redirects)?;

        let (command_name, command_args) =
            if let Some(alias_value) = self.runtime.get_alias(&raw_command_name) {
                let parts: Vec<&str> = alias_value.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(anyhow!("Empty alias expansion for '{}'", raw_command_name));
                }

                let new_name = parts[0].to_string();
                let mut new_args = Vec::new();
                for part in parts.iter().skip(1) {
                    new_args.push(Argument::Literal(part.to_string()));
                }
                new_args.extend(command_args);

                (new_name, new_args)
            } else {
                (raw_command_name, command_args)
            };

        if self.runtime.get_function(&command_name).is_some() {
            let args = self.expand_and_resolve_arguments(&command_args)?;
            if let Some(last) = args.last() {
                self.runtime.set_last_arg(last.clone());
            }
            let result = self.execute_user_function(&command_name, args)?;
            let result = if command.redirects.is_empty() {
                result
            } else {
                self.apply_redirects(result, &command.redirects)?
            };
            self.runtime.set_last_exit_code(result.exit_code);
            self.restore_prefix_env(&saved_env);
            return Ok(result);
        }

        let args = self.expand_and_resolve_arguments(&command_args)?;
        if let Some(last) = args.last() {
            self.runtime.set_last_arg(last.clone());
        }

        match self.maybe_confirm_effects(&command_name, &args)? {
            ApprovalDecision::Denied => {
                self.restore_prefix_env(&saved_env);
                return Ok(ExecutionResult::error(format!(
                    "Cancelled {}\n",
                    command_name
                )));
            }
            ApprovalDecision::Approved | ApprovalDecision::NotRequired => {}
        }

        if self.builtins.is_builtin(&command_name) {
            if command_name == "exec" && args.is_empty() && !command.redirects.is_empty() {
                self.apply_permanent_exec_redirects(&command.redirects)?;
                self.runtime.set_last_exit_code(0);
                self.restore_prefix_env(&saved_env);
                return Ok(ExecutionResult::success(String::new()));
            }

            if command_name == "cat" {
                if let Some(fd) = self.extract_process_substitution_stdin(&command.redirects)? {
                    let saved_redirects = std::mem::take(&mut command.redirects);
                    let mut result =
                        crate::builtins::cat::builtin_cat_with_fd(&[], &mut self.runtime, fd)?;
                    command.redirects = saved_redirects;
                    self.runtime.set_last_exit_code(result.exit_code);
                    self.restore_prefix_env(&saved_env);
                    if !command.redirects.is_empty() {
                        result = self.apply_redirects(result, &command.redirects)?;
                    }
                    return Ok(result);
                }
            }
            if command_name == "source" || command_name == "." {
                let has_process_substitution_arg = command_args.first().is_some_and(|arg| {
                    matches!(arg, Argument::Literal(value) if value.starts_with("__AUSH_PROCESS_SUBST_ARG__"))
                });
                if has_process_substitution_arg {
                    self.runtime.set_last_exit_code(0);
                    self.restore_prefix_env(&saved_env);
                    return Ok(ExecutionResult::success(String::new()));
                }
                if let Some(_fd) = self.extract_process_substitution_stdin(&command.redirects)? {
                    self.runtime.set_last_exit_code(0);
                    self.restore_prefix_env(&saved_env);
                    return Ok(ExecutionResult::success(String::new()));
                }
            }

            let stdin_content = self.extract_stdin_content(&command.redirects)?;
            let stdin_fd = self.extract_stdin_fd(&command.redirects)?;
            let read_write_stdin_fd = self.extract_read_write_stdin_fd(&command.redirects)?;
            let piped_stdin = self.runtime.get_piped_stdin().map(|s| s.to_vec());

            let builtin_result_to_stderr =
                |res: Result<ExecutionResult>, cmd_name: &str| -> Result<ExecutionResult> {
                    match res {
                        Ok(r) => Ok(r),
                        Err(e) => {
                            if let Some(exit_sig) =
                                e.downcast_ref::<crate::builtins::exit_builtin::ExitSignal>()
                            {
                                let mut result = ExecutionResult::success(String::new());
                                result.exit_code = exit_sig.exit_code;
                                return Ok(result);
                            }
                            if crate::executor::flow_signals::is_flow_control_signal(&e) {
                                return Err(e);
                            }
                            if matches!(
                                cmd_name,
                                "break" | "continue" | "command" | "exec" | "local" | "shift"
                            ) {
                                return Err(e);
                            }
                            Ok(ExecutionResult::error(format!("{}: {}\n", cmd_name, e)))
                        }
                    }
                };

            let has_process_substitution_stdin = command.redirects.iter().any(|redirect| {
                matches!(redirect.kind, RedirectKind::Stdin)
                    && redirect
                        .target
                        .as_deref()
                        .is_some_and(|target| target.starts_with("__AUSH_PROCESS_SUBST__"))
            });
            let process_substitution_stdin =
                self.extract_process_substitution_stdin(&command.redirects)?;
            let effective_stdin_fd = process_substitution_stdin
                .or(stdin_fd)
                .or(read_write_stdin_fd);
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
            } else if let Some(fd) = effective_stdin_fd {
                if fd < 0 {
                    ExecutionResult::error("bad file descriptor".to_string())
                } else {
                    let previous_stdin = self.runtime.get_permanent_stdin();
                    self.runtime.set_permanent_stdin(Some(fd));
                    let result = if command_name == "cat" && has_process_substitution_stdin {
                        crate::builtins::cat::builtin_cat_with_fd(&[], &mut self.runtime, fd)
                    } else if command_name == "cat" {
                        crate::builtins::cat::builtin_cat_with_fd(&args, &mut self.runtime, fd)
                    } else {
                        builtin_result_to_stderr(
                            self.builtins.execute_with_stdin(
                                &command_name,
                                args,
                                &mut self.runtime,
                                None,
                            ),
                            &command_name,
                        )
                    };
                    self.runtime.set_permanent_stdin(previous_stdin);
                    result?
                }
            } else if command_name == "cat" && has_process_substitution_stdin {
                builtin_result_to_stderr(
                    self.builtins
                        .execute_with_stdin(&command_name, args, &mut self.runtime, None),
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
                let has_process_substitution_redirect = command.redirects.iter().any(|redirect| {
                    matches!(redirect.kind, RedirectKind::Stdin)
                        && redirect
                            .target
                            .as_deref()
                            .is_some_and(|target| target.starts_with("__AUSH_PROCESS_SUBST__"))
                });
                if !(command_name == "cat" && has_process_substitution_redirect) {
                    result = self.apply_redirects(result, &command.redirects)?;
                }
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

    fn prepare_command_substitution_redirects(&mut self, redirects: &[Redirect]) -> Result<()> {
        use std::fs::OpenOptions;
        use std::os::fd::IntoRawFd;
        use std::path::Path;

        let cwd = self.runtime.get_cwd().to_path_buf();
        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::FdOut(fd) if *fd > 2 => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let path = Path::new(&target);
                        let resolved = if path.is_absolute() {
                            path.to_path_buf()
                        } else {
                            cwd.join(path)
                        };
                        let file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(resolved)?;
                        self.runtime
                            .set_permanent_fd(*fd as i32, Some(file.into_raw_fd()));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn apply_permanent_exec_redirects(&mut self, redirects: &[Redirect]) -> Result<()> {
        use std::fs::{File, OpenOptions};
        use std::os::fd::IntoRawFd;
        use std::path::Path;

        let cwd = self.runtime.get_cwd().to_path_buf();
        let resolve_path = |target: &str| -> std::path::PathBuf {
            let path = Path::new(target);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                cwd.join(target)
            }
        };

        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::Stdout | RedirectKind::StdoutAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let file = open_output_fd(
                            &resolve_path(&target),
                            matches!(redirect.kind, RedirectKind::StdoutAppend),
                        )?;
                        self.runtime.set_permanent_stdout(Some(file));
                    }
                }
                RedirectKind::Stderr => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let file = open_output_fd(&resolve_path(&target), false)?;
                        self.runtime.set_permanent_stderr(Some(file));
                    }
                }
                RedirectKind::Stdin | RedirectKind::ReadWrite => {
                    if let Some(raw_target) = &redirect.target {
                        let target = if let Some((path, _)) =
                            crate::executor::commands::split_process_substitution_argument(
                                raw_target,
                            ) {
                            path.to_string()
                        } else if let Some(command) =
                            raw_target.strip_prefix("__AUSH_PROCESS_SUBST__")
                        {
                            self.materialize_input_process_substitution(command)?
                        } else {
                            expand_redirect_target(raw_target, &self.runtime)
                        };
                        let file = if matches!(redirect.kind, RedirectKind::ReadWrite) {
                            OpenOptions::new()
                                .read(true)
                                .write(true)
                                .open(resolve_path(&target))?
                        } else {
                            File::open(resolve_path(&target))?
                        };
                        self.runtime.set_permanent_stdin(Some(file.into_raw_fd()));
                    }
                }
                RedirectKind::FdOut(fd) => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let file = open_output_fd(&resolve_path(&target), false)?;
                        self.runtime.set_permanent_fd(*fd as i32, Some(file));
                    }
                }
                RedirectKind::FdIn(fd) => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let file = File::open(resolve_path(&target))?;
                        self.runtime
                            .set_permanent_fd(*fd as i32, Some(file.into_raw_fd()));
                    }
                }
                RedirectKind::StderrToStdout => {
                    let target = self.runtime.get_permanent_stdout().unwrap_or(1);
                    self.runtime.set_permanent_stderr(Some(target));
                }
                RedirectKind::StdoutToStderr => {
                    let target = self.runtime.get_permanent_stderr().unwrap_or(2);
                    self.runtime.set_permanent_stdout(Some(target));
                }
                RedirectKind::StdoutToFd(fd) => {
                    let target = match self.runtime.permanent_fds().get(&(*fd as i32)) {
                        Some(PermanentFd::Open(raw_fd)) => *raw_fd,
                        Some(PermanentFd::Closed) => {
                            return Err(anyhow!("bad file descriptor"));
                        }
                        None => *fd as i32,
                    };
                    self.runtime.set_permanent_stdout(Some(target));
                }
                RedirectKind::FdInputFrom(from, to) => {
                    let target = match self.runtime.permanent_fds().get(&(*to as i32)) {
                        Some(PermanentFd::Open(raw_fd)) => *raw_fd,
                        Some(PermanentFd::Closed) => {
                            return Err(anyhow!("bad file descriptor"));
                        }
                        None => *to as i32,
                    };
                    let dup = unsafe { libc::dup(target) };
                    if dup < 0 {
                        return Err(anyhow!("bad file descriptor"));
                    }
                    if *from == 0 {
                        self.runtime.set_permanent_stdin(Some(dup));
                    } else {
                        self.runtime.set_permanent_fd(*from as i32, Some(dup));
                    }
                }
                RedirectKind::CloseFd(fd) => {
                    self.runtime.close_permanent_fd(*fd as i32);
                }
                RedirectKind::Invalid(message) => {
                    return Err(anyhow!(message.clone()));
                }
                RedirectKind::Both | RedirectKind::BothAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let file = open_output_fd(
                            &resolve_path(&target),
                            matches!(redirect.kind, RedirectKind::BothAppend),
                        )?;
                        let dup = unsafe { libc::dup(file) };
                        if dup < 0 {
                            return Err(anyhow!("Failed to duplicate fd {}", file));
                        }
                        self.runtime.set_permanent_stdout(Some(file));
                        self.runtime.set_permanent_stderr(Some(dup));
                    }
                }
                RedirectKind::ProcessSubstitutionInputArg
                | RedirectKind::ProcessSubstitutionOutputArg => {}
                RedirectKind::HereDoc | RedirectKind::HereDocLiteral => {}
            }
        }

        Ok(())
    }

    pub(crate) fn apply_redirects(
        &self,
        mut result: ExecutionResult,
        redirects: &[Redirect],
    ) -> Result<ExecutionResult> {
        use std::collections::BTreeMap;
        use std::fs::{File, OpenOptions};
        use std::io::Write;
        use std::path::Path;

        let cwd = self.runtime.get_cwd().to_path_buf();
        let resolve_path = |target: &str| -> std::path::PathBuf {
            let path = Path::new(target);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                cwd.join(target)
            }
        };

        let mut fd_targets: BTreeMap<u32, StreamTarget> = BTreeMap::new();
        fd_targets.insert(1, StreamTarget::Capture);
        fd_targets.insert(2, StreamTarget::Capture);
        for (fd, permanent_fd) in self.runtime.permanent_fds() {
            match permanent_fd {
                PermanentFd::Open(raw_fd) => {
                    fd_targets.insert(*fd as u32, StreamTarget::RawFd(*raw_fd));
                }
                PermanentFd::Closed => {
                    fd_targets.insert(*fd as u32, StreamTarget::Closed);
                }
            }
        }

        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::Stdout => {
                    if let Some(raw_target) = &redirect.target {
                        let target = if let Some((path, _)) =
                            split_process_substitution_output(raw_target)
                        {
                            path.to_string()
                        } else if let Some(path) =
                            materialize_process_substitution_argument(raw_target)
                        {
                            path
                        } else if let Some(command) =
                            raw_target.strip_prefix("__AUSH_PROCESS_SUBST_OUT__")
                        {
                            process_substitution_output_path(command)?
                        } else {
                            expand_redirect_target(raw_target, &self.runtime)
                        };
                        fd_targets.insert(1, StreamTarget::File(resolve_path(&target), false));
                    }
                }
                RedirectKind::StdoutAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        fd_targets.insert(1, StreamTarget::File(resolve_path(&target), true));
                    }
                }
                RedirectKind::Stdin => {
                    if let Some(raw_target) = &redirect.target {
                        if raw_target.starts_with("__AUSH_PROCESS_SUBST__") {
                            continue;
                        }
                        if let Some((path, _)) =
                            crate::executor::commands::split_process_substitution_argument(
                                raw_target,
                            )
                        {
                            fd_targets.insert(
                                0,
                                StreamTarget::File(std::path::PathBuf::from(path), false),
                            );
                            continue;
                        }
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        fd_targets.insert(0, StreamTarget::File(resolve_path(&target), false));
                    }
                }
                RedirectKind::Stderr => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        fd_targets.insert(2, StreamTarget::File(resolve_path(&target), false));
                    }
                }
                RedirectKind::FdOut(fd) => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        fd_targets.insert(*fd, StreamTarget::File(resolve_path(&target), false));
                    }
                }
                RedirectKind::FdIn(_) => {}
                RedirectKind::ReadWrite => {
                    if let Some(raw_target) = &redirect.target {
                        let target = if let Some((path, _)) =
                            crate::executor::commands::split_process_substitution_argument(
                                raw_target,
                            ) {
                            path.to_string()
                        } else if let Some(command) =
                            raw_target.strip_prefix("__AUSH_PROCESS_SUBST__")
                        {
                            self.materialize_input_process_substitution(command)?
                        } else {
                            expand_redirect_target(raw_target, &self.runtime)
                        };
                        fd_targets.insert(0, StreamTarget::File(resolve_path(&target), false));
                    }
                }
                RedirectKind::StderrToStdout => {
                    let stdout_target = clone_stream_target(fd_targets.get(&1));
                    if matches!(stdout_target, StreamTarget::Capture) {
                        move_captured_stream(&mut result, 2, 1);
                    }
                    fd_targets.insert(2, stdout_target);
                }
                RedirectKind::StdoutToStderr => {
                    let stderr_target = clone_stream_target(fd_targets.get(&2));
                    if matches!(stderr_target, StreamTarget::Capture) {
                        move_captured_stream(&mut result, 1, 2);
                    }
                    fd_targets.insert(1, stderr_target);
                }
                RedirectKind::StdoutToFd(fd) => {
                    let target = clone_stream_target(fd_targets.get(fd));
                    if matches!(target, StreamTarget::Capture) {
                        move_captured_stream(&mut result, 1, *fd);
                    } else if matches!(target, StreamTarget::Closed) {
                        return Ok(ExecutionResult::error("bad file descriptor".to_string()));
                    }
                    fd_targets.insert(1, target);
                }
                RedirectKind::FdInputFrom(from, to) => {
                    if *from == 0 {
                        let target = clone_stream_target(fd_targets.get(to));
                        if matches!(target, StreamTarget::Closed) {
                            return Ok(ExecutionResult::error("bad file descriptor".to_string()));
                        }
                        fd_targets.insert(0, target);
                    }
                }
                RedirectKind::CloseFd(fd) => {
                    fd_targets.insert(*fd, StreamTarget::Closed);
                }
                RedirectKind::Invalid(message) => {
                    let mut result = ExecutionResult::error(message.clone());
                    if message.contains("syntax error near unexpected token") {
                        result.exit_code = 2;
                    }
                    return Ok(result);
                }
                RedirectKind::Both | RedirectKind::BothAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let file_target = StreamTarget::File(
                            resolve_path(&target),
                            matches!(redirect.kind, RedirectKind::BothAppend),
                        );
                        fd_targets.insert(1, clone_stream_target(Some(&file_target)));
                        fd_targets.insert(2, file_target);
                    }
                }
                RedirectKind::ProcessSubstitutionInputArg => {
                    if let Some(raw_target) = &redirect.target {
                        if let Some(path) = materialize_process_substitution_argument(raw_target) {
                            fd_targets.insert(
                                0,
                                StreamTarget::File(std::path::PathBuf::from(path), false),
                            );
                        }
                    }
                }
                RedirectKind::ProcessSubstitutionOutputArg => {
                    if let Some(raw_target) = &redirect.target {
                        if let Some((path, _)) = split_process_substitution_argument(raw_target) {
                            fd_targets.insert(
                                1,
                                StreamTarget::File(std::path::PathBuf::from(path), false),
                            );
                        }
                    }
                }
                RedirectKind::HereDoc | RedirectKind::HereDocLiteral => {}
            }
        }

        if let (
            Some(StreamTarget::File(stdout_path, stdout_append)),
            Some(StreamTarget::File(stderr_path, stderr_append)),
        ) = (fd_targets.get(&1), fd_targets.get(&2))
        {
            if stdout_path == stderr_path && stdout_append == stderr_append {
                let mut file = if *stdout_append {
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(stdout_path)
                        .map_err(|e| anyhow!("Failed to open '{}': {}", stdout_path.display(), e))?
                } else {
                    File::create(stdout_path).map_err(|e| {
                        anyhow!("Failed to create '{}': {}", stdout_path.display(), e)
                    })?
                };
                file.write_all(result.stdout().as_bytes()).map_err(|e| {
                    anyhow!("Failed to write to '{}': {}", stdout_path.display(), e)
                })?;
                file.write_all(result.stderr.as_bytes()).map_err(|e| {
                    anyhow!("Failed to write to '{}': {}", stdout_path.display(), e)
                })?;
                result.clear_stdout();
                result.stderr.clear();
                return Ok(result);
            }
        }

        apply_stream_target(1, &mut result, fd_targets.get(&1))?;
        apply_stream_target(2, &mut result, fd_targets.get(&2))?;
        finalize_process_substitution_outputs(redirects, &[], &self.runtime)?;

        Ok(result)
    }

    fn expand_command_name(&self, name: &str) -> Result<String> {
        if name == "$@" || name == "${@}" {
            return Ok(self
                .runtime
                .get_positional_params()
                .first()
                .cloned()
                .unwrap_or_default());
        }

        self.expand_string_value(name)
    }

    fn expand_command_name_args(&self, name: &str, args: Vec<Argument>) -> Vec<Argument> {
        if name == "$@" || name == "${@}" {
            return self
                .runtime
                .get_positional_params()
                .iter()
                .skip(1)
                .cloned()
                .map(Argument::Literal)
                .chain(args)
                .collect();
        }

        args
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

        let cwd = self.runtime.get_cwd().to_path_buf();
        let resolve_path = |target: &str| -> std::path::PathBuf {
            let path = Path::new(target);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                cwd.join(target)
            }
        };

        for redirect in &command.redirects {
            match &redirect.kind {
                RedirectKind::Stdout => {
                    if let Some(raw_target) = &redirect.target {
                        let target = if let Some(command) =
                            raw_target.strip_prefix("__AUSH_PROCESS_SUBST_OUT__")
                        {
                            process_substitution_output_path(command)?
                        } else {
                            expand_redirect_target(raw_target, &self.runtime)
                        };
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
                        let target = if let Some(command) =
                            raw_target.strip_prefix("__AUSH_PROCESS_SUBST__")
                        {
                            self.materialize_input_process_substitution(command)?
                        } else {
                            expand_redirect_target(raw_target, &self.runtime)
                        };
                        let resolved = resolve_path(&target);
                        let file = File::open(&resolved)
                            .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?;
                        cmd.stdin(Stdio::from(file));
                        stdin_redirect = true;
                    }
                }
                RedirectKind::FdIn(fd) => {
                    if *fd == 0 {
                        if let Some(raw_target) = &redirect.target {
                            let target = expand_redirect_target(raw_target, &self.runtime);
                            let resolved = resolve_path(&target);
                            let file = File::open(&resolved)
                                .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?;
                            cmd.stdin(Stdio::from(file));
                            stdin_redirect = true;
                        }
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
                RedirectKind::FdOut(fd) => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = File::create(&resolved)
                            .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?;
                        if *fd == 2 {
                            cmd.stderr(Stdio::from(file));
                            stderr_redirect = true;
                        }
                    }
                }
                RedirectKind::ReadWrite => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(&resolved)
                            .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?;
                        cmd.stdin(Stdio::from(file));
                        stdin_redirect = true;
                    }
                }
                RedirectKind::StderrToStdout => {
                    stderr_to_stdout = true;
                }
                RedirectKind::StdoutToStderr => {
                    cmd.stdout(Stdio::piped());
                    stdout_redirect = true;
                }
                RedirectKind::StdoutToFd(fd) => {
                    if *fd == 2 {
                        cmd.stdout(Stdio::piped());
                        stdout_redirect = true;
                    }
                }
                RedirectKind::FdInputFrom(_, _) => {}
                RedirectKind::CloseFd(fd) => match *fd {
                    0 => {
                        cmd.stdin(Stdio::null());
                        stdin_redirect = true;
                    }
                    1 => {
                        cmd.stdout(Stdio::null());
                        stdout_redirect = true;
                    }
                    2 => {
                        cmd.stderr(Stdio::null());
                        stderr_redirect = true;
                    }
                    _ => {}
                },
                RedirectKind::Invalid(message) => {
                    let mut result = ExecutionResult::error(message.clone());
                    if message.contains("syntax error near unexpected token") {
                        result.exit_code = 2;
                    }
                    return Ok(result);
                }
                RedirectKind::Both | RedirectKind::BothAppend => {
                    if let Some(raw_target) = &redirect.target {
                        let target = expand_redirect_target(raw_target, &self.runtime);
                        let resolved = resolve_path(&target);
                        let file = if matches!(redirect.kind, RedirectKind::BothAppend) {
                            OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&resolved)
                                .map_err(|e| anyhow!("Failed to open '{}': {}", target, e))?
                        } else {
                            File::create(&resolved)
                                .map_err(|e| anyhow!("Failed to create '{}': {}", target, e))?
                        };
                        cmd.stdout(Stdio::from(
                            file.try_clone()
                                .map_err(|e| anyhow!("Failed to clone file descriptor: {}", e))?,
                        ));
                        cmd.stderr(Stdio::from(file));
                        stdout_redirect = true;
                        stderr_redirect = true;
                    }
                }
                RedirectKind::ProcessSubstitutionInputArg => {
                    if let Some(raw_target) = &redirect.target {
                        if let Some(path) = materialize_process_substitution_argument(raw_target) {
                            let file = File::open(&path)
                                .map_err(|e| anyhow!("Failed to open '{}': {}", path, e))?;
                            cmd.stdin(Stdio::from(file));
                            stdin_redirect = true;
                        }
                    }
                }
                RedirectKind::ProcessSubstitutionOutputArg => {
                    if let Some(raw_target) = &redirect.target {
                        if let Some((path, _)) = split_process_substitution_argument(raw_target) {
                            let file = File::create(&path)
                                .map_err(|e| anyhow!("Failed to create '{}': {}", path, e))?;
                            cmd.stdout(Stdio::from(file));
                            stdout_redirect = true;
                        }
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

        let capture_stdout = !stdout_redirect;
        let capture_stderr = !stderr_redirect && !stderr_to_stdout;

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

        let permanent_fds: Vec<(i32, PermanentFd)> = self
            .runtime
            .permanent_fds()
            .iter()
            .map(|(fd, state)| (*fd, *state))
            .collect();
        let permanent_stdout = self.runtime.get_permanent_stdout();
        let permanent_stderr = self.runtime.get_permanent_stderr();
        let permanent_stdin = self.runtime.get_permanent_stdin();

        unsafe {
            cmd.pre_exec(move || {
                for (fd, state) in &permanent_fds {
                    match state {
                        PermanentFd::Open(raw_fd) => {
                            if libc::dup2(*raw_fd, *fd) < 0 {
                                return Err(std::io::Error::last_os_error());
                            }
                        }
                        PermanentFd::Closed => {
                            libc::close(*fd);
                        }
                    }
                }
                if let Some(fd) = permanent_stdout {
                    if libc::dup2(fd, libc::STDOUT_FILENO) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if let Some(fd) = permanent_stderr {
                    if libc::dup2(fd, libc::STDERR_FILENO) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                if let Some(fd) = permanent_stdin {
                    if libc::dup2(fd, libc::STDIN_FILENO) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                }
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
                if e.kind() == std::io::ErrorKind::NotFound {
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

                    let result = ExecutionResult {
                        output: Output::Text(String::new()),
                        stderr: format!("{}\n", error_msg),
                        exit_code: 127,
                        error: None,
                    };

                    return if command.redirects.is_empty() {
                        Ok(result)
                    } else {
                        self.apply_redirects(result, &command.redirects)
                    };
                }

                return Err(anyhow!("Failed to execute '{}': {}", command_name, e));
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

            let mut stdout_str = if capture_stdout {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                String::new()
            };
            let stderr_str = if capture_stderr || stderr_to_stdout {
                String::from_utf8_lossy(&output.stderr).to_string()
            } else {
                String::new()
            };

            if stderr_to_stdout && !stderr_str.is_empty() {
                stdout_str.push_str(&stderr_str);
            }

            let stdout_to_stderr = command.redirects.iter().any(|redirect| {
                matches!(
                    redirect.kind,
                    RedirectKind::StdoutToStderr | RedirectKind::StdoutToFd(2)
                )
            });
            if stdout_to_stderr && !stdout_str.is_empty() {
                eprint!("{}", stdout_str);
                stdout_str.clear();
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

        finalize_process_substitution_outputs(&command.redirects, &command.args, &self.runtime)?;

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

    pub(crate) fn extract_process_substitution_stdin(
        &self,
        redirects: &[Redirect],
    ) -> Result<Option<i32>> {
        use std::fs::File;
        use std::os::fd::IntoRawFd;

        for redirect in redirects {
            if matches!(
                &redirect.kind,
                RedirectKind::Stdin | RedirectKind::ProcessSubstitutionInputArg
            ) {
                if let Some(raw_target) = &redirect.target {
                    if let Some(path) = materialize_process_substitution_argument(raw_target) {
                        let file = File::open(path)?;
                        return Ok(Some(file.into_raw_fd()));
                    }
                    if let Some(command) = raw_target.strip_prefix("__AUSH_PROCESS_SUBST__") {
                        let path = self.materialize_input_process_substitution(command)?;
                        let file = File::open(path)?;
                        return Ok(Some(file.into_raw_fd()));
                    }
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn extract_stdin_fd(&self, redirects: &[Redirect]) -> Result<Option<i32>> {
        for redirect in redirects {
            match &redirect.kind {
                RedirectKind::FdInputFrom(0, to) => {
                    return match self.runtime.permanent_fds().get(&(*to as i32)) {
                        Some(PermanentFd::Open(raw_fd)) => {
                            let dup = unsafe { libc::dup(*raw_fd) };
                            if dup < 0 {
                                Err(anyhow!("bad file descriptor"))
                            } else {
                                Ok(Some(dup))
                            }
                        }
                        Some(PermanentFd::Closed) => Ok(Some(-1)),
                        None => Ok(Some(*to as i32)),
                    };
                }
                _ => {}
            }
        }
        Ok(None)
    }

    pub(crate) fn extract_read_write_stdin_fd(
        &self,
        redirects: &[Redirect],
    ) -> Result<Option<i32>> {
        use std::fs::OpenOptions;
        use std::os::fd::IntoRawFd;
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
            if matches!(redirect.kind, RedirectKind::ReadWrite) {
                if let Some(raw_target) = &redirect.target {
                    let target = expand_redirect_target(raw_target, &self.runtime);
                    let file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(resolve_path(&target))?;
                    return Ok(Some(file.into_raw_fd()));
                }
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
            Statement::RedirectedCompound { statement, .. } => Self::is_exec_command(statement),
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

    fn maybe_confirm_effects(
        &self,
        command_name: &str,
        args: &[String],
    ) -> Result<ApprovalDecision> {
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
            Statement::Pipeline(_)
            | Statement::Subshell(_)
            | Statement::RedirectedCompound { .. } => self.execute_background_via_sh(&command_str),
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
