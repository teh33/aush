---
id: '23'
title: Draft the Actually Usable benchmark suite and integrate it into Rush benchmark docs
slug: draft-the-actually-usable-benchmark-suite-and-inte
status: open
priority: 1
created_at: '2026-04-24T07:50:13.322350Z'
updated_at: '2026-04-24T08:36:21.590594Z'
acceptance: Repository contains an explicit 'Actually Usable' benchmark/checklist document and the existing benchmark docs/entrypoints are supplemented so they reference usability-oriented benchmarks alongside raw performance benchmarks. Changes are documentation and light benchmark entrypoint supplementation only unless a narrow script stub is clearly useful.
notes: |-
  ---
  2026-04-24T08:36:21.590336+00:00
  Built the first benchmark split:
  - Added `benches/aush_smoke_fast.sh` as a quick gate
  - Added `benches/aush_suite.sh` as the full suite wrapper
  - Added `make bench-aush-fast` and repointed `make bench-aush` to the full suite
  - Updated BENCHMARKS.md and docs/benchmarking.md accordingly

  Verification:
  - `make -n bench-aush-fast` and `make -n bench-aush` show expected commands
  - `bash ./benches/aush_smoke_fast.sh ./target/release/rush` completes and produces actionable results

  Current fast smoke outcome:
  - Passes startup smoke, non-interactive pipe, login/no-rc behavior, signal interruption behavior
  - Skips PTY-only checks (first prompt visibility, revoked PTY recovery)
  - Fails on the embedded core smoke suite due to real shell/product issues, not harness timeouts
    - missing command/exit-code behavior around nonexistent external commands
    - absolute path execution like `/bin/true` and `/bin/false`
    - `git log --oneline -1` builtin path in smoke suite

  This is now a functioning product-readiness gate rather than only benchmark plumbing.
labels:
- benchmarks
- docs
- aush
- usability
verify: cd /Users/asher/rush && rg -n 'Actually Usable|interactive reliability|daily-driver|trust/recovery' BENCHMARKS.md docs/benchmarking.md Makefile docs 2>/dev/null
verify_timeout: 60
kind: job
---

Draft an 'Actually Usable' benchmark suite for the shell and supplement the existing Rush benchmark materials.

Context:
- User is considering the publishable name AUSH = Actually Usable Shell.
- Existing benchmark coverage is strong on startup and micro/perf comparisons, but weaker on interactive usability, daily-driver reliability, and acceptance-style shell readiness.
- Goal is to define a benchmark/checklist that makes 'actually usable' measurable, then wire that framing into current benchmark docs and lightweight entrypoints.

Current benchmark assets inspected:
- BENCHMARKS.md focuses on startup, memory, builtins, parser, executor, Criterion, hyperfine.
- docs/benchmarking.md focuses on script speed, startup overhead, builtins vs zsh/external commands.
- Makefile has `bench-start` and PGO-related benchmark targets.
- benches/interactive_benchmark.sh and benches/session_benchmark.sh exist, but the benchmark docs do not elevate them as part of a formal usability suite.

Desired outcome:
1. Define categories for 'Actually Usable': startup, interactive reliability, shell correctness, daily-driver workflows, and trust/recovery.
2. Convert those into a benchmark/checklist doc with concrete commands and pass/fail expectations where possible.
3. Update BENCHMARKS.md and/or docs/benchmarking.md so the current benchmark philosophy clearly distinguishes microbenchmarks from usability benchmarks.
4. Optionally add a small Make target or documented entrypoint for running the usability/session benchmarks if that can be done narrowly.

Constraints:
- Keep scope small and docs-first.
- Do not invent results; define the suite and point to how to run it.
- Prefer supplementing existing assets over broad reorganization.

Verification:
- grep for 'Actually Usable' and usability benchmark references in the updated docs
- if adding a make target, ensure it is present in Makefile and points at an existing script/command
