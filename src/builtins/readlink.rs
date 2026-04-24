use crate::executor::{ExecutionResult, Output};
use crate::runtime::Runtime;
use anyhow::Result;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CanonicalizeMode {
    /// -f / --canonicalize: all but the last component must exist.
    AllButLast,
    /// -e / --canonicalize-existing: all components must exist.
    Existing,
    /// -m / --canonicalize-missing: no components are required to exist.
    Missing,
}

struct ReadlinkOptions {
    canonicalize_mode: Option<CanonicalizeMode>,
    /// -n / --no-newline: do not output trailing newline.
    no_newline: bool,
    /// -q / --quiet / -s / --silent: suppress error messages.
    quiet: bool,
    files: Vec<String>,
}

impl ReadlinkOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut opts = ReadlinkOptions {
            canonicalize_mode: None,
            no_newline: false,
            quiet: false,
            files: vec![],
        };
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--" {
                opts.files.extend(args[i + 1..].iter().cloned());
                break;
            } else if arg == "--canonicalize" {
                opts.canonicalize_mode = Some(CanonicalizeMode::AllButLast);
            } else if arg == "--canonicalize-existing" {
                opts.canonicalize_mode = Some(CanonicalizeMode::Existing);
            } else if arg == "--canonicalize-missing" {
                opts.canonicalize_mode = Some(CanonicalizeMode::Missing);
            } else if arg == "--no-newline" {
                opts.no_newline = true;
            } else if arg == "--quiet" || arg == "--silent" {
                opts.quiet = true;
            } else if arg.starts_with('-') && arg.len() > 1 && arg != "-" {
                for ch in arg[1..].chars() {
                    match ch {
                        'f' => opts.canonicalize_mode = Some(CanonicalizeMode::AllButLast),
                        'e' => opts.canonicalize_mode = Some(CanonicalizeMode::Existing),
                        'm' => opts.canonicalize_mode = Some(CanonicalizeMode::Missing),
                        'n' => opts.no_newline = true,
                        'q' | 's' => opts.quiet = true,
                        _ => return Err(format!("readlink: invalid option -- '{}'", ch)),
                    }
                }
            } else {
                opts.files.push(arg.clone());
            }
            i += 1;
        }
        Ok(opts)
    }
}

fn resolve_path(path_str: &str, cwd: &Path) -> PathBuf {
    let p = if path_str.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            home.join(path_str.trim_start_matches("~/"))
        } else {
            PathBuf::from(path_str)
        }
    } else {
        PathBuf::from(path_str)
    };
    if p.is_absolute() {
        p
    } else {
        cwd.join(p)
    }
}

#[derive(Debug)]
enum PendingComponent {
    RootDir,
    CurDir,
    ParentDir,
    Normal(OsString),
}

fn push_components(path: &Path, queue: &mut VecDeque<PendingComponent>) {
    for component in path.components() {
        match component {
            Component::RootDir => queue.push_back(PendingComponent::RootDir),
            Component::CurDir => queue.push_back(PendingComponent::CurDir),
            Component::ParentDir => queue.push_back(PendingComponent::ParentDir),
            Component::Normal(part) => {
                queue.push_back(PendingComponent::Normal(part.to_os_string()))
            }
            Component::Prefix(prefix) => {
                queue.push_back(PendingComponent::Normal(prefix.as_os_str().to_os_string()))
            }
        }
    }
}

fn prepend_components(path: &Path, queue: &mut VecDeque<PendingComponent>) {
    let mut prefixed = VecDeque::new();
    push_components(path, &mut prefixed);
    prefixed.append(queue);
    *queue = prefixed;
}

fn is_ignorable_missing_lookup(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::NotADirectory
    )
}

fn not_a_directory_error(path: &Path) -> io::Error {
    io::Error::new(
        io::ErrorKind::NotADirectory,
        format!("Not a directory: {}", path.display()),
    )
}

fn has_trailing_separator(path: &str) -> bool {
    path.chars().last().is_some_and(std::path::is_separator)
}

