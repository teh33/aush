# AUSH

[![CI](https://github.com/opus-workshop/aush/actions/workflows/integration-tests.yml/badge.svg)](https://github.com/opus-workshop/aush/actions/workflows/integration-tests.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**Another Unix Shell** — a Rust shell for Unix-style command execution, native builtins, structured output, and automation-heavy workflows.

AUSH is intentionally familiar: commands, pipelines, redirections, variables, functions, control flow, jobs, and exit codes should feel like a shell. The difference is that common operations are implemented natively and increasingly expose structured output for tools and coding agents.

```text
┌──────────────────────────── AUSH ────────────────────────────┐
│ $ aush --no-rc -c 'printf "one\ntwo\n" | tail -n 1'         │
│ two                                                          │
│                                                              │
│ $ aush --no-rc -c 'grep -q beta <<EOF                       │
│ alpha                                                        │
│ beta                                                         │
│ EOF'                                                         │
│ # exit status: 0                                             │
│                                                              │
│ $ aushd                                                      │
│ daemon protocol for warm, resident clients                   │
└──────────────────────────────────────────────────────────────┘
```

## Why AUSH

Traditional shells are excellent at composition, but they were not designed for programs that make hundreds of short shell calls and need reliable machine-readable results.

AUSH focuses on four things:

1. **Shell compatibility where it matters** — scripts should keep using ordinary shell syntax.
2. **Native builtins for hot paths** — avoid subprocess overhead for common commands.
3. **Structured output** — prefer JSON/typed records when commands are used by programs.
4. **Agent-friendly behavior** — deterministic startup, useful exit codes, and less stdout scraping.

AUSH is not trying to be a toy shell or a total reinvention of Unix. It is a practical shell with modern implementation choices.

## Status

AUSH is early but usable for focused workflows. Treat it as a shell you can experiment with, script against, and improve — not yet as a guaranteed login-shell replacement for every Bash/Zsh edge case.

Good fits today:

- `aush --no-rc -c '...'` command execution;
- native builtin and file/text workflows;
- shell compatibility testing;
- structured JSON-oriented automation;
- coding-agent harnesses and benchmarks;
- daemon protocol experiments.

Use caution for:

- replacing your daily login shell;
- production scripts that depend on obscure Bash/Zsh behavior;
- platform-specific job-control edge cases;
- long-running interactive terminal sessions.

## Install

```bash
cargo install aush
```

From source:

```bash
git clone https://github.com/opus-workshop/aush.git
cd aush
cargo install --path .
```

Build local release binaries:

```bash
cargo build --release --bins
./target/release/aush --no-rc -c 'echo hello from aush'
```

The package installs two binaries:

- `aush` — shell CLI;
- `aushd` — daemon/server binary for warm execution experiments.

## Quick start

```bash
# Run without startup files for deterministic automation
aush --no-rc -c 'echo hello'

# Pipelines and redirections
aush --no-rc -c 'printf "one\ntwo\nthree\n" | tail -n 2'
aush --no-rc -c 'echo log line >> /tmp/aush-example.log'

# Shell-style exit codes
aush --no-rc -c 'grep -q needle README.md'
echo $?

# Native file/text commands
aush --no-rc -c 'find . -name "*.rs" -print -quit'
aush --no-rc -c 'ls -d src'
aush --no-rc -c 'grep -c "TODO" README.md'
```

## Feature overview

### Shell language

AUSH implements a growing Unix shell language surface:

- simple commands and pipelines;
- `;` command sequencing;
- command substitution;
- arithmetic expansion;
- variables and environment mutation;
- functions;
- `if`/`elif`/`else`;
- `while`, `until`, and `for` loops;
- `case` patterns;
- here-docs and common redirections;
- background jobs and job-control builtins;
- shell-style exit status propagation.

### Builtins

AUSH includes native implementations for common shell and utility commands, including:

- shell/session: `cd`, `pwd`, `echo`, `printf`, `read`, `export`, `unset`, `source`, `eval`, `exec`, `exit`;
- tests/control: `test`, `[`, `true`, `false`, `return`, `shift`;
- jobs/signals: `jobs`, `fg`, `bg`, `kill`, `wait`, `trap`;
- files/directories: `ls`, `cat`, `mkdir`, `rm`, `cp`, `mv`, `chmod`, `readlink`, `find`;
- text: `grep`, `head`, `tail`, `wc`, `sort`, `uniq`-style structured operators where available;
- developer helpers: Git, HTTP, JSON, and structured-output commands behind the default feature set.

Native builtins are ordinary shell commands from the user’s perspective, but they avoid fork/exec when AUSH can handle the operation itself.

### Structured output

Several native commands support `--json` for programmatic access. That lets scripts and agents consume command results without parsing human-formatted text.

Examples:

```bash
# File search as JSON records
aush --no-rc -c 'find . --json -name "*.rs"'

# Grep matches as JSON records
aush --no-rc -c 'grep --json "CommandNotFound" src'

# Directory listing as JSON records
aush --no-rc -c 'ls --json src'
```

Structured pipeline operators such as `where`, `select`, `sort`, and `count` are part of the direction of the project. They are useful for AUSH-native data, and their compatibility surface is still being expanded.

### Coding-agent workflows

AUSH is designed for callers that need repeatable shell behavior:

- `--no-rc` skips startup config for deterministic tests;
- command-not-found uses shell-style exit code `127`;
- native JSON output reduces fragile text parsing;
- benchmarks and smoke tests fail loudly when commands fail;
- daemon protocol support is available for resident clients.

Python example:

```python
import subprocess

result = subprocess.run(
    ["aush", "--no-rc", "-c", "grep --json 'TODO' src"],
    text=True,
    capture_output=True,
)

if result.returncode == 0:
    print(result.stdout)
else:
    raise RuntimeError(result.stderr)
```

### Daemon mode

`aushd` provides a daemon path for clients that want to keep shell/runtime state warm instead of launching a fresh process for every command.

Current benchmark interpretation:

- direct daemon protocol calls can complete in hundreds of microseconds for simple commands;
- one-shot `aush -c` still pays process startup and is measured in milliseconds;
- daemon mode is most useful for resident clients, editor integrations, and agent runtimes that make many calls.

## Compatibility notes

AUSH is intended to be Unix-shell-shaped, not Bash-perfect on day one.

Recent compatibility work includes:

- `grep -q` quiet behavior;
- `grep -c`, `grep -l`, and `grep -L`;
- `tail -f` / `--follow` fails loudly instead of being silently ignored;
- GNU-style non-interactive `rm -r` behavior;
- `find -print` and `find -quit`;
- `ls -d`;
- command-not-found exit status `127`.

Known limitations:

- some POSIX edge cases remain under active development;
- Bash/Zsh-specific extensions are not all implemented;
- interactive login-shell use is experimental;
- long-running follow/watch-style commands are conservative unless explicitly supported.

### POSIX regression coverage

AUSH tracks POSIX shell behavior with Rust integration tests plus optional external harnesses.

Current local POSIX 2024 regression result:

```text
cargo test --test posix_2024_compliance --quiet
146 passed / 3 failed / 31 ignored / 180 total
```

That is **98.0% passing among executed tests** (`146/149`) and **81.1% passing if ignored/pending tests are counted in the total** (`146/180`). This is a project regression benchmark, not formal POSIX certification.

The older POSIX compliance smoke test also passes its executed cases:

```text
cargo test --test posix_compliance_tests --quiet
8 passed / 0 failed / 2 ignored
```

The ShellSpec/Bats harness under `tests/posix/` is optional. With ShellSpec installed, the current direct ShellSpec run executes 142 examples before aborting on a spec syntax issue: **110 pass / 16 fail / 16 warnings**. Bats is installed locally, but the repository currently has **0 Bats test files** under `tests/posix/bats/`.

The ignored Rust tests are explicit pending coverage for known gaps such as case bracket/fallthrough behavior, dynamic file descriptors, `read -d`, `cd -e`, break/continue semantics, `exec` of builtins, exit/return propagation, `set -u`, EXIT traps, bracket test syntax, background jobs, here-strings, quoting edge cases, arithmetic ternary/increment, and wait/trap behavior.

## Benchmarks and smoke tests

Benchmarking in AUSH is split by purpose:

- **smoke gates** check whether the shell works at all for release-critical behavior;
- **compatibility workloads** exercise agent-style command sequences;
- **Criterion benches** measure startup, builtins, shell comparison, and daemon latency;
- **ad hoc scripts** compare against other shells or profile specific scenarios.

Common commands:

```bash
# Build optimized binaries
cargo build --release --bins

# Fast release smoke
bash benches/aush_smoke_fast.sh ./target/release/aush

# Broader compatibility workload
bash benches/aush_suite.sh ./target/release/aush compat

# Compile benchmark targets without running long measurements
cargo test --benches --no-run

# Criterion benches
cargo bench --bench startup
cargo bench --bench builtins
cargo bench --bench daemon_latency
```

See [benches/README.md](benches/README.md) for the benchmark map and when to use each script.

## Development

```bash
# Format
cargo fmt --check

# Build
cargo build --bins

# Focused tests
cargo test --test command_tests
cargo test --test grep_integration_test
cargo test --test tail_builtin_tests

# Package verification
cargo package
```

Before publishing or cutting a release, run at least:

```bash
cargo fmt --check
cargo test --test command_tests --quiet
cargo test --benches --no-run
cargo build --release --bins
bash benches/aush_smoke_fast.sh ./target/release/aush
bash benches/aush_suite.sh ./target/release/aush compat
cargo package
```

## Project layout

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

## Documentation

- [docs/README.md](docs/README.md)
- [docs/AI_AGENT_GUIDE.md](docs/AI_AGENT_GUIDE.md)
- [docs/AI_AGENT_JSON_REFERENCE.md](docs/AI_AGENT_JSON_REFERENCE.md)
- [benches/README.md](benches/README.md)
- [tests/posix/README.md](tests/posix/README.md)
- [examples/README.md](examples/README.md)

## Name

AUSH can be read as **Another Unix Shell**. It started from a simpler idea: make a shell that is actually pleasant to use from both humans and programs.

## Contributing

Small compatibility improvements are valuable. Good first contributions include:

- adding a missing flag to a native builtin;
- adding a regression test for Bash/POSIX behavior;
- improving exit-code or stderr compatibility;
- making a benchmark fail loudly instead of silently measuring a failed command;
- documenting a known limitation with a reproducible example.

Please run the smallest relevant tests plus `cargo fmt --check` before sending changes.

## License

Dual-licensed under either:

- MIT — see [LICENSE-MIT](LICENSE-MIT)
- Apache-2.0 — see [LICENSE-APACHE](LICENSE-APACHE)
