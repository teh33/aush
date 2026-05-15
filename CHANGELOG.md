# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

### Release positioning
- Preparing AUSH 0.1.0 as a public alpha / beta-candidate, not a full beta.
- AUSH is a real local shell and is not sandboxed by default; commands run with the current user's permissions.
- Login-shell use is available for Ghostty/startup-shell trials, but users should keep a fallback shell and rollback path.
- External shell behavior corpora are used as regression inputs; this is not a product-level Brush compatibility promise.

### Added
- POSIX-compliant shell with 45+ built-in commands
- Daemon mode with pre-forked worker pool for sub-millisecond dispatch
- AI agent optimized builtins with `--json` output
- Built-in `git_status`, `git_log`, `git_diff` commands
- Built-in `find`, `grep`, `ls`, `cat` with JSON output
- HTTP `fetch` builtin for API calls
- Job control (bg, fg, jobs, wait)
- Command history with file persistence
- Tab completion
- Signal handling (SIGINT, SIGTSTP, SIGCHLD, SIGTERM)
- Variable expansion, command substitution, arithmetic expansion
- Here documents and here strings
- Functions with local variables
- Comprehensive test suite (500+ tests)