fn canonicalize_path(path: &Path, mode: CanonicalizeMode) -> io::Result<PathBuf> {
    const MAX_SYMLINK_EXPANSIONS: usize = 40;

    let mut resolved = PathBuf::new();
    let mut pending = VecDeque::new();
    let mut symlink_expansions = 0usize;

    push_components(path, &mut pending);

    while let Some(component) = pending.pop_front() {
        match component {
            PendingComponent::RootDir => {
                resolved = PathBuf::from(std::path::MAIN_SEPARATOR.to_string());
            }
            PendingComponent::CurDir => {}
            PendingComponent::ParentDir => {
                if !resolved.pop() && resolved.as_os_str().is_empty() {
                    resolved = PathBuf::from(std::path::MAIN_SEPARATOR.to_string());
                }
            }
            PendingComponent::Normal(part) => {
                let candidate = if resolved.as_os_str().is_empty() {
                    PathBuf::from(&part)
                } else {
                    resolved.join(&part)
                };
                let is_last = pending.is_empty();

                match std::fs::symlink_metadata(&candidate) {
                    Ok(metadata) if metadata.file_type().is_symlink() => {
                        symlink_expansions += 1;
                        if symlink_expansions > MAX_SYMLINK_EXPANSIONS {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                "Too many levels of symbolic links",
                            ));
                        }

                        let target = std::fs::read_link(&candidate)?;
                        if target.is_absolute() {
                            resolved = PathBuf::new();
                        }
                        prepend_components(&target, &mut pending);
                    }
                    Ok(metadata) => {
                        if pending.front().is_some()
                            && mode != CanonicalizeMode::Missing
                            && !metadata.file_type().is_dir()
                        {
                            return Err(not_a_directory_error(&candidate));
                        }
                        resolved.push(&part);
                    }
                    Err(error) => match mode {
                        CanonicalizeMode::Existing => return Err(error),
                        CanonicalizeMode::AllButLast => {
                            if is_last && error.kind() == io::ErrorKind::NotFound {
                                resolved.push(&part);
                            } else {
                                return Err(error);
                            }
                        }
                        CanonicalizeMode::Missing => {
                            if is_ignorable_missing_lookup(&error) {
                                resolved.push(&part);
                            } else {
                                return Err(error);
                            }
                        }
                    },
                }
            }
        }
    }

    if resolved.as_os_str().is_empty() {
        Ok(PathBuf::from(std::path::MAIN_SEPARATOR.to_string()))
    } else {
        Ok(resolved)
    }
}

pub fn builtin_readlink(args: &[String], runtime: &mut Runtime) -> Result<ExecutionResult> {
    if args.len() == 1 && args[0] == "--help" {
        return Ok(ExecutionResult::success(HELP_TEXT.to_string()));
    }

    if args.is_empty() {
        return Ok(ExecutionResult {
            output: Output::Text(String::new()),
            stderr: "readlink: missing operand\nTry 'readlink --help' for more information.\n"
                .to_string(),
            exit_code: 1,
            error: None,
        });
    }

    let opts = match ReadlinkOptions::parse(args) {
        Ok(o) => o,
        Err(e) => {
            return Ok(ExecutionResult {
                output: Output::Text(String::new()),
                stderr: format!("{}\n", e),
                exit_code: 1,
                error: None,
            })
        }
    };

    if opts.files.is_empty() {
        return Ok(ExecutionResult {
            output: Output::Text(String::new()),
            stderr: "readlink: missing operand\nTry 'readlink --help' for more information.\n"
                .to_string(),
            exit_code: 1,
            error: None,
        });
    }

    let cwd = runtime.get_cwd().clone();
    let mut output = String::new();
    let mut stderr_output = String::new();
    let mut exit_code = 0;

    for (i, file_arg) in opts.files.iter().enumerate() {
        let path = resolve_path(file_arg, &cwd);
        let is_last = i == opts.files.len() - 1;

        let resolved = if let Some(mode) = opts.canonicalize_mode {
            match canonicalize_path(&path, mode).and_then(|canonicalized| {
                if mode != CanonicalizeMode::Missing && has_trailing_separator(file_arg) {
                    match std::fs::metadata(&canonicalized) {
                        Ok(metadata) if !metadata.is_dir() => {
                            Err(not_a_directory_error(&canonicalized))
                        }
                        Ok(_) => Ok(canonicalized),
                        Err(error)
                            if mode == CanonicalizeMode::AllButLast
                                && error.kind() == io::ErrorKind::NotFound =>
                        {
                            Ok(canonicalized)
                        }
                        Err(error) => Err(error),
                    }
                } else {
                    Ok(canonicalized)
                }
            }) {
                Ok(p) => Some(p.to_string_lossy().to_string()),
                Err(e) => {
                    if !opts.quiet {
                        stderr_output.push_str(&format!("readlink: {}: {}\n", file_arg, e));
                    }
                    exit_code = 1;
                    None
                }
            }
        } else {
            match std::fs::read_link(&path) {
                Ok(target) => Some(target.to_string_lossy().to_string()),
                Err(e) => {
                    if !opts.quiet {
                        stderr_output.push_str(&format!("readlink: {}: {}\n", file_arg, e));
                    }
                    exit_code = 1;
                    None
                }
            }
        };

        if let Some(target) = resolved {
            if opts.no_newline && is_last {
                output.push_str(&target);
            } else {
                output.push_str(&target);
                output.push('\n');
            }
        }
    }

    Ok(ExecutionResult {
        output: Output::Text(output),
        stderr: stderr_output,
        exit_code,
        error: None,
    })
}

