# AUSH Benchmark Scripts

This directory contains scripts for running performance benchmarks and comparisons.

## Available Scripts

### benchmark.sh

Comprehensive real-world performance comparison using [hyperfine](https://github.com/sharkdp/hyperfine).

**What it benchmarks:**
1. Shell startup time (AUSH vs bash vs zsh)
2. Echo command performance
3. PWD command performance
4. CD command performance
5. Multiple sequential commands
6. Environment variable export
7. Complex operations
8. Memory usage comparison

**Usage:**
```bash
# Make executable (first time only)
chmod +x scripts/benchmark.sh

# Run all benchmarks
./scripts/benchmark.sh

# Results are saved to results/ directory
```

**Prerequisites:**
```bash
# macOS
brew install hyperfine

# Linux
cargo install hyperfine
# OR
apt install hyperfine  # Debian/Ubuntu
```

**Output:**
- Terminal output with colored performance comparisons
- Markdown tables exported to `results/` directory
- Memory usage statistics

## Adding New Scripts

When adding new benchmark scripts:

1. Make them executable: `chmod +x scripts/your-script.sh`
2. Add shebang: `#!/usr/bin/env bash`
3. Include error handling: `set -euo pipefail`
4. Document in this README
5. Use consistent output formatting

## Tips

### Accurate Benchmarking

For most accurate results:
- Close other applications
- Disable background processes
- Run on consistent power settings (plugged in)
- Use `--warmup` for cache stability
- Increase `--min-runs` for noisy benchmarks

### Interpreting Results

hyperfine output format:
```
Benchmark 1: aush -c exit
  Time (mean ± σ):      8.5 ms ±   0.3 ms    [User: 2.1 ms, System: 4.2 ms]
  Range (min … max):    8.1 ms …   9.2 ms    50 runs
```

- **mean**: Average execution time
- **σ (sigma)**: Standard deviation (lower is more consistent)
- **Range**: Fastest to slowest run
- **Runs**: Number of benchmark iterations

### Comparing Results

When comparing AUSH vs other shells:
- **< 100%**: AUSH is faster (good!)
- **= 100%**: Same performance
- **> 100%**: Other shell is faster (needs optimization)

### Troubleshooting

**Command not found: hyperfine**
```bash
cargo install hyperfine
```

**Permission denied**
```bash
chmod +x scripts/benchmark.sh
```

**Inconsistent results**
- Increase warmup: `hyperfine --warmup 10 ...`
- Increase runs: `hyperfine --min-runs 100 ...`
- Close background apps
- Run multiple times and average

## See Also

- [BENCHMARKS.md](../BENCHMARKS.md) - Complete benchmarking documentation
- [benches/](../benches/) - Criterion microbenchmarks
- [Hyperfine README](https://github.com/sharkdp/hyperfine)
