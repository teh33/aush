# AUSH Performance Benchmarks

This document describes the performance targets, benchmarking methodology, and how to run benchmarks for the AUSH shell.

## The "Actually Usable" Standard

Raw speed is not enough. A shell can benchmark well and still fail as a daily driver.

AUSH therefore distinguishes between:
- **performance benchmarks** — startup, memory, parser, builtins
- **usability benchmarks** — interactive reliability, correctness under common workflows, and recovery behavior

If AUSH is ever published as **AUSH** (**Actually Usable Shell**), this second category is the bar that matters.

### Actually Usable categories

| Category | What it means | Example evidence |
|--------|--------|-----------|
| **Startup** | Feels instant enough to not be annoying | `aush -c exit`, interactive launch latency |
| **Interactive reliability** | Prompt, editing, signals, redraw, and session behavior are stable | prompt appears immediately, `Ctrl-C` recovers cleanly |
| **Shell correctness** | Common shell constructs behave like users expect | loops, quoting, pipelines, redirects, substitutions |
| **Daily-driver workflows** | Real repo/file/git tasks work smoothly in a persistent session | `pwd`, `ls`, `git status`, pipes, rc loading |
| **Trust / recovery** | Failure modes do not wedge the shell or terminal | bad commands, interrupted commands, PTY loss |

## Performance Targets

AUSH is designed to be fast and lightweight. Our key performance targets are:

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Startup Time** | < 10ms | Shell should feel instant |
| **Memory Usage** | < 10MB | Lightweight for embedded use |
| **Builtin Performance** | ≥ GNU utils | Builtins should not be slower than system commands |
| **Parser Latency** | < 1ms | Interactive commands should parse instantly |
| **Executor Init** | < 100μs | Minimal overhead for command execution |

## Benchmark Suite

AUSH includes two classes of benchmarks:

### 1. Performance Benchmarks

