# AUSH Documentation

## Getting Started

- [PERFORMANCE.md](PERFORMANCE.md) -- Performance characteristics and benchmarks
- [benchmarking.md](benchmarking.md) -- How to benchmark AUSH
- [pgo-build.md](pgo-build.md) -- Profile-guided optimization builds
- [POSIX_COMPLIANCE_AUDIT.md](POSIX_COMPLIANCE_AUDIT.md) -- POSIX compliance status

## Shell Features

- [variable-expansion.md](variable-expansion.md) -- Parameter and variable expansion
- [glob-expansion.md](glob-expansion.md) -- Pathname expansion and globbing
- [command-substitution.md](command-substitution.md) -- Command substitution (`$(...)`)
- [file-redirection.md](file-redirection.md) -- I/O redirection
- [subshells.md](subshells.md) -- Subshell execution
- [functions.md](functions.md) -- Shell functions
- [exit-codes.md](exit-codes.md) -- Exit code conventions
- [shell-options.md](shell-options.md) -- Shell option flags
- [signal-handling.md](signal-handling.md) -- Signal handling behavior
- [job-control.md](job-control.md) -- Job control (bg, fg, jobs)
- [error-recovery.md](error-recovery.md) -- Error recovery design

## Interactive Features

- [command-history.md](command-history.md) -- Command history
- [tab-completion.md](tab-completion.md) -- Tab completion
- [context-detection.md](context-detection.md) -- Project context detection
- [login-shell-init.md](login-shell-init.md) -- Login shell initialization
- [non-tty-mode.md](non-tty-mode.md) -- Non-TTY / scripting mode

## Builtin Reference

- [builtins/find.md](builtins/find.md) -- `find` builtin
- [builtins/ls.md](builtins/ls.md) -- `ls` builtin
- [PRINTF_QUICK_REFERENCE.md](PRINTF_QUICK_REFERENCE.md) -- `printf` builtin
- [TEST_BUILTIN_QUICK_REFERENCE.md](TEST_BUILTIN_QUICK_REFERENCE.md) -- `test` / `[` builtin

## AI Agent Integration

- [AI_AGENT_GUIDE.md](AI_AGENT_GUIDE.md) -- Guide for AI agents using AUSH
- [AI_AGENT_JSON_REFERENCE.md](AI_AGENT_JSON_REFERENCE.md) -- JSON output reference for agents

## Architecture

- [daemon-architecture.md](daemon-architecture.md) -- Daemon mode architecture

## Internal

Development notes, implementation summaries, and design docs are in [internal/](internal/).
