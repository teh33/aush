# AUSH Criterion Benchmarks

This directory contains microbenchmarks using the [Criterion](https://github.com/bheisler/criterion.rs) framework.

## Benchmark Files

### startup.rs
Benchmarks AUSH initialization and startup performance:
- **Cold startup**: Shell launch and immediate exit
- **Startup with command**: Launch, execute, exit
- **Lexer init**: Tokenization performance
- **Parser init**: AST construction performance
- **Executor init**: Execution engine initialization
- **Runtime init**: Runtime environment setup
- **Memory footprint**: Allocation measurements

**Target**: < 10ms startup time

### builtins.rs
Benchmarks builtin commands vs GNU equivalents:
- **Echo**: AUSH builtin vs system echo
- **PWD**: AUSH builtin vs system pwd
- **CD**: Directory change performance
- **Export**: Environment variable setting
- **Find**: File search (AUSH vs GNU find)
- **Dispatch**: Builtin lookup performance
- **Arg scaling**: Performance with varying argument counts

**Target**: Equal to or faster than GNU utilities

## Running Benchmarks

### All benchmarks
```bash
cargo bench
```

### Specific suite
```bash
cargo bench --bench startup
cargo bench --bench builtins
```

### Specific function
```bash
cargo bench --bench startup bench_lexer_init
cargo bench --bench builtins bench_echo_builtin
```

### With baseline comparison
```bash
# Save baseline
cargo bench -- --save-baseline main

# Make changes...

# Compare to baseline
cargo bench -- --baseline main
```

## Viewing Results

Criterion generates detailed HTML reports:

```bash
# Open report in browser
open target/criterion/report/index.html

# Or navigate to specific benchmark
open target/criterion/startup/cold_start_exit/report/index.html
```

Reports include:
- Statistical analysis (mean, median, std dev)
- Performance regression detection
- Historical comparison charts
- Detailed timing distributions
- Violin plots showing data distribution

## Understanding Output

Terminal output format:
```
startup/cold_start_exit
                        time:   [8.234 ms 8.456 ms 8.678 ms]
                        change: [-5.123% -3.456% -1.789%] (p = 0.00 < 0.05)
                        Performance has improved.
```

- **time**: [lower_bound mean upper_bound] with 95% confidence
- **change**: Performance delta vs previous run
- **p-value**: Statistical significance (< 0.05 = significant)

## Benchmark Configuration

Current settings:
- **Warmup**: 3-5 runs to stabilize cache
- **Sample size**: 30-100 iterations
- **Measurement time**: 5-10 seconds
- **Confidence level**: 95%

Adjust in individual benchmarks:
```rust
group.warmup_time(Duration::from_secs(5));
group.measurement_time(Duration::from_secs(10));
group.sample_size(100);
```

## Adding New Benchmarks

1. Add function to appropriate file:
```rust
fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // Code to benchmark
            let result = my_function(black_box(input));
            black_box(result);
        });
    });
}
```

2. Add to criterion_group:
```rust
criterion_group!(
    startup_benches,
    bench_my_feature,  // Add here
    bench_other_feature
);
```

3. Always use `black_box()` to prevent optimization

## Best Practices

### DO
- ✅ Use `black_box()` for inputs and outputs
- ✅ Benchmark realistic scenarios
- ✅ Compare against baselines (bash, GNU utils)
- ✅ Run warmup iterations
- ✅ Document expected performance in comments

### DON'T
- ❌ Benchmark trivial operations
- ❌ Ignore variance in results
- ❌ Optimize based on single run
- ❌ Skip warmup in noisy benchmarks
- ❌ Forget to update baselines after improvements

## Performance Targets

| Component | Target | Current | Status |
|-----------|--------|---------|--------|
| Startup (cold) | < 10ms | TBD | ⏳ |
| Lexer tokenize | < 1ms | TBD | ⏳ |
| Parser (simple) | < 500μs | TBD | ⏳ |
| Executor init | < 100μs | TBD | ⏳ |
| Echo builtin | ≤ GNU | TBD | ⏳ |
| PWD builtin | ≤ GNU | TBD | ⏳ |
| Find builtin | ≤ GNU | TBD | ⏳ |

Run benchmarks and update this table!

## Profiling

For deeper analysis:

### Flamegraphs
```bash
cargo install flamegraph
cargo flamegraph --bench startup
```

### Instruments (macOS)
```bash
cargo install cargo-instruments
cargo instruments -t time --bench builtins
```

### Cachegrind (Linux)
```bash
valgrind --tool=cachegrind --cachegrind-out-file=cache.out \
    target/release/aush -c exit
cg_annotate cache.out
```

## Troubleshooting

### Noisy benchmarks
- Increase warmup and sample size
- Close background applications
- Run on consistent power settings

### "Benchmark took too long"
- Reduce sample size
- Decrease measurement time
- Check for infinite loops

### Inconsistent results
- Enable CPU frequency scaling
- Disable turbo boost for consistency
- Run multiple times and average

## See Also

- [BENCHMARKS.md](../BENCHMARKS.md) - Complete benchmarking guide
- [scripts/benchmark.sh](../scripts/benchmark.sh) - Real-world hyperfine benchmarks
- [Criterion Book](https://bheisler.github.io/criterion.rs/book/)
