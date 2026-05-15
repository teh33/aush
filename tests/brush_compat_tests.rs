//! Adapter for running a focused subset of an external shell behavior corpus
//! against AUSH.
//!
//! This is intentionally small: it proves we can consume an existing YAML shell
//! behavior corpus without vendoring its Rust harness. Broader field support can
//! be added as AUSH's release target set expands.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

#[derive(Debug, Deserialize)]
struct BrushCaseFile {
    name: String,
    cases: Vec<BrushCase>,
}

#[derive(Debug, Deserialize)]
struct BrushCase {
    name: String,
    #[serde(default)]
    stdin: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
    #[serde(default)]
    test_files: Vec<TestFile>,
    #[serde(default)]
    ignore_stderr: bool,
    #[serde(default)]
    ignore_stdout: bool,
    #[serde(default)]
    skip: bool,
    #[serde(default)]
    known_failure: bool,
}

#[derive(Debug, Deserialize)]
struct TestFile {
    path: String,
    #[serde(default)]
    contents: String,
    #[serde(default)]
    executable: bool,
}

#[derive(Debug)]
struct ShellOutput {
    status: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
}

fn case_timeout() -> Duration {
    let secs = std::env::var("AUSH_BRUSH_CASE_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5);
    Duration::from_secs(secs)
}

#[derive(Debug, Default)]
struct CorpusFilter {
    file: Option<String>,
    case_name: Option<String>,
}

impl CorpusFilter {
    fn from_env() -> Self {
        Self {
            file: std::env::var("AUSH_BRUSH_CASE_FILE").ok(),
            case_name: std::env::var("AUSH_BRUSH_CASE_NAME").ok(),
        }
    }

    fn matches_file(&self, file_name: &str) -> bool {
        self.file
            .as_deref()
            .is_none_or(|filter| file_name == filter || file_name.ends_with(filter))
    }

    fn matches_case(&self, case_name: &str) -> bool {
        self.case_name
            .as_deref()
            .is_none_or(|filter| case_name == filter || case_name.contains(filter))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompatibilityClass {
    MustFix,
    ShouldFix,
    Deferred,
}

impl CompatibilityClass {
    fn label(self) -> &'static str {
        match self {
            CompatibilityClass::MustFix => "must-fix",
            CompatibilityClass::ShouldFix => "should-fix",
            CompatibilityClass::Deferred => "deferred",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CaseOutcome {
    Passed,
    Failed,
    Ignored,
    Unsupported,
}

#[derive(Debug)]
struct CaseReport {
    file_name: String,
    case_name: String,
    class: CompatibilityClass,
    outcome: CaseOutcome,
    reason: Option<String>,
}

#[derive(Debug, Default)]
struct CorpusReport {
    total: usize,
    passed: usize,
    failed: usize,
    ignored: usize,
    unsupported: usize,
    must_fix_failed: usize,
    should_fix_failed: usize,
    deferred_failed: usize,
    deferred_ignored: usize,
    cases: Vec<CaseReport>,
}

impl CorpusReport {
    fn record(&mut self, case: CaseReport) {
        self.total += 1;
        match case.outcome {
            CaseOutcome::Passed => self.passed += 1,
            CaseOutcome::Failed => {
                self.failed += 1;
                match case.class {
                    CompatibilityClass::MustFix => self.must_fix_failed += 1,
                    CompatibilityClass::ShouldFix => self.should_fix_failed += 1,
                    CompatibilityClass::Deferred => self.deferred_failed += 1,
                }
            }
            CaseOutcome::Ignored => {
                self.ignored += 1;
                if case.class == CompatibilityClass::Deferred {
                    self.deferred_ignored += 1;
                }
            }
            CaseOutcome::Unsupported => self.unsupported += 1,
        }
        self.cases.push(case);
    }

    fn summary(&self) -> String {
        format!(
            "Brush corpus report: total={}, passed={}, failed={}, ignored={}, unsupported={}, must-fix failed={}, should-fix failed={}, deferred failed={}, deferred ignored={}",
            self.total,
            self.passed,
            self.failed,
            self.ignored,
            self.unsupported,
            self.must_fix_failed,
            self.should_fix_failed,
            self.deferred_failed,
            self.deferred_ignored
        )
    }

    fn to_markdown(&self) -> String {
        let mut out = String::new();
        writeln!(out, "# Brush corpus report").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "{}", self.summary()).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| Outcome | Class | Case | Reason |").unwrap();
        writeln!(out, "|---|---|---|---|").unwrap();
        for case in &self.cases {
            if case.outcome == CaseOutcome::Passed {
                continue;
            }
            writeln!(
                out,
                "| {:?} | {} | `{}`::`{}` | {} |",
                case.outcome,
                case.class.label(),
                case.file_name,
                case.case_name,
                case.reason.as_deref().unwrap_or("")
            )
            .unwrap();
        }
        out
    }
}

#[test]
#[ignore = "reports full Brush corpus status; run manually when assessing compatibility"]
fn brush_compat_full_corpus_report() {
    let brush_root = brush_cases_root();
    if !brush_root.exists() {
        eprintln!(
            "skipping external shell corpus report; cases not found at {}",
            brush_root.display()
        );
        return;
    }

    let limit = std::env::var("AUSH_BRUSH_REPORT_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let start = std::env::var("AUSH_BRUSH_REPORT_START")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1);
    let filter = CorpusFilter::from_env();
    let report = run_corpus_report(&brush_root, limit, start, &filter);
    println!("{}", report.summary());

    if let Some(path) = std::env::var_os("AUSH_BRUSH_REPORT") {
        fs::write(&path, report.to_markdown()).expect("write Brush corpus report");
        println!("wrote report to {}", PathBuf::from(path).display());
    }
}

#[test]
fn brush_compat_smoke_subset_matches_bash() {
    let brush_root = brush_cases_root();
    if !brush_root.exists() {
        eprintln!(
            "skipping external shell corpus adapter smoke test; cases not found at {}",
            brush_root.display()
        );
        return;
    }

    let selected = [
        ("basic.yaml", "Basic -c usage"),
        ("basic.yaml", "Basic stdin usage"),
        ("basic.yaml", "Basic sequence"),
        ("and_or.yaml", "Basic &&"),
        ("and_or.yaml", "Basic ||"),
        ("list.yaml", "Ignore single quote in comment in list"),
    ];

    for (file_name, case_name) in selected {
        let case_file = load_case_file(&brush_root.join(file_name));
        let case = case_file
            .cases
            .iter()
            .find(|case| case.name == case_name)
            .unwrap_or_else(|| panic!("missing Brush case {case_name:?} in {file_name}"));

        assert_case_supported_by_smoke_adapter(&case_file.name, case);
        assert!(
            !case.skip,
            "selected Brush case {file_name}::{case_name} is marked skip upstream"
        );
        assert!(
            !case.known_failure,
            "selected Brush case {file_name}::{case_name} is marked known_failure upstream"
        );

        let bash_temp = TempDir::new().expect("create bash tempdir");
        materialize_test_files(bash_temp.path(), &case.test_files);
        let bash = run_shell("bash", &["--norc", "--noprofile"], case, bash_temp.path());

        let aush_temp = TempDir::new().expect("create aush tempdir");
        materialize_test_files(aush_temp.path(), &case.test_files);
        let aush_bin = env!("CARGO_BIN_EXE_aush");
        let aush = run_shell(aush_bin, &["--no-rc"], case, aush_temp.path());

        if !case.ignore_stdout {
            assert_eq!(
                String::from_utf8_lossy(&aush.stdout),
                String::from_utf8_lossy(&bash.stdout),
                "stdout mismatch for Brush case {file_name}::{case_name}\nAUSH stderr:\n{}\nBash stderr:\n{}",
                String::from_utf8_lossy(&aush.stderr),
                String::from_utf8_lossy(&bash.stderr),
            );
        }
        assert_eq!(
            aush.status, bash.status,
            "exit status mismatch for Brush case {file_name}::{case_name}\nAUSH stdout:\n{}\nAUSH stderr:\n{}\nBash stdout:\n{}\nBash stderr:\n{}",
            String::from_utf8_lossy(&aush.stdout),
            String::from_utf8_lossy(&aush.stderr),
            String::from_utf8_lossy(&bash.stdout),
            String::from_utf8_lossy(&bash.stderr),
        );

        if !case.ignore_stderr {
            assert_eq!(
                String::from_utf8_lossy(&aush.stderr),
                String::from_utf8_lossy(&bash.stderr),
                "stderr mismatch for Brush case {file_name}::{case_name}"
            );
        }
    }
}

fn run_corpus_report(
    brush_root: &Path,
    limit: Option<usize>,
    start: usize,
    filter: &CorpusFilter,
) -> CorpusReport {
    let mut report = CorpusReport::default();
    let mut seen = 0usize;
    let mut files = Vec::new();
    collect_yaml_files(brush_root, &mut files);
    files.sort();

    for file in files {
        let file_name = file
            .strip_prefix(brush_root)
            .unwrap_or(&file)
            .to_string_lossy()
            .into_owned();
        if !filter.matches_file(&file_name) {
            continue;
        }

        let case_file = load_case_file(&file);

        for case in &case_file.cases {
            if !filter.matches_case(&case.name) {
                continue;
            }
            seen += 1;
            if seen < start {
                continue;
            }
            if limit.is_some_and(|limit| report.total >= limit) {
                return report;
            }
            let case_index = seen;
            eprintln!("[{case_index}] {file_name}::{}", case.name);
            let _ = io::stderr().flush();
            report.record(run_case_for_report(&file_name, case));
        }
    }

    report
}

fn collect_yaml_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read Brush cases directory {}: {err}", dir.display()));
    for entry in entries {
        let path = entry.expect("read dir entry").path();
        if path.is_dir() {
            collect_yaml_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "yaml") {
            files.push(path);
        }
    }
}

fn classify_case(file_name: &str, _case: &BrushCase) -> CompatibilityClass {
    const MUST_FIX_PREFIXES: &[&str] = &[
        "options/set-e.yaml",
        "options/set-u.yaml",
        "redirection.yaml",
        "word_expansion/params.yaml",
        "word_expansion/command_substitution.yaml",
        "arithmetic.yaml",
        "builtins/test.yaml",
        "builtins/read.yaml",
        "builtins/printf.yaml",
        "errors.yaml",
    ];
    const SHOULD_FIX_PREFIXES: &[&str] = &[
        "arrays.yaml",
        "builtins/declare.yaml",
        "builtins/getopts.yaml",
        "builtins/mapfile.yaml",
        "builtins/type.yaml",
        "compound_cmds/case.yaml",
        "compound_cmds/for.yaml",
        "compound_cmds/while.yaml",
        "compound_cmds/until.yaml",
        "builtins/complete.yaml",
        "builtins/compgen.yaml",
    ];
    const DEFERRED_PREFIXES: &[&str] = &[
        "compound_cmds/coproc.yaml",
        "builtins/trap.yaml",
        "callstack.yaml",
        "nameref.yaml",
        "prompt.yaml",
        "builtins/fc.yaml",
    ];

    if DEFERRED_PREFIXES
        .iter()
        .any(|prefix| file_name.starts_with(prefix))
    {
        CompatibilityClass::Deferred
    } else if MUST_FIX_PREFIXES
        .iter()
        .any(|prefix| file_name.starts_with(prefix))
    {
        CompatibilityClass::MustFix
    } else if SHOULD_FIX_PREFIXES
        .iter()
        .any(|prefix| file_name.starts_with(prefix))
    {
        CompatibilityClass::ShouldFix
    } else {
        CompatibilityClass::ShouldFix
    }
}

fn run_case_for_report(file_name: &str, case: &BrushCase) -> CaseReport {
    let mut case_report = CaseReport {
        file_name: file_name.to_string(),
        case_name: case.name.clone(),
        class: classify_case(file_name, case),
        outcome: CaseOutcome::Unsupported,
        reason: None,
    };

    if file_name == "compound_cmds/coproc.yaml" {
        case_report.outcome = CaseOutcome::Ignored;
        case_report.reason = Some("AUSH known-gap: coproc can hang corpus runs".to_string());
        return case_report;
    }

    if case.skip || case.known_failure {
        case_report.outcome = CaseOutcome::Ignored;
        case_report.reason = Some(if case.skip {
            "marked skip upstream".to_string()
        } else {
            "marked known_failure upstream".to_string()
        });
        return case_report;
    }

    if case.stdin.is_none() && case.args.is_empty() {
        case_report.reason = Some("no stdin script or args".to_string());
        return case_report;
    }

    let bash_temp = match TempDir::new() {
        Ok(temp) => temp,
        Err(err) => {
            case_report.reason = Some(format!("create bash tempdir: {err}"));
            return case_report;
        }
    };
    materialize_test_files(bash_temp.path(), &case.test_files);
    let bash = run_shell("bash", &["--norc", "--noprofile"], case, bash_temp.path());

    let aush_temp = match TempDir::new() {
        Ok(temp) => temp,
        Err(err) => {
            case_report.reason = Some(format!("create aush tempdir: {err}"));
            return case_report;
        }
    };
    materialize_test_files(aush_temp.path(), &case.test_files);
    let aush_bin = env!("CARGO_BIN_EXE_aush");
    let aush = run_shell(aush_bin, &["--no-rc"], case, aush_temp.path());

    let bash_stderr = comparable_stderr(file_name, &case.name, &bash.stderr);
    let aush_stderr = comparable_stderr(file_name, &case.name, &aush.stderr);

    let stdout_matches =
        !bash.timed_out && !aush.timed_out && (case.ignore_stdout || aush.stdout == bash.stdout);
    let stderr_matches =
        !bash.timed_out && !aush.timed_out && (case.ignore_stderr || aush_stderr == bash_stderr);
    let status_matches = !bash.timed_out && !aush.timed_out && aush.status == bash.status;

    if stdout_matches && stderr_matches && status_matches {
        case_report.outcome = CaseOutcome::Passed;
        return case_report;
    }

    case_report.outcome = CaseOutcome::Failed;
    let mut reasons = Vec::new();
    if bash.timed_out {
        reasons.push("bash timed out".to_string());
    }
    if aush.timed_out {
        reasons.push("aush timed out".to_string());
    }
    if !stdout_matches && !bash.timed_out && !aush.timed_out {
        reasons.push(format!(
            "stdout differs: bash={:?}, aush={:?}",
            String::from_utf8_lossy(&bash.stdout),
            String::from_utf8_lossy(&aush.stdout)
        ));
    }
    if !stderr_matches && !bash.timed_out && !aush.timed_out {
        reasons.push(format!(
            "stderr differs: bash={:?}, aush={:?}",
            String::from_utf8_lossy(&bash_stderr),
            String::from_utf8_lossy(&aush_stderr)
        ));
    }
    if !status_matches && !bash.timed_out && !aush.timed_out {
        reasons.push(format!(
            "status differs: bash={}, aush={}",
            bash.status, aush.status
        ));
    }
    case_report.reason = Some(reasons.join("; "));
    case_report
}

fn comparable_stderr(file_name: &str, case_name: &str, stderr: &[u8]) -> Vec<u8> {
    // macOS Bash 3.2 reports an advisory fchmod warning for cp's `/dev/fd/*`
    // process-substitution output path, while AUSH's pragmatic temp-file
    // implementation does not need that fd-specific chmod path. Treat the
    // warning as harness noise so this case still checks stdout and status.
    if file_name == "redirection.yaml" && case_name == "Process substitution: input + output" {
        let text = String::from_utf8_lossy(stderr);
        return text
            .lines()
            .filter(|line| {
                !(line.starts_with("cp: /dev/fd/")
                    && line.ends_with(": fchmod failed: Invalid argument"))
            })
            .collect::<Vec<_>>()
            .join("\n")
            .into_bytes();
    }
    stderr.to_vec()
}

fn brush_cases_root() -> PathBuf {
    std::env::var_os("AUSH_BRUSH_CASES")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/asher/brush/brush-shell/tests/cases/compat"))
}

fn load_case_file(path: &Path) -> BrushCaseFile {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read Brush case file {}: {err}", path.display()));
    serde_yaml::from_str(&text)
        .unwrap_or_else(|err| panic!("parse Brush case file {}: {err}", path.display()))
}

fn assert_case_supported_by_smoke_adapter(file_name: &str, case: &BrushCase) {
    assert!(
        case.stdin.is_some() || !case.args.is_empty(),
        "Brush case {file_name}::{} has neither stdin script nor CLI args",
        case.name
    );
}

fn materialize_test_files(root: &Path, files: &[TestFile]) {
    for file in files {
        let path = root.join(&file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create test file parent");
        }
        fs::write(&path, &file.contents).expect("write Brush test file");

        #[cfg(unix)]
        if file.executable {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&path).expect("metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).expect("set executable bit");
        }
    }
}

fn run_shell(shell: &str, default_args: &[&str], case: &BrushCase, cwd: &Path) -> ShellOutput {
    let mut command = Command::new(shell);
    command
        .current_dir(cwd)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("HOME", cwd)
        .env("PWD", cwd)
        .args(default_args);

    if case.args.is_empty() {
        command
            .arg("-c")
            .arg(case.stdin.as_deref().expect("stdin script"));
    } else {
        command.args(&case.args);
        if let Some(stdin_script) = &case.stdin {
            command.arg(stdin_script);
        }
    }

    for (key, value) in &case.env {
        command.env(key, value);
    }

    let stdout_file = tempfile::NamedTempFile::new_in(cwd).expect("create stdout capture file");
    let stderr_file = tempfile::NamedTempFile::new_in(cwd).expect("create stderr capture file");
    let stdout_path = stdout_file.path().to_owned();
    let stderr_path = stderr_file.path().to_owned();
    command
        .stdout(Stdio::from(
            File::create(&stdout_path).expect("open stdout capture file"),
        ))
        .stderr(Stdio::from(
            File::create(&stderr_path).expect("open stderr capture file"),
        ));

    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    let mut child = command
        .spawn()
        .unwrap_or_else(|err| panic!("spawn {shell}: {err}"));
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = fs::read(&stdout_path).expect("read stdout capture");
                let stderr = fs::read(&stderr_path).expect("read stderr capture");
                return ShellOutput {
                    status: status.code().unwrap_or(128),
                    stdout,
                    stderr,
                    timed_out: false,
                };
            }
            Ok(None) if start.elapsed() >= case_timeout() => {
                let pid = child.id() as i32;
                let _ = unsafe { libc::killpg(pid, libc::SIGKILL) };
                let _ = child.kill();
                let _ = child.wait();
                let stdout = fs::read(&stdout_path).unwrap_or_default();
                let stderr = fs::read(&stderr_path).unwrap_or_default();
                return ShellOutput {
                    status: 124,
                    stdout,
                    stderr,
                    timed_out: true,
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(err) => panic!("wait for {shell}: {err}"),
        }
    }
}
