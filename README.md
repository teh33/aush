# AUSH

[![CI](https://github.com/kfcafe/aush/actions/workflows/integration-tests.yml/badge.svg)](https://github.com/kfcafe/aush/actions/workflows/integration-tests.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**Actually Usable Shell** — a Rust shell for Unix-style command execution, native builtins, structured output, and automation-heavy workflows.

AUSH aims to feel familiar where it should: commands, pipelines, redirections, variables, functions, control flow, jobs, and exit codes all follow the shape of a Unix shell. The difference is that common operations can run natively, and more commands can return structured data for programs and coding agents.

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

AUSH focuses on four practical goals:

1. **Shell compatibility where it matters** — ordinary shell syntax should keep working.
2. **Native hot paths** — common builtins avoid subprocess overhead when AUSH can handle the work directly.
3. **Structured output** — JSON and typed records reduce fragile stdout scraping.
4. **Agent-friendly behavior** — deterministic startup, useful exit codes, and clear failure modes for automation.

AUSH is not trying to replace every Bash or Zsh feature on day one. It is a pragmatic shell with modern implementation choices.

## Status

AUSH is **alpha software**. Use it for focused workflows, automation experiments,
and release-candidate testing, but keep a known-good shell such as `zsh` or
`bash` available for recovery. AUSH runs real commands on your real filesystem;
it is not a sandbox or virtual Bash environment.

Treat it as a shell you can experiment with, script against, and improve — not
yet as a guaranteed login-shell replacement for every Bash/Zsh edge case.

Good fits today:

- deterministic `aush --no-rc -c '...'` command execution;
- native file, text, and shell builtin workflows;
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

From crates.io, once published:

```bash
cargo install aush
```

From source:

```bash
git clone https://github.com/kfcafe/aush.git
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

## Security model

AUSH is a real local shell. It executes native builtins and external programs on
the host machine with the permissions of the current user.

Important release-safety facts:

- AUSH is **not sandboxed by default**.
- Redirections, file builtins, and external commands can read, write, overwrite,
  and delete host files that the user can access.
- External commands run as real child processes; AUSH does not virtualize the
  filesystem, network, process table, or operating system.
- Startup files (`~/.aush_profile`, `~/.aushrc`) are shell scripts and can run
  arbitrary commands when sourced.
- `aushd` is experimental daemon infrastructure for warm local clients. Only run
  it in environments where local clients with access to its socket are trusted.
- There are no built-in global execution limits for arbitrary scripts yet: no
  max command count, loop timeout, process cap, or memory cap is enforced by
  default. Use OS-level controls such as `timeout`, separate users, containers,
  or VMs for untrusted code.

AUSH does have internal effect/risk metadata for some builtins and a design path
for future guardrails. See [docs/sandboxing.md](docs/sandboxing.md). For this
alpha, assume commands have the same authority they would have in another local
Unix shell.

## Startup and login-shell semantics

AUSH currently has two distinct startup paths, and the difference is intentional:

- `aush -c '...'` is a fast, deterministic command runner. It skips login-shell and rc-file sourcing, even if you also pass `--login`.
- stdin/scripted non-interactive use such as `printf 'echo hi\n' | aush` also skips startup files.
- interactive startup sources `~/.aushrc` once.
- interactive startup with `--login` (or login-shell argv0 invocation like `-aush`) sources `~/.aush_profile` first, then `~/.aushrc`.
- `--no-rc` disables both `~/.aush_profile` and `~/.aushrc` for interactive startup.

Current startup file order is therefore:

1. `~/.aush_profile` for interactive login shells only
2. `~/.aushrc` for all interactive shells unless `--no-rc` is set

AUSH does **not** currently source `/etc/profile`, `~/.profile`, or zsh/bash startup files on its own. If you need shared environment setup, put AUSH-specific commands in `~/.aush_profile` or explicitly source another file from there.

If a sourced startup file contains an error, AUSH prints the file and line number to stderr and continues processing later lines. That makes rc failures actionable without leaving the shell unable to start.

### Recommended Ghostty setup

If you want to daily-drive AUSH in Ghostty without depending on zsh to bootstrap it first:

```sh
# Ghostty config
command = /absolute/path/to/aush
command-arg = --login
```

Recommended file split:

- put login/session environment setup in `~/.aush_profile`
- put aliases, prompt, and interactive shell tweaks in `~/.aushrc`

If startup goes sideways, rollback is just changing Ghostty back to your previous shell command, for example:

```sh
command = /bin/zsh
# or on Apple Silicon Homebrew setups:
# command = /opt/homebrew/bin/zsh
```

For one-off recovery from a terminal that should ignore config, start AUSH with:

```sh
aush --no-rc
```

### Trying AUSH as a login shell

For a public alpha, prefer a terminal-specific trial before changing your system
login shell. Ghostty is a good low-risk path because rollback is just editing its
config.

1. Install or build AUSH somewhere stable:

   ```sh
   cargo install aush
   # or from a checkout:
   cargo build --release --bin aush
   cp target/release/aush ~/bin/aush
   ```

2. Create AUSH-specific startup files. Keep them small at first:

   ```sh
   cat > ~/.aush_profile <<'EOF_PROFILE'
   # Login/session environment. Prefer idempotent PATH edits.
   export PATH="$HOME/.bun/bin:$HOME/bin:$HOME/.cargo/bin:$HOME/.local/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
   EOF_PROFILE

   cat > ~/.aushrc <<'EOF_RC'
   # Interactive-only tweaks go here.
   EOF_RC
   ```

3. Configure Ghostty to launch AUSH as a login shell:

   ```text
   command = /Users/YOU/bin/aush
   command-arg = --login
   ```

4. Open a fresh Ghostty window and verify your fallback path before doing real
   work:

   ```sh
   command -v aush
   aush --version
   echo "$PATH"
   command -v git
   command -v cargo
   command -v imp
   type -a imp
   ```

Rollback from this trial is immediate: set Ghostty back to `/bin/zsh` (or your
previous shell) and open a new window.

System-wide `chsh` is not recommended for first use. If you do choose it after a
successful terminal-specific trial, keep another terminal open and make sure AUSH
is listed in `/etc/shells`:

```sh
AUSH_PATH="$(command -v aush)"
grep -qxF "$AUSH_PATH" /etc/shells || echo "$AUSH_PATH" | sudo tee -a /etc/shells
chsh -s "$AUSH_PATH"
```

Rollback:

```sh
chsh -s /bin/zsh
# or, if your normal zsh is Homebrew-installed:
# chsh -s /opt/homebrew/bin/zsh
```


## Feature support matrix

| Area | Status | Notes |
| --- | --- | --- |
| `aush --no-rc -c` command execution | Supported | Best-supported automation path. |
| stdin/script execution | Supported | Non-interactive startup files are skipped. |
| Interactive shell | Alpha | Usable, but still under active terminal/job-control hardening. |
| Login shell mode | Alpha | `~/.aush_profile` then `~/.aushrc`; keep rollback shell configured. |
| Pipelines and common redirections | Supported | Includes pipes, heredocs, fd duplication, and common redirect forms. |
| Process substitution | Partial | Pragmatic temp-file-backed support for common cases; not full Bash `/dev/fd` parity. |
| Variables, command substitution, arithmetic | Supported/partial | Common forms covered; obscure Bash expansion corners remain. |
| Functions and control flow | Supported/partial | `if`, loops, `case`, functions, `break`/`continue` covered by tests; POSIX edge cases remain. |
| Job control and signals | Alpha | Core behavior exists; terminal/platform edge cases remain release-risk areas. |
| Native file/text builtins | Supported/partial | Common flags covered; missing flags should fail clearly or fall back where appropriate. |
| Structured JSON output/operators | Experimental | Useful for AUSH-native workflows; API/behavior may still change. |
| `aushd` daemon | Experimental | Intended for warm local clients and agent runtimes, not a stable public protocol yet. |
| POSIX/Bash compatibility | Ongoing | Tracked by Rust tests plus external shell behavior corpus runs; not formal certification. |

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
- text: `grep`, `head`, `tail`, `wc`, `sort`, and related structured operators where available;
- developer helpers: Git, HTTP, JSON, and structured-output commands behind the default feature set.

Native builtins are ordinary shell commands from the user’s perspective, but they avoid fork/exec when AUSH can handle the operation itself.

### Optional Lua scripting

AUSH has an experimental embedded Lua extension system for custom builtins,
hooks, prompts, and completions. It is intentionally **not enabled by default**
for the public alpha because it expands the dependency and execution surface.
Install or build with the `lua` feature to try it:

```bash
cargo install aush --features lua
# or from source:
cargo build --release --features lua --bins
```

When enabled, Lua scripts under `~/.aush/lua/*.lua` can run local code with the
same permissions as AUSH. Treat them like shell startup files: only load scripts
you trust.

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

## Compatibility

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

The external ShellSpec harness is also runnable through `bash tests/posix/run_tests.sh`. Current local result: **286 examples, 55 failures, 33 warnings, 5 skips**. Bats is supported by the runner, but this repository currently has no `.bats` files under `tests/posix/bats/`.

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

# Resource usage: wall time, CPU time, peak RSS, page faults, context switches
bash benches/resource_usage.sh ./target/release/aush
```

`benches/resource_usage.sh` uses macOS/BSD `/usr/bin/time -l`, writes raw logs under `${AUSH_BENCH_OUT_DIR:-/tmp/aush-resource}`, and compares installed shells when available (`bash`, `zsh`, `dash`). It intentionally measures one-shot CLI invocations. Use Criterion or hyperfine for latency; daemon protocol latency belongs in `aushd_compare.sh` and `daemon_latency.rs`.

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

Before publishing or cutting a release, run the bounded gate used by current
release candidates:

```bash
cargo fmt --check
cargo test --quiet --tests --no-fail-fast
cargo test --quiet --lib -- --test-threads=1
cargo test --quiet --bins -- --test-threads=1
cargo test --benches --no-run
cargo build --release --bins
bash benches/aush_smoke_fast.sh ./target/release/aush
bash benches/aush_suite.sh ./target/release/aush compat
cargo package --list
cargo publish --dry-run
```

If the worktree is intentionally dirty while iterating, `cargo publish --dry-run
--allow-dirty` can prove package verification, but the final release gate should
pass without `--allow-dirty` from a clean committed tree.

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

AUSH can be read as **Actually Usable Shell** or **Another Unix Shell**. The project started from a simple idea: make a shell that is pleasant to use from both humans and programs.

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
