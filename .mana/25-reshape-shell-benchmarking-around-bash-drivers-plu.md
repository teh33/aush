---
id: '25'
title: Reshape shell benchmarking around bash drivers plus Rush-native workload scripts
slug: reshape-shell-benchmarking-around-bash-drivers-plu
status: open
priority: 1
created_at: '2026-04-24T08:47:16.442724Z'
updated_at: '2026-04-27T04:38:07.653553Z'
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
