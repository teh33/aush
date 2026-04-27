# AUSH Benchmarks and Smoke Gates

This directory contains several kinds of performance and release checks. They are intentionally separated because not every benchmark answers the same question.

## Which command should I run?

### Release smoke

Use these before publishing or after touching shell execution behavior:

```bash
cargo build --release --bins
bash benches/aush_smoke_fast.sh ./target/release/aush
bash benches/aush_suite.sh ./target/release/aush compat
```

- `aush_smoke_fast.sh` runs a small, fast release gate.
- `aush_suite.sh ... compat` runs the fast smoke plus compatibility workloads that model agent-style command sequences.
- These scripts should fail loudly when a command under test fails. They are gates, not statistical benchmarks.

### Compile benchmark targets

Use this after editing benchmark code or shared APIs:

```bash
cargo test --benches --no-run
```

This catches benchmark compile breaks without spending time on measurements.

### Criterion measurements

Use Criterion when you need statistically useful timing data:

```bash
cargo bench --bench startup
cargo bench --bench builtins
cargo bench --bench shell_comparison
cargo bench --bench ai_agent_workloads
cargo bench --bench daemon_latency
```

Criterion reports are written under `target/criterion/`.

### Daemon comparison

Use this when working on `aushd` protocol latency:

```bash
cargo build --release --bins
AUSH_BENCH_OUT_DIR=/tmp/aushd-compare \
  bash benches/aushd_compare.sh ./target/release/aush ./target/release/aushd
```

Interpretation:

- direct daemon protocol calls measure resident-client latency;
- one-shot `aush -c` measurements include process startup;
- those are different paths and should not be described as the same thing.

### POSIX regression score

AUSH has a Rust POSIX regression suite in addition to optional ShellSpec/Bats fixtures:

```bash
cargo test --test posix_2024_compliance --quiet
cargo test --test posix_compliance_tests --quiet
bash tests/posix/run_tests.sh
```

Current local results:

- `posix_2024_compliance`: 146 passed, 3 failed, 31 ignored, 180 total;
- executed POSIX 2024 pass rate: 146/149 = 98.0%;
- total POSIX 2024 coverage pass rate if ignored tests are counted as pending: 146/180 = 81.1%;
- `posix_compliance_tests`: 8 passed, 0 failed, 2 ignored;
- ShellSpec direct run from `tests/posix/shellspec`: 142 examples executed before aborting on a spec syntax issue; 110 pass, 16 fail, 16 warnings;
- Bats: installed locally, but the repository currently has 0 Bats test files under `tests/posix/bats/`.

Ignored Rust POSIX tests are explicit pending coverage, not hidden passes. They currently cover known gaps such as case bracket/fallthrough behavior, dynamic file descriptors, `read -d`, `cd -e`, break/continue, `exec` of builtins, exit/return propagation, `set -u`, EXIT traps, bracket test syntax, background jobs, here-strings, quoting edge cases, arithmetic ternary/increment, and wait/trap behavior.

Use the executed pass rate to track regressions in implemented POSIX behavior. Use the total-with-ignored number to track how much of the planned POSIX surface is covered and passing. Do not describe either number as formal POSIX certification.

## Benchmark map

| File | Purpose | Typical use |
| --- | --- | --- |
| `aush_smoke_fast.sh` | Fast release smoke gate | Run before commits/releases |
| `aush_suite.sh` | Orchestrates smoke/compat suites | Run `compat` before publishing |
| `agentic_compat.sh` | Agent-style compatibility workloads | Check automation behavior |
| `workloads/agentic_core.sh` | Core shell workload fixture | Used by compatibility suite |
| `ai_agent_workloads.rs` | Criterion benchmark for agent-like operations | Measure native command paths |
| `daemon_latency.rs` | Criterion daemon protocol benchmarks | Work on `aushd` latency |
| `aushd_compare.sh` | Scripted daemon vs CLI comparison | Quick before/after daemon check |
| `startup.rs` | Startup/initialization Criterion benches | Work on startup cost |
| `builtins.rs` | Builtin command Criterion benches | Work on native command speed |
| `shell_comparison.rs` | Criterion comparison across shells | Contextual performance work |
| `compare_bash.sh` | Scripted Bash comparison | Exploratory comparison |
| `shellbench.sh` | Broad shell behavior timing script | Exploratory shell comparison |
| `benchmark.sh` | POSIX-ish behavior benchmark script | Exploratory compatibility/perf |
| `quick_benchmark.sh` | Short local timing helper | Manual iteration |
| `profile_benchmark.sh` | Profiling helper | Hot-path investigation |
| `interactive_benchmark.sh` | Interactive/session timing helper | Manual UX/perf checks |
| `session_benchmark.sh` | Persistent session timing helper | Manual session checks |
| `claude_code_benchmark.py` | Python benchmark harness for coding-agent flows | Exploratory agent workload timing |

## Interpreting numbers

Avoid turning one local timing result into a permanent product claim. Prefer language like:

- “direct daemon protocol calls are in the hundreds of microseconds on this machine”;
- “one-shot `aush -c` includes process startup and is measured in milliseconds”;
- “native builtins avoid subprocess overhead for supported operations.”

When reporting numbers, include:

- hardware/OS if relevant;
- release/debug build;
- exact command;
- whether startup is included;
- whether the command succeeded.

## Recommended release gate

```bash
cargo fmt --check
cargo test --test command_tests --quiet
cargo test --benches --no-run
cargo build --release --bins
bash benches/aush_smoke_fast.sh ./target/release/aush
bash benches/aush_suite.sh ./target/release/aush compat
cargo package
```

## Adding or changing benchmarks

- Make benchmark commands fail loudly if the shell command fails.
- Do not silently time failed commands.
- Keep release gates deterministic and short.
- Keep statistical Criterion benchmarks separate from smoke tests.
- Use temp directories for filesystem workloads.
- Avoid depending on user-specific shell config; prefer `--no-rc`.
- Document what path is being measured: CLI startup, builtin execution, daemon protocol, or external command execution.

## Viewing Criterion reports

```bash
open target/criterion/report/index.html
```

Criterion output includes confidence intervals, outlier detection, and historical comparisons when baselines are saved.

```bash
cargo bench -- --save-baseline main
cargo bench -- --baseline main
```
