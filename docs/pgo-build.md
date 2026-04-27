# Profile-Guided Optimization (PGO) Build

PGO uses runtime profiling data to optimize branch prediction, code layout, and inlining decisions. This typically improves AUSH startup time by 10-20%.

## Quick Start

```bash
# One command does everything:
make pgo
```

## Prerequisites

PGO requires `llvm-profdata` from the Rust toolchain:

```bash
rustup component add llvm-tools-preview
```

Verify it's available:

```bash
find $(rustc --print sysroot) -name llvm-profdata
```

For benchmarking (optional but recommended):

```bash
cargo install hyperfine
```

## How It Works

The PGO build is a 4-step pipeline:

1. **Instrumented build** -- Compiles AUSH with profiling hooks that record which code paths execute and how often branches are taken.

2. **Profile collection** -- Runs representative workloads (startup, echo, builtins, pipelines) hundreds of times. Each run writes `.profraw` files to `/tmp/aush-pgo-data/`.

3. **Profile merging** -- `llvm-profdata merge` combines all `.profraw` files into a single `merged.profdata` file.

4. **Optimized build** -- Recompiles AUSH using the merged profile data. LLVM uses this to:
   - Lay out hot code paths contiguously (better instruction cache usage)
   - Optimize branch predictions based on actual taken/not-taken ratios
   - Make better inlining decisions for frequently-called functions
   - Optimize virtual call targets

## Makefile Targets

| Target | Description |
|---|---|
| `make pgo` | Full PGO pipeline (all 4 steps) |
| `make pgo-check` | Verify llvm-profdata is available |
| `make pgo-instrument` | Step 1: Instrumented build only |
| `make pgo-collect` | Step 2: Run profiling workloads |
| `make pgo-merge` | Step 3: Merge profile data |
| `make pgo-build` | Step 4: Optimized build only |
| `make bench-pgo` | Full before/after comparison |
| `make bench-start` | Benchmark current binary startup |
| `make build` | Standard release build (no PGO) |

## Benchmarking

### Before/After Comparison

The easiest way to see the PGO improvement:

```bash
make bench-pgo
```

This builds a baseline, runs PGO, and compares both with `hyperfine`.

### Manual Benchmarking

```bash
# Build standard release
make build

# Benchmark baseline
hyperfine --warmup 5 --runs 30 './target/release/aush -c exit'

# Build PGO
make pgo

# Benchmark PGO
hyperfine --warmup 5 --runs 30 './target/release/aush -c exit'
```

## Release Profile

AUSH's `Cargo.toml` release profile is already optimized:

```toml
[profile.release]
opt-level = 3       # Maximum optimization
lto = true          # Link-time optimization (cross-crate inlining)
codegen-units = 1   # Single codegen unit (better optimization, slower compile)
strip = true        # Strip debug symbols (smaller binary)
```

These settings apply to both standard and PGO builds. PGO adds an additional layer of optimization on top of these.

## Troubleshooting

### `llvm-profdata` not found

```bash
rustup component add llvm-tools-preview
```

If it's still not found, the binary may not be in PATH. The Makefile searches for it automatically within the Rust sysroot.

### Profile data warnings

During the PGO build step, you may see warnings like:

```
warning: no profile data available for function ...
```

This is normal -- it means some functions weren't exercised during profiling. They'll still be compiled with standard optimizations.

### Stale profile data

If you make significant code changes, re-run the full PGO pipeline:

```bash
make clean
make pgo
```

## CI Integration

For CI/CD, add PGO to your release workflow:

```yaml
- name: Install llvm-tools
  run: rustup component add llvm-tools-preview

- name: PGO build
  run: make pgo

- name: Package
  run: cp target/release/aush aush-$(uname -s)-$(uname -m)
```
