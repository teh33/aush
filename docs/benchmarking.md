# AUSH Benchmarking Guide

## Overview

AUSH has multiple benchmarking approaches, each measuring different aspects of performance.

AUSH now distinguishes between:
- **performance benchmarks** — startup cost, builtin speed, script throughput
- **usability benchmarks** — whether the shell is actually pleasant and reliable as a daily driver

This second class matters if the project leans into the working name **AUSH**: **Actually Usable Shell**.

## **TL;DR: AUSH is 3.39x faster than Zsh** 🚀

When measuring **real-world script execution** (startup happens once), AUSH significantly outperforms Zsh due to built-in commands and zero-copy pipelines.

## Benchmark Scripts

### 1. Script-Based Benchmark (`script-bench-simple.sh`) ✅ RECOMMENDED

**Best for:** Real-world script performance comparison

```bash
cd benchmarks && bash ./script-bench-simple.sh
```

**Results:**
- **AUSH: 100ms total**
- **Zsh: 339ms total**  
- **AUSH is 3.39x faster** (239% improvement)

**Why AUSH wins:**
- Built-in cat, grep, ls (no process spawning)
- Memory-mapped I/O for file operations
- Zero-copy in-memory pipelines

**Key insight:** This is the fair comparison. Startup happens once per script, then AUSH's builtins dominate.

### 2. Fair Comparison (`compare-fair.sh`)

**Best for:** Understanding startup overhead impact

```bash
cd benchmarks && bash ./compare-fair.sh
```

**What it does:**
- Measures AUSH vs Zsh with realistic startup overhead
- Uses AUSH's `-c` flag for non-interactive execution
- Runs 10 iterations per test
- Shows per-test and total comparison
- Includes analysis of startup overhead impact

**Key insight:** Shows that AUSH has ~4-5ms startup overhead per `-c` invocation, but this is amortized in interactive sessions.

### 3. Criterion Benchmarks (`cargo bench`)

**Best for:** Pure builtin performance without startup overhead

```bash
cargo bench --bench shell_comparison
```

**What it does:**
- Measures AUSH builtins programmatically (no shell startup)
- Uses statistical analysis (mean, std dev, outliers)
- Generates HTML reports in `target/criterion/`
- Compares against previous runs to detect regressions

**Results vs Zsh/External Commands:**

| Operation | AUSH (builtin) | Zsh (external) | AUSH Advantage |
|-----------|----------------|----------------|----------------|
| echo | 8.47 µs | 24.21 µs | 2.85x faster |
| pwd | 8.30 µs | 21.84 µs | 2.63x faster |
| cat (10K lines) | 10.01 µs | 1,644.53 µs | **164x faster** |
| grep | 11.83 µs | 1,226.02 µs | **103x faster** |
| ls (50 files) | 113.18 µs | 1,858.93 µs | **16x faster** |
| ls -la | 118.94 µs | 4,106.07 µs | **34x faster** |
| find (1000 files) | 8.83 µs | 6,722.95 µs | **761x faster** |

**Key insight:** AUSH's built-in commands avoid process spawning overhead (~1-7ms per command), making file operations 16-761x faster than external commands. This is why AUSH dominates in script benchmarks.

**Compare Zsh to Criterion:**
```bash
cd benchmarks && bash ./criterion-vs-zsh.sh
```

### 4. Usability / Session Benchmarks

**Best for:** Testing whether AUSH is actually usable as a persistent shell session, not just fast in one-off invocations

```bash
bash ./benches/interactive_benchmark.sh
bash ./benches/session_benchmark.sh
bash ./benches/aush_smoke_fast.sh ./target/release/aush
bash ./benches/aush_suite.sh ./target/release/aush
```

**What they do:**
- compare AUSH and Zsh inside a persistent session
- separate shell startup overhead from in-session command execution
- exercise practical commands like `pwd`, `ls`, `git status`, `cat`, pipes, and substitution

**Key insight:** A shell can lose trivial `-c` startup comparisons and still be the better daily-driver shell. These benchmarks help measure that.

### 5. Legacy Scripts (Not Recommended)

- `compare.sh` - Doesn't actually benchmark AUSH (pre -c flag)
- `compare-v2.sh` - Combines shell and Criterion, but less clear
- `compare-fixed.sh` - Works but superseded by `compare-fair.sh`

## Actually Usable Checklist

Use this alongside the performance numbers.

