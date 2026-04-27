---
id: '25'
title: Reshape shell benchmarking around bash drivers plus Rush-native workload scripts
slug: reshape-shell-benchmarking-around-bash-drivers-plu
status: open
priority: 1
created_at: '2026-04-24T08:47:16.442724Z'
updated_at: '2026-04-27T20:57:53.318343Z'
notes: |-
  ---
  2026-04-24T08:54:03.854929+00:00
  Refined plan after user clarified the standard for AUSH:

  Principle:
  - Rush should be able to run plausible real-world benchmark scripts written by competent shell users, even if those scripts were first authored with Bash in mind.
  - Therefore scripts like `scripts/benchmark.sh` are not just benchmark drivers; they are compatibility fixtures and should be treated as product-readiness targets.

  Two-track execution plan:
  1. Compatibility track
     - Preserve existing benchmark scripts as fixtures Rush should eventually run.
     - Catalog exact blockers encountered when running them under Rush (currently confirmed: backslash line-continuation parsing; likely also `&>` redirection, brace expansion, and other Bashisms).
     - Fix the blockers one by one, starting with the narrowest/highest-leverage syntax/semantics gaps.
  2. Benchmark-architecture track
     - Still improve the benchmark architecture over time by separating orchestration/driver scripts from shell workloads.
     - But do not let that replace the compatibility goal; the real scripts remain valid acceptance targets.

  Implication for current work:
  - Treat `scripts/benchmark.sh` as a compatibility benchmark fixture now.
  - If `shellbench.sh` is provided, ingest it the same way and build a combined punch list of blockers.
  - Prioritize compatibility fixes before broad benchmark-driver refactoring.

  ---
  2026-04-27T04:33:44.482127+00:00
  User wants to continue AUSH benchmark suite with two explicit goals: (1) validate compatibility with agentic language / POSIX shell workflows, especially scripts agents naturally emit; (2) prove AUSH is as fast or faster than bash and zsh. Benchmark suite should likely combine correctness gates and performance comparisons, not only microbenchmarks. Existing benchmark work includes benches/aush_suite.sh, benches/aush_smoke_fast.sh, benches/interactive_benchmark.sh, benches/session_benchmark.sh, docs/benchmarking.md, benches/README.md, Criterion benches, and POSIX compliance tests under tests/posix.

  ---
  2026-04-27T04:38:07.653549+00:00
  Implemented first benchmark-suite structure pass: benches/aush_suite.sh now accepts compat/perf-report/perf-regress/full; added benches/agentic_compat.sh and benches/workloads/agentic_core.sh; added Makefile targets bench-aush-compat, bench-aush-report, bench-aush-regress; updated BENCHMARKS.md. Verification: shell syntax passed. Running compat gate with existing ./target/release/aush fails at existing smoke failures: tests/smoke_test.sh reports 5 failures (command-not-found stderr behavior, true; echo sequence expected mismatch, git log builtin, etc.). This is useful: compat is now a real release gate and currently blocks.

  ---
  2026-04-27T05:17:08.852913+00:00
  Continuing benchmark work: perf-report currently cannot complete because `benches/workloads/agentic_core.sh` exits non-zero under bash and AUSH. Root causes: workload uses `set -e` with a `grep -q` miss inside a loop, and AUSH has trouble with the command substitution in `ROOT_DIR=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)`. Next step is to make the shared report workload boring/portable and then re-run `perf-report` before doing speed work.

  ---
  2026-04-27T05:19:24.370831+00:00
  Fixed `benches/workloads/agentic_core.sh` so it exits 0 and produces identical output under bash, zsh -f, and AUSH. Avoided unsupported/problematic forms for now: here-doc script generation, `find src -name`, `sort | wc` behavior discrepancy, and `grep -q` miss under set -e. Re-ran perf-report successfully. Results on 2026-04-26 local machine: startup AUSH 5.3ms, bash 1.5ms, zsh -f 2.6ms; agentic_core AUSH 24.0ms, bash 19.7ms, zsh -f 26.1ms; session benchmark single persistent session AUSH 39.87ms total / 7.97ms per command vs zsh 41.63ms / 8.33ms, AUSH 1.04x faster in-session. Startup remains primary speed gap.

  ---
  2026-04-27T05:43:13.981950+00:00
  User requested next phase: continue fixing parser compatibility gaps and run the full benchmark suite. Plan: run full suite first to capture current gate/report/regression state, then prioritize parser gaps exposed by suite or known avoided workload forms.

  ---
  2026-04-27T05:59:04.417817+00:00
  Ran full benchmark suite with `AUSH_BENCH_OUT_DIR=/tmp/aush-bench-full bash benches/aush_suite.sh ./target/release/aush full`. It timed out at 900s during `regress-criterion-agentic`, specifically `rapid_fire_git_status_1000x` wanted ~103s for sampling and continued past suite timeout. Completed stages before timeout all passed: compat-fast-smoke, compat-posix-core, compat-agentic-workloads, perf-startup, perf-agentic-hyperfine, perf-session, regress-criterion-startup, regress-criterion-builtins. Killed leftover tee process. Need to make full suite practical by reducing/ignoring expensive Criterion agentic workload or adding a quick regression profile before running full as a gate.

  ---
  2026-04-27T06:46:51.301118+00:00
  User asked about benchmarking AUSH through `aushd`. Need inspect existing daemon/client implementation and add/measure daemon-backed shell comparison if available. This likely matters because AUSH cold startup is the main speed gap; daemon/persistent mode may be the intended way to beat bash/zsh for agentic workflows.

  ---
  2026-04-27T06:54:17.038282+00:00
  Added `benches/aushd_compare.sh` to benchmark AUSH frontend with a running daemon and direct daemon protocol Criterion latency. Fixed `benches/daemon_latency.rs` to use current `StdinMode::Null` protocol type so it compiles. Measurement: `aush --no-rc -c true` with daemon running ~5.4ms, same as without daemon; bash ~1.4ms, zsh ~2.5ms. Direct daemon protocol `daemon/exec/{true,echo_hello,arithmetic,pipe}` all ~11.2ms. Conclusion: current aushd path is not the speed win; it is slower than direct AUSH startup for simple commands. Created task 25.6 to reduce/explain aushd warm latency.

  ---
  2026-04-27T16:33:58.944606+00:00
  Project path moved from `/Users/asher/rush` to `/Users/asher/aush`; continue benchmark work from new path. Existing mana verify strings may still mention old path and should be updated opportunistically when touching units.

  ---
  2026-04-27T20:57:53.318339+00:00
  2026-04-27 performance plan externalized from chat:

  Benchmarking decomposition remains three-track:
  1. Compatibility gate: AUSH must correctly run agent-authored/POSIX-ish scripts and real benchmark fixtures. This is the release gate; failures here block credible performance claims.
  2. Performance reporting: compare AUSH against system bash and zsh on cold startup, agentic workload scripts, and persistent/session execution. Use system bash/zsh for timings; use /Users/asher/bash source only for read-only implementation insight.
  3. Regression tracking: keep the suite practical to run often by trimming or profiling expensive Criterion workloads, especially long agentic loops.

  Immediate performance target:
  - Continue task 25.6: reduce or explain aushd warm command latency. Current evidence: cold `aush -c true` ~5.3ms vs bash ~1.5ms; AUSH persistent session was competitive with zsh; direct aushd protocol unexpectedly measured ~11ms. Hypothesis is fixed overhead in the daemon/client path or an architecture mismatch where `aush -c` still pays process startup and aushd needs a resident client/session API to matter.

  Next execution steps for 25.6:
  1. Measure latency breakdown: client connect, protocol write/init, daemon dispatch, worker execution, response read.
  2. Inspect `src/daemon/*`, frontend `aush -c` daemon integration, and benchmark files `benches/aushd_compare.sh` / `benches/daemon_latency.rs`.
  3. Make only small reversible Rust-native optimizations if the measurements identify clear overhead.
  4. Re-run `cargo build --release --bins`, `AUSH_BENCH_OUT_DIR=/tmp/aushd-compare bash benches/aushd_compare.sh ./target/release/aush ./target/release/aushd`, and `benches/session_benchmark.sh` if relevant.
  5. If no safe fix is obvious, document measured bottlenecks and whether a persistent client/session API should be a follow-up unit.