Located in `benches/`, these use the [Criterion](https://github.com/bheisler/criterion.rs) framework for detailed statistical analysis.

**Startup Benchmarks** (`benches/startup.rs`):
- Cold shell startup and exit
- Shell startup with simple command
- Lexer initialization and tokenization
- Parser initialization and AST creation
- Executor initialization
- Runtime initialization
- Memory footprint measurements

**Builtin Benchmarks** (`benches/builtins.rs`):
- Each builtin vs GNU equivalent comparison
- Builtin dispatch performance
- Argument scaling tests
- Initialization overhead

### 2. Usability Benchmarks

These focus on whether AUSH is usable as a daily shell, not just whether it is fast.

**Persistent-session benchmarks**:
- `benches/interactive_benchmark.sh` — compare commands in a persistent AUSH session vs Zsh
- `benches/session_benchmark.sh` — separate startup cost from in-session command execution

These are especially important for validating claims like **Actually Usable Shell** because they measure:
- prompt/session startup amortization
- repeated command execution in one shell session
- practical workflows like `pwd`, `ls`, `git status`, `cat`, pipes, and substitution

### 3. Hyperfine Real-World Benchmarks

Located in `scripts/benchmark.sh`, these compare AUSH against other shells (bash, zsh) in real-world scenarios using [hyperfine](https://github.com/sharkdp/hyperfine).

## Running Benchmarks

### Prerequisites

Install hyperfine for real-world benchmarks:
```bash
# macOS
brew install hyperfine

# Linux
cargo install hyperfine

# Or use your package manager
apt install hyperfine  # Debian/Ubuntu
```

### Quick Start

Run all benchmarks:
```bash
# Build optimized release binary
cargo build --release

# Run criterion microbenchmarks
cargo bench

# Run hyperfine real-world benchmarks
./scripts/benchmark.sh

# Run usability/session benchmarks
bash ./benches/interactive_benchmark.sh
bash ./benches/session_benchmark.sh

# Run the AUSH fast smoke benchmark
bash ./benches/aush_smoke_fast.sh ./target/release/aush

# Run the compatibility release gate
make bench-aush-compat
# or
bash ./benches/aush_suite.sh ./target/release/aush compat

# Generate bash/zsh performance comparison artifacts
make bench-aush-report
# or
bash ./benches/aush_suite.sh ./target/release/aush perf-report

# Run performance regression tracking checks
make bench-aush-regress
# or
bash ./benches/aush_suite.sh ./target/release/aush perf-regress

# Run the full AUSH benchmark suite
make bench-aush
# or
bash ./benches/aush_suite.sh ./target/release/aush full
```

### AUSH fast smoke benchmark
```bash
make bench-aush-fast
# or
bash ./benches/aush_smoke_fast.sh ./target/release/aush
```

This fast gate is intended to finish quickly and covers:
- startup smoke
- non-interactive pipe behavior
- login/no-rc behavior
- signal interruption behavior
- targeted core smoke coverage

### AUSH benchmark suite modes
```bash
make bench-aush-compat   # release gate
make bench-aush-report   # markdown/json bash+zsh performance artifacts
make bench-aush-regress  # Criterion regression checks
make bench-aush          # all modes
```

The suite separates three responsibilities:
- **compat** is the release gate. It runs fast smoke checks, POSIX-core tests, and agentic shell workload fixtures.
- **perf-report** is for human-facing reporting. It exports markdown and JSON comparisons under `reports/benchmarks/<timestamp>/`.
- **perf-regress** is for developer regression tracking using Criterion suites.

### AUSH full benchmark suite
```bash
make bench-aush
# or
bash ./benches/aush_suite.sh ./target/release/aush full
```

This full suite runs:
- compatibility gate checks
- bash/zsh performance report workloads
- Criterion regression workloads

### Individual Benchmark Suites

Run specific benchmark suites:
```bash
# Startup benchmarks only
cargo bench --bench startup

# Builtin benchmarks only
cargo bench --bench builtins

# Usability/session benchmarks
bash ./benches/interactive_benchmark.sh
bash ./benches/session_benchmark.sh

# Run specific benchmark function
cargo bench --bench startup bench_lexer_init
```

### Viewing Results

Criterion generates detailed HTML reports:
```bash
# Open the latest benchmark report
open target/criterion/report/index.html
```

Results include:
- Statistical analysis (mean, median, std dev)
- Performance regressions detection
- Historical comparison charts
- Detailed timing distributions

## Actually Usable Checklist

Use this as a release/readiness gate in addition to the speed numbers.

### 1. Startup
- `hyperfine --warmup 5 './target/release/aush -c exit'`
- interactive shell launches without blank prompt or redraw glitch
- startup remains competitive enough that the shell feels instant in Ghostty/iTerm

### 2. Interactive reliability
- prompt is visible immediately on first paint
- `Ctrl-C` cancels input without wedging the session
- `Ctrl-D` exits cleanly
- multiline input behaves predictably
- history/completion/search do not corrupt the prompt

### 3. Shell correctness
- lib test suite passes for parser/executor control flow
- common user-facing constructs verified:
  - quoting
  - variable expansion
  - command substitution
  - loops / conditionals
  - pipelines / redirects
  - functions

### 4. Daily-driver workflows
- run `bash ./benches/interactive_benchmark.sh`
- run `bash ./benches/session_benchmark.sh`
- run `bash ./benches/aush_smoke_fast.sh ./target/release/aush`
- run `bash ./benches/aush_suite.sh ./target/release/aush`
- manually validate in a real repo:
  - `pwd`
  - `ls`
  - `git status`
  - `git branch`
  - `cat README.md`
  - `echo test | cat`
  - rc/profile loading works in interactive/login mode

### 5. Trust / recovery
- invalid commands fail cleanly and return control to the prompt
- interrupted commands restore terminal usability
- PTY loss / closed terminal does not spin or wedge the process
- config-file errors degrade gracefully with useful messages

A shell is **actually usable** when it clears this checklist, not merely when it posts good microbenchmark numbers.

## Benchmark Configuration

### Criterion Settings

Benchmarks are configured with:
- **Warmup**: 3-5 runs to stabilize cache
- **Sample size**: 30-100 iterations depending on benchmark
- **Measurement time**: 5-10 seconds for statistical significance
- **HTML reports**: Enabled for detailed analysis

### Hyperfine Settings

Real-world benchmarks use:
- **Warmup**: 3-5 runs
- **Min runs**: 10-50 depending on variance
- **Markdown export**: For tracking trends

## Performance Profiling

For deeper performance analysis:

### Flamegraphs

Generate flamegraphs to identify hotspots:
```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph for startup
cargo flamegraph --bench startup

# Generate flamegraph for builtins
cargo flamegraph --bench builtins
```

### Instruments (macOS)

Use Xcode Instruments for detailed profiling:
```bash
# Install cargo-instruments
cargo install cargo-instruments

# Profile with Time Profiler
cargo instruments -t time --bench startup

# Profile with Allocations
cargo instruments -t alloc --bench builtins
```

### Valgrind (Linux)

Memory and cache analysis:
```bash
# Install valgrind and deps
sudo apt install valgrind

# Memory profiling
valgrind --tool=massif target/release/aush -c exit

# Cache profiling
valgrind --tool=cachegrind target/release/aush -c "echo test"
```

## Interpreting Results

### Startup Time

Target: **< 10ms**

Example output:
```
startup/cold_start_exit time: [8.234 ms 8.456 ms 8.678 ms]
```

If startup exceeds 10ms, investigate:
- Dependency initialization (reedline, tokio runtime)
- Module loading overhead
- Static initialization

### Memory Usage

Target: **< 10MB**

Check peak resident set size:
```bash
/usr/bin/time -l target/release/aush -c exit
# Look for "maximum resident set size"
```

If memory exceeds 10MB, investigate:
- Large static allocations
- Dependency memory overhead
- Inefficient data structures

### Builtin Performance

Target: **≥ GNU utilities**

Example comparison:
```
echo/aush_builtin   time: [1.234 μs 1.456 μs 1.678 μs]
echo/gnu_baseline   time: [2.234 μs 2.456 μs 2.678 μs]
```

AUSH builtins should be faster or comparable to system commands because:
- No process fork overhead
- No dynamic linking
- Direct function calls

## Continuous Benchmarking

### Before Committing

Run benchmarks before major changes:
```bash
# Establish baseline
cargo bench -- --save-baseline main

# Make changes...

# Compare against baseline
cargo bench -- --baseline main
```

Criterion will highlight regressions:
```
Performance has regressed
    startup/cold_start_exit
        time:   [8.456 ms 8.678 ms 8.901 ms]
        change: [+15.234% +18.456% +21.678%] (p = 0.00 < 0.05)
        Performance has regressed.
```

### CI Integration

Add to GitHub Actions:
```yaml
- name: Run benchmarks
  run: |
    cargo build --release
    cargo bench --no-fail-fast
```

## Benchmark Maintenance

### Adding New Benchmarks

When adding new features, add corresponding benchmarks:

1. **Microbenchmarks**: Add to appropriate bench file
   ```rust
   fn bench_new_feature(c: &mut Criterion) {
       c.bench_function("feature_name", |b| {
           b.iter(|| {
               // benchmark code
           });
       });
   }
   ```

2. **Real-world**: Add to `scripts/benchmark.sh`
   ```bash
   hyperfine \
       --warmup 3 \
       "$AUSH_BIN -c 'new command'" \
       "bash -c 'new command'"
   ```

### Benchmark Best Practices

1. **Use `black_box`** to prevent compiler optimizations from eliminating code
2. **Warmup properly** to account for cache effects
3. **Test realistic scenarios** not just synthetic cases
4. **Compare to baselines** (bash, zsh, GNU utils)
5. **Document expectations** in benchmark comments

## Current Performance Status

| Benchmark | Current | Target | Status |
|-----------|---------|--------|--------|
| Startup Time | TBD | < 10ms | ⏳ Pending |
| Memory Usage | TBD | < 10MB | ⏳ Pending |
| Echo builtin | TBD | ≤ GNU | ⏳ Pending |
| PWD builtin | TBD | ≤ GNU | ⏳ Pending |
| CD builtin | TBD | ≤ GNU | ⏳ Pending |
| Parser latency | TBD | < 1ms | ⏳ Pending |

*Run benchmarks and update this table with actual results*

## Performance Optimization Tips

### For Contributors

When working on performance improvements:

1. **Measure first**: Run benchmarks to establish baseline
2. **Target hotspots**: Use profiling to find bottlenecks
3. **Optimize iteratively**: Make small changes and measure
4. **Avoid premature optimization**: Focus on algorithmic improvements first
5. **Document tradeoffs**: Note any complexity vs performance decisions

### Common Optimizations

- **Reduce allocations**: Use stack allocation where possible
- **Minimize cloning**: Use references and `Cow` types
- **Efficient data structures**: Choose HashMap vs Vec appropriately
- **Lazy initialization**: Defer work until needed
- **Batch operations**: Process multiple items together
- **Cache results**: Memoize expensive computations

## Troubleshooting

### Benchmarks Won't Build

```bash
# Ensure release build exists
cargo build --release

# Check for missing dependencies
cargo check --benches
```

### Inconsistent Results

- Close other applications
- Run on consistent power settings
- Use `--warmup` to stabilize measurements
- Increase sample size for noisy benchmarks

### Hyperfine Not Found

```bash
# Install hyperfine
cargo install hyperfine

# Or use package manager
brew install hyperfine        # macOS
apt install hyperfine          # Debian/Ubuntu
pacman -S hyperfine           # Arch
```

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Hyperfine GitHub](https://github.com/sharkdp/hyperfine)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph](https://github.com/flamegraph-rs/flamegraph)

## Questions?

For questions about benchmarking or performance:
- Open an issue on GitHub
- Check existing benchmark results in CI
- Review the performance optimization guide in CONTRIBUTING.md