### Startup
- `hyperfine --warmup 5 './target/release/aush -c exit'`
- no blank first prompt / redraw regression in a real terminal

### Interactive reliability
- `Ctrl-C` and `Ctrl-D` behave cleanly
- prompt redraw/history/completion do not corrupt the session

### Shell correctness
- parser/executor lib tests pass
- loops, quoting, pipelines, redirects, substitution, and functions behave as expected

### Daily-driver workflows
- `bash ./benches/interactive_benchmark.sh`
- `bash ./benches/session_benchmark.sh`
- manual repo workflow smoke test: `pwd`, `ls`, `git status`, `git branch`, `cat README.md`, `echo test | cat`

### Trust / recovery
- bad commands return control to the prompt
- terminal stays usable after interrupts and abnormal session endings
- `bash ./benches/aush_smoke_fast.sh ./target/release/aush`
- `bash ./benches/aush_suite.sh ./target/release/aush`

A shell is actually usable when it clears this checklist, not just when it wins microbenchmarks.

## Viewing Results

### HTML Reports (Criterion)

After running `cargo bench`:

```bash
open target/criterion/report/index.html
```

### Text Results

Benchmark results are saved in `benchmarks/benchmark-results/`:
- `zsh_TIMESTAMP.txt` - Zsh results
- `aush_TIMESTAMP.txt` - AUSH results (from aush-benchmark-v2.sh)

## Understanding the Numbers

### Three Different Measurements

**1. Script Execution (RECOMMENDED):**
- Measures: End-to-end script performance
- Startup: Once per script (realistic)
- Result: AUSH 3.39x faster than Zsh
- Why: Built-in commands eliminate process spawning

**2. `-c` Flag Invocations:**
- Measures: Single command with full startup
- Startup: Every invocation (worst case)
- Result: Zsh 2.26x faster than AUSH
- Why: AUSH's 5ms startup dominates short commands

**3. Pure Builtins (Criterion):**
- Measures: Builtin execution only (no startup)
- Startup: None (microbenchmark)
- Result: AUSH executes in microseconds
- Why: Shows raw builtin performance

### Startup Overhead

AUSH has ~4-5ms overhead per `-c` execution due to:
- Binary loading
- Rust runtime initialization
- Parsing and execution setup

**This overhead disappears in:**
- Interactive shell sessions (startup happens once)
- Script file execution (coming in Phase 4)
- Long-running commands

### Builtin Performance

When measuring pure builtin performance (Criterion benchmarks), AUSH often outperforms traditional shells on file-heavy operations due to:
- Memory-mapped I/O
- Zero-copy string handling
- Optimized Rust implementations

## Benchmark Philosophy

1. **Fair comparison** - Account for all real-world costs
2. **Honest reporting** - Don't hide startup overhead
3. **Context matters** - Different use cases favor different shells
4. **Continuous measurement** - Use Criterion to catch regressions

## Summary: When is AUSH Faster?

### ✅ AUSH Wins

**Script Execution: 3.39x faster overall**
- Startup once, run many commands
- Built-in cat, grep, ls, find eliminate process spawning
- Zero-copy in-memory pipelines

**Interactive Sessions: Same 3.39x advantage**  
- Startup once per terminal session
- Every file operation is 16-761x faster
- Pipelines are in-process (no kernel buffers)

**Pure Builtin Performance: 16-761x faster**
- cat: 164x faster than /bin/cat (10 µs vs 1,645 µs)
- grep: 103x faster than /bin/grep (12 µs vs 1,226 µs)
- find: 761x faster than /usr/bin/find (9 µs vs 6,723 µs)
- No process spawning overhead (~1-7ms per external command)

### ❌ AUSH Loses (2.26x slower)
- **Repeated `-c` invocations** - 5ms startup penalty each time
- **Single trivial commands** - `aush -c "echo hi"` (startup >> work)

### The Bottom Line

**AUSH's architecture is fundamentally faster for real shell work.**

The 16-761x builtin advantage compounds in scripts and interactive sessions. When you run a script with 20 cat/grep operations:
- Zsh: 20 × 1-7ms = 20-140ms in process spawning alone
- AUSH: 20 × 10µs = 0.2ms total (built-in function calls)

This is why AUSH achieves 3.39x speedup in real scripts despite having 5ms startup overhead.

The `-c` flag overhead only matters for contrived automation that repeatedly spawns AUSH. For actual shell usage, AUSH dominates.