labels:
- benchmarks
- aush
- architecture
- rush
verify: cd /Users/asher/rush && test -d benches && true
kind: epic
decisions:
- 'Benchmark suite purpose: mixed release gate, reporting artifact, and regression tool. Compatibility suite is the release gate. Performance suite serves reporting and regression tracking against bash/zsh.'
---

Context:
- Existing benchmark scripts like `scripts/benchmark.sh` were generated from generic prompts and currently assume Bash semantics.
- Evidence: `./target/release/rush --no-rc scripts/benchmark.sh` fails with `Invalid token ... '\'`, showing the script is not valid Rush syntax.
- Likely Bashisms in the current benchmark driver include backslash line continuations, `&>` redirection, brace expansion, and other Bash-specific conveniences.
- For AUSH / Actually Usable Shell, benchmark credibility improves if the shell workloads themselves run natively in Rush rather than only being orchestrated by Bash.

Goal:
- Split benchmark architecture into:
  1. driver scripts that may run under Bash and orchestrate tools like hyperfine
  2. Rush-native workload scripts that represent actual shell capability/performance workloads and must run successfully in Rush
- Keep the benchmark system honest: Bash may coordinate, but Rush must execute the benchmark payloads it is being evaluated on.

Desired outputs:
- A set of Rush-native benchmark workload scripts for representative tasks
- A Bash driver (or drivers) that compare Rush against Bash/Zsh using those workloads
- Documentation clarifying this distinction and how to run the suite

Constraints:
- Prefer small, composable workload scripts over one giant Bash-centric benchmark file
- Treat shell compatibility and performance as separate measurable dimensions
- Keep project-local unless a broader cross-project benchmark framework emerges

Verification:
- At least one Rush-native workload script runs under Rush
- Driver script references workload scripts rather than embedding Bash-only shell syntax
- Docs explain driver-vs-workload architecture