const HELP_TEXT: &str = "Usage: readlink [OPTION]... FILE...
Print value of a symbolic link or canonical file name.

Options:
  -f, --canonicalize            canonicalize by following every symlink;
                                all but the last component must exist
  -e, --canonicalize-existing   canonicalize; all components must exist
  -m, --canonicalize-missing    canonicalize even if components missing
  -n, --no-newline              do not output trailing newline
  -q, --quiet, -s, --silent     suppress most error messages
  --help                        display this help and exit

Examples:
  readlink /path/to/symlink     print where symlink points
  readlink -f ./relative/path   resolve to absolute canonical path
";

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_runtime(dir: &TempDir) -> Runtime {
        let mut rt = Runtime::new();
        rt.set_cwd(dir.path().to_path_buf());
        rt
    }

    #[test]
    fn test_readlink_symlink() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        let target = tmp.path().join("target.txt");
        std::fs::write(&target, "data").unwrap();
        std::os::unix::fs::symlink(&target, tmp.path().join("link.txt")).unwrap();

        let result = builtin_readlink(&["link.txt".to_string()], &mut rt).unwrap();
        assert_eq!(result.exit_code, 0, "stderr: {}", result.stderr);
        assert!(result.stdout().trim() == target.to_string_lossy());
    }

    #[test]
    fn test_readlink_not_a_symlink_errors() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        std::fs::write(tmp.path().join("real.txt"), "data").unwrap();

        let result = builtin_readlink(&["real.txt".to_string()], &mut rt).unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("readlink:"));
    }

    #[test]
    fn test_readlink_canonicalize() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        let target = tmp.path().join("target.txt");
        std::fs::write(&target, "data").unwrap();
        std::os::unix::fs::symlink(&target, tmp.path().join("link.txt")).unwrap();

        let result =
            builtin_readlink(&["-f".to_string(), "link.txt".to_string()], &mut rt).unwrap();
        assert_eq!(result.exit_code, 0, "stderr: {}", result.stderr);
        assert_eq!(
            result.stdout().trim(),
            target.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn test_readlink_f_allows_missing_final_component_but_e_does_not() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        let existing_dir = tmp.path().join("existing");
        std::fs::create_dir(&existing_dir).unwrap();

        let missing_path = existing_dir.canonicalize().unwrap().join("missing.txt");

        let canonicalized = builtin_readlink(
            &["-f".to_string(), "existing/missing.txt".to_string()],
            &mut rt,
        )
        .unwrap();
        assert_eq!(
            canonicalized.exit_code, 0,
            "stderr: {}",
            canonicalized.stderr
        );
        assert_eq!(
            canonicalized.stdout().trim(),
            missing_path.to_string_lossy()
        );

        let existing_only = builtin_readlink(
            &["-e".to_string(), "existing/missing.txt".to_string()],
            &mut rt,
        )
        .unwrap();
        assert_eq!(existing_only.exit_code, 1);
        assert!(existing_only.stderr.contains("readlink:"));
    }

    #[test]
    fn test_readlink_m_allows_missing_components_after_symlink_prefix() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        let real_dir = tmp.path().join("real");
        std::fs::create_dir(&real_dir).unwrap();
        std::os::unix::fs::symlink(&real_dir, tmp.path().join("link")).unwrap();

        let missing_tail = real_dir.canonicalize().unwrap().join("missing/file.txt");

        let canonicalized_missing = builtin_readlink(
            &["-m".to_string(), "link/missing/file.txt".to_string()],
            &mut rt,
        )
        .unwrap();
        assert_eq!(
            canonicalized_missing.exit_code, 0,
            "stderr: {}",
            canonicalized_missing.stderr
        );
        assert_eq!(
            canonicalized_missing.stdout().trim(),
            missing_tail.to_string_lossy()
        );

        let canonicalized_f = builtin_readlink(
            &["-f".to_string(), "link/missing/file.txt".to_string()],
            &mut rt,
        )
        .unwrap();
        assert_eq!(canonicalized_f.exit_code, 1);
        assert!(canonicalized_f.stderr.contains("readlink:"));
    }

    #[test]
    fn test_readlink_trailing_separator_distinguishes_missing_mode() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        let file = tmp.path().join("file.txt");
        std::fs::write(&file, "data").unwrap();

        let all_but_last =
            builtin_readlink(&["-f".to_string(), "file.txt/".to_string()], &mut rt).unwrap();
        assert_eq!(all_but_last.exit_code, 1);
        assert!(all_but_last.stderr.contains("Not a directory"));

        let existing_only =
            builtin_readlink(&["-e".to_string(), "file.txt/".to_string()], &mut rt).unwrap();
        assert_eq!(existing_only.exit_code, 1);
        assert!(existing_only.stderr.contains("Not a directory"));

        let missing_mode =
            builtin_readlink(&["-m".to_string(), "file.txt/".to_string()], &mut rt).unwrap();
        assert_eq!(missing_mode.exit_code, 0, "stderr: {}", missing_mode.stderr);
        assert_eq!(
            missing_mode.stdout().trim(),
            file.canonicalize().unwrap().to_string_lossy()
        );
    }

    #[test]
    fn test_readlink_modes_distinguish_missing_symlink_target() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);
        std::os::unix::fs::symlink("missing/nested.txt", tmp.path().join("link")).unwrap();

        let canonicalized_missing =
            builtin_readlink(&["-m".to_string(), "link".to_string()], &mut rt).unwrap();
        assert_eq!(
            canonicalized_missing.exit_code, 0,
            "stderr: {}",
            canonicalized_missing.stderr
        );
        assert_eq!(
            canonicalized_missing.stdout().trim(),
            tmp.path()
                .canonicalize()
                .unwrap()
                .join("missing/nested.txt")
                .to_string_lossy()
        );

        let canonicalized_f =
            builtin_readlink(&["-f".to_string(), "link".to_string()], &mut rt).unwrap();
        assert_eq!(canonicalized_f.exit_code, 1);
        assert!(canonicalized_f.stderr.contains("readlink:"));

        let canonicalized_e =
            builtin_readlink(&["-e".to_string(), "link".to_string()], &mut rt).unwrap();
        assert_eq!(canonicalized_e.exit_code, 1);
        assert!(canonicalized_e.stderr.contains("readlink:"));
    }

    #[test]
    fn test_readlink_no_args_errors() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);

        let result = builtin_readlink(&[], &mut rt).unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(result.stderr.contains("missing operand"));
    }

    #[test]
    fn test_readlink_quiet_suppresses_errors() {
        let tmp = TempDir::new().unwrap();
        let mut rt = make_runtime(&tmp);

        let result =
            builtin_readlink(&["-q".to_string(), "nonexistent.txt".to_string()], &mut rt).unwrap();
        assert_eq!(result.exit_code, 1);
        assert!(
            result.stderr.is_empty(),
            "expected no stderr, got: {}",
            result.stderr
        );
    }
}
