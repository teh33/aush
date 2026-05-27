# AUSH

[![CI](https://github.com/teh33/aush/actions/workflows/integration-tests.yml/badge.svg)](https://github.com/teh33/aush/actions/workflows/integration-tests.yml)
[![Crates.io](https://img.shields.io/crates/v/aush.svg)](https://crates.io/crates/aush)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**Actually Usable Shell** — a shell for automation, coding agents, and developer workflows where shell syntax is useful but stringly text plumbing is the bottleneck.

AUSH is not trying to be “a better Bash” for your `.bashrc`. Bash is already excellent at being Bash. AUSH is trying to make the shell a better **programmable command runtime**: deterministic startup, native hot-path commands, structured output, and a daemon path for low-latency local clients.

```sh
# deterministic command execution: no startup files
aush --no-rc -c 'printf "one\ntwo\n" | tail -n 1'
# two

# shell-style status codes
aush --no-rc -c 'grep -q beta <<EOF
alpha
beta
EOF'
echo $?
# 0

# structured output from native builtins
aush --no-rc -c 'find src --json -name "*.rs" | head -n 3'
```

## Why would this exist?

Unix shells are still the best glue language for local machines, but they are awkward as an embedding/runtime layer for modern tools:

- startup is often nondeterministic because user rc files mutate behavior;
- every tiny text/file operation usually means another process spawn;
- pipelines collapse structured state into text and force lossy reparsing;
- automation wants predictable errors, machine-readable output, and cheap repeated calls;
- coding agents need a local command runtime, not a pretend sandbox or a pile of ad hoc subprocess wrappers.

AUSH keeps the shell-shaped interface — commands, pipes, redirects, variables, scripts — but moves common operations into one Rust runtime that can expose structured values and eventually stay warm behind `aushd`.

## What is different from Bash/Zsh/Fish?

| If you want... | Use... | Why |
| --- | --- | --- |
| Maximum POSIX/Bash compatibility | Bash | It is the compatibility target. |
| A polished daily interactive shell | Zsh/Fish | Completion, themes, plugins, and terminal UX are much more mature. |
| Deterministic shell execution inside tools/tests/agents | AUSH | `--no-rc` and non-interactive modes avoid user startup-file drift. |
| Native file/text/Git/JSON operations without fork-per-step overhead | AUSH | Common operations can run in-process and emit structured output. |
| Shell pipelines that can grow toward typed/structured dataflow | AUSH | Selected builtins already support `--json`; structured operators are part of the design. |
| Low-latency repeated local command execution | AUSH | `aushd` is the experimental warm-runtime path. |

The bet is not that AUSH replaces Bash everywhere. The bet is that many developer and agent workflows use the shell as an execution substrate, and that substrate should be deterministic, inspectable, structured, and cheap to call repeatedly.

## What is it?

AUSH is a real local shell. It runs commands, pipelines, redirections, variables, functions, control flow, background jobs, and common shell builtins.

Its current differentiators are:

- **Deterministic command mode:** `aush --no-rc -c '...'` is the primary automation path and skips startup files.
- **Native hot-path commands:** `ls`, `cat`, `find`, `grep`, `head`, `tail`, `sort`, `wc`, file operations, and shell/session builtins can run inside the shell process.
- **Structured output:** selected native commands support `--json`, so scripts can inspect data without brittle text parsing.
- **Developer-workflow builtins:** Git, HTTP, JSON, and structured-output helpers are built as first-class shell commands behind the default feature set.
- **Agent-friendly errors:** error formatting can be made machine-readable for tools that need to classify failures.
- **Warm runtime direction:** `aushd` exists as experimental infrastructure for clients that need lower latency than repeated process startup.

## Status

AUSH `0.1.0` is a **public alpha**.

Good uses today:

- run focused shell commands with `aush --no-rc -c`;
- test shell compatibility and parser/runtime behavior;
- build automation or agent harnesses that prefer deterministic startup;
- experiment with native structured-output commands;
- try the interactive shell with a normal fallback shell available.

Use caution for:

- replacing your login shell;
- production scripts with obscure Bash/Zsh/POSIX edge cases;
- untrusted code — AUSH is not a sandbox;
- long-running interactive/job-control-heavy sessions.

Current local release gate for `0.1.0` passed:

```text
cargo test --quiet --tests --no-fail-fast     # passed
cargo test --quiet --lib -- --test-threads=1 # 1034 passed / 0 failed / 8 ignored
cargo test --quiet --bins -- --test-threads=1# passed
cargo build --release --bins                 # passed
tests/smoke_test.sh ./target/release/aush    # 120 passed / 0 failed
cargo publish --dry-run                      # packaged and verified aush v0.1.0
```

## Install

### Cargo

```sh
cargo install aush
```

This installs:

- `aush` — the shell CLI;
- `aushd` — experimental daemon/server binary.

### Homebrew

```sh
brew tap kfcafe/aush https://github.com/teh33/aush
brew install aush
```

### Build from source

```sh
git clone https://github.com/teh33/aush.git
cd aush
cargo install --path .
```

Or build release binaries in-place:

```sh
cargo build --release --bins
./target/release/aush --no-rc -c 'echo hello from aush'
```

## Quick start

```sh
# Use --no-rc for deterministic automation
aush --no-rc -c 'echo hello'

# Pipelines, redirections, and exit codes
aush --no-rc -c 'printf "one\ntwo\nthree\n" | tail -n 2'
aush --no-rc -c 'echo log line >> /tmp/aush-example.log'
aush --no-rc -c 'grep -q needle README.md'
echo $?

# Native file/text builtins
aush --no-rc -c 'find . -name "*.rs" -print -quit'
aush --no-rc -c 'ls -d src'
aush --no-rc -c 'grep -c "TODO" README.md'

# Structured output for tools/scripts
aush --no-rc -c 'find . --json -name "*.rs"'
aush --no-rc -c 'grep --json "CommandNotFound" src'
aush --no-rc -c 'ls --json src'

# Interactive shell
aush
```

## Programming model

AUSH is meant to be used in three modes:

1. **One-shot command runner** — `aush --no-rc -c '...'` for deterministic local command execution.
2. **Script/runtime shell** — familiar shell syntax plus native structured commands for developer automation.
3. **Warm local service** — experimental `aushd` for clients that want shell semantics without paying full startup every call.

That makes it closer to “a local automation runtime with shell syntax” than to “another login shell with a prompt theme.” Interactive use matters, but it is not the reason the project exists.

## Security model

AUSH is not sandboxed. It executes native builtins and external programs on the host machine with the permissions of the current user.

Important facts:

- redirections and file builtins can read, write, overwrite, and delete host files;
- external commands run as real child processes;
- AUSH does not virtualize the filesystem, network, process table, or OS;
- startup files (`~/.aush_profile`, `~/.aushrc`) can run arbitrary commands;
- `aushd` is experimental and should only be exposed to trusted local clients;
- there are no default global execution limits for arbitrary scripts yet.

Use OS-level controls such as `timeout`, separate users, containers, or VMs for untrusted code. See [docs/sandboxing.md](docs/sandboxing.md) for the current guardrail design direction.

## Shell behavior

### Startup files

AUSH keeps command-runner startup deterministic:

- `aush --no-rc -c '...'` skips startup files;
- `aush -c '...'` also skips login/rc files;
- stdin/scripted non-interactive use skips startup files;
- interactive `aush` sources `~/.aushrc`;
- interactive `aush --login` sources `~/.aush_profile`, then `~/.aushrc`;
- `--no-rc` disables both startup files for interactive sessions.

AUSH does not source `/etc/profile`, `~/.profile`, or Bash/Zsh startup files unless you explicitly source them.

For a terminal-specific trial, prefer configuring your terminal to run `aush --login` before changing your system login shell. Keep `/bin/zsh` or `/bin/bash` as a rollback path.

### Supported surface

| Area | Status | Notes |
| --- | --- | --- |
| `aush --no-rc -c` | Supported | Best-supported automation path. |
| stdin/script execution | Supported | Startup files are skipped. |
| Interactive shell | Alpha | Usable, still under terminal/job-control hardening. |
| Login shell mode | Alpha | `~/.aush_profile` then `~/.aushrc`; keep rollback shell. |
| Pipelines/redirections/heredocs | Supported | Common forms covered. |
| Variables/substitution/arithmetic | Supported/partial | Common forms covered; obscure expansion corners remain. |
| Functions/control flow | Supported/partial | `if`, loops, `case`, functions, `break`/`continue` covered by tests. |
| Job control/signals | Alpha | Core behavior exists; platform edge cases remain. |
| Native builtins | Supported/partial | Common flags covered; missing parity is tracked incrementally. |
| Structured output/operators | Experimental | Useful now; API may change. |
| `aushd` daemon | Experimental | Not a stable public protocol yet. |
| POSIX/Bash compatibility | Ongoing | Regression-tested, not certified. |

## Builtins

Native builtins include:

- shell/session: `cd`, `pwd`, `echo`, `printf`, `read`, `mapfile`, `export`, `unset`, `readonly`, `local`, `set`, `shopt`, `source`, `.`, `eval`, `exec`, `exit`, `umask`, `history`, `fc`, `hash`, `type`, `command`, `builtin`, `help`;
- tests/control: `test`, `[`, `true`, `false`, `:`, `return`, `shift`, `break`, `continue`, `getopts`, `time`, `times`;
- jobs/signals: `jobs`, `fg`, `bg`, `disown`, `kill`, `wait`, `trap`, `ulimit`;
- directories/files: `ls`, `pwd`, `cd`, `dirs`, `pushd`, `popd`, `cat`, `mkdir`, `rm`, `cp`, `mv`, `chmod`, `chown`, `ln`, `touch`, `readlink`, `stat`, `du`, `find`, `mktemp`;
- text/data: `grep`, `head`, `tail`, `wc`, `sort`, `uniq`, `cut`, `tr`, `tee`, `contains`, `count`, `math`, `path`;
- IO/network/structured helpers: `fetch`, `json_get`, `json_set`, `json_query`, `write`, `edit`, `profile`, `status`;
- compatibility stubs/helpers: `let`, `bind`, `suspend`, `enable`, `select`, `coproc`, `newgrp`, `logout`;
- Git: `git` dispatches to native Git builtins when built with `git-builtins`, otherwise to external `git`.

Native commands are ordinary shell commands from the user’s perspective; they avoid fork/exec when AUSH can handle the operation itself. Some builtins are compatibility-focused and do not yet implement every Bash/POSIX/GNU option.

## Structured output

Selected native commands support `--json`:

```sh
aush --no-rc -c 'find . --json -name "*.rs"'
aush --no-rc -c 'grep --json "CommandNotFound" src'
aush --no-rc -c 'ls --json src'
```

Structured pipeline operators such as `where`, `select`, `sort`, and `count` are part of the project direction. Treat that API as experimental in `0.1.x`.

## Compatibility

AUSH is Unix-shell-shaped, not Bash-perfect.

Recent compatibility work includes:

- command-not-found exit status `127`;
- `grep -q`, `grep -c`, `grep -l`, and `grep -L`;
- `find -print` and `find -quit`;
- `ls -d`;
- `umask`;
- GNU-style non-interactive `rm -r` behavior;
- `tail -f` / `tail --follow` for following appended file data;
- broader native builtin coverage for common file/text/session utilities;

Known gaps include some POSIX edge cases around case patterns, dynamic file descriptors, `read -d`, `cd -e`, `set -u`, traps, here-strings, arithmetic extensions, and platform-specific job-control behavior.

Current POSIX regression signal from the local suite:

```text
cargo test --test posix_2024_compliance --quiet
146 passed / 3 failed / 31 ignored / 180 total
```

That is a regression benchmark, not POSIX certification.

## Daemon mode

`aushd` is for resident clients that want warm shell/runtime state instead of launching a fresh process per command.

Current interpretation:

- direct daemon protocol calls can complete in hundreds of microseconds for simple commands;
- one-shot `aush -c` still pays process startup and is measured in milliseconds;
- the daemon protocol is experimental and intended for local trusted clients.

## Development

```sh
cargo fmt --check
cargo test --quiet --tests --no-fail-fast
cargo test --quiet --lib -- --test-threads=1
cargo test --quiet --bins -- --test-threads=1
cargo build --release --bins
tests/smoke_test.sh ./target/release/aush
cargo publish --dry-run
```

Project layout:

```text
src/
  builtins/    native shell/file/text/developer commands
  daemon/      aushd protocol and server pieces
  executor/    command execution, pipelines, redirects
  parser/      shell parser
  runtime/     variables, cwd, environment state
  value/       structured value model
benches/       release smoke, compatibility workloads, Criterion benches
tests/         integration and compatibility tests
docs/          deeper references and design notes
examples/      sample shell scripts and usage patterns
```

Useful docs:

- [docs/README.md](docs/README.md)
- [docs/AI_AGENT_GUIDE.md](docs/AI_AGENT_GUIDE.md)
- [docs/AI_AGENT_JSON_REFERENCE.md](docs/AI_AGENT_JSON_REFERENCE.md)
- [benches/README.md](benches/README.md)
- [tests/posix/README.md](tests/posix/README.md)
- [examples/README.md](examples/README.md)

## License

Dual-licensed under either:

- MIT — see [LICENSE-MIT](LICENSE-MIT)
- Apache-2.0 — see [LICENSE-APACHE](LICENSE-APACHE)
