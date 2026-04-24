---
id: '21'
title: Triage Rush failing nested-loop tests and dependency audit posture
slug: triage-rush-failing-nested-loop-tests-and-dependen
status: open
priority: 1
created_at: '2026-04-24T03:31:02.268513Z'
updated_at: '2026-04-24T07:20:04.573026Z'
acceptance: Documented root cause or likely cause for the two failing nested-loop executor tests, dependency inventory counts for the Rush Cargo graph, a triaged summary of the current OSV findings with likely remediation paths, and a rough comparative note on dependency footprints of bash/fish/zsh grounded in inspected sources or package manifests.
notes: |-
  ---
  2026-04-24T03:33:09.770160+00:00
  Progress notes:
  - Reproduced both failing tests directly. Current runtime output is `1a/1b/2a/2b` and `00/01/10/11`, while tests still expect old spaced behavior.
  - Inspected src/executor/mod.rs around the failing tests: both tests include inline TODO comments saying variable concatenation should eventually produce POSIX-style concatenation without inserted spaces. The implementation now appears to do that, so the failures are very likely stale test expectations rather than a new runtime bug.
  - Dependency inventory from current Rush repo:
    - 35 direct runtime deps from Cargo metadata
    - 2 dev deps
    - ~278 reachable packages excluding root from cargo metadata resolve graph
    - cargo tree unique package-name count came out ~193, but package-ID reachability count is the better number for lock-graph breadth because it includes multiple versions.
  - Vulnerable crates triaged with cargo tree -i:
    - atty: direct dep
    - bincode: direct dep
    - git2: direct dep
    - bytes: transitive via ureq -> ureq-proto -> http -> bytes
    - rustls-webpki: transitive via ureq -> rustls
  - Initial comparative research:
    - fish is now Rust-based and has a substantially larger dependency footprint than Rush. Grounded evidence gathered from upstream Cargo.toml/Cargo.lock: ~48 workspace deps, ~30 package deps, and Cargo.lock package count around 239.
    - zsh and bash are C shells with much smaller external library surfaces, but they rely on system libraries and optional features rather than Cargo-style package graphs. Evidence gathered from zsh configure.ac shows optional checks for pcre2, ncurses/termcap/curses, iconv, gdbm, dl, and POSIX capabilities. Bash traditionally depends mainly on libc/termcap-or-ncurses and readline/history (sometimes bundled/vendored in source).
  - No code changes made yet; analysis only.

  ---
  2026-04-24T04:01:21.537511+00:00
  Patched the two stale nested-loop executor tests in src/executor/mod.rs to assert current POSIX-style adjacent variable concatenation (`1a`/`00` forms) instead of the old spaced outputs. Verification:
  - `cargo test -q executor::tests::test_for_loop_nested --lib` passed
  - `cargo test -q executor::tests::test_while_nested --lib` passed
  - `cargo test -q --lib` passed with 1006 passed, 0 failed, 8 ignored
  Note: full test output still contains OSC 133 escape sequences in stderr from terminal-related code, but the lib suite passed.

  ---
  2026-04-24T04:03:18.474278+00:00
  Dependency remediation memo findings:
  - Source usage inspection:
    - `atty` is only used for terminal/TTY detection in `src/value/render.rs`, `src/executor/commands.rs`, `src/main.rs`, and related render paths. This is a low-risk replacement candidate with `std::io::IsTerminal` on modern Rust or another maintained approach.
    - `bincode` is used in `src/daemon/protocol.rs` for daemon message serialize/deserialize. Exposure appears local IPC rather than internet-facing, but it is still a deserialization boundary and worth remediation.
    - `git2` is used broadly across git builtins/completion (`src/git/mod.rs`, `src/builtins/git_*`, `src/completion/mod.rs`). Upgrading to a fixed major/minor may require code changes.
    - `ureq` is used in `fetch` builtin and AI providers; this is the network/TLS-facing path that pulls in `bytes` and `rustls-webpki`.
  - Updateability:
    - `cargo update -p bytes --dry-run` would move `1.11.0 -> 1.11.1`.
    - `cargo update -p rustls-webpki --dry-run` would move `0.103.10 -> 0.103.13`.
    - `git2`, `atty`, and `bincode` had no patch-level updates available under current version constraints in dry-run.
  - Recommended remediation order:
    1. Low-risk immediate: update lockfile for `bytes` and `rustls-webpki`.
    2. Medium-risk cleanup: replace direct `atty` usage with maintained std/alt API.
    3. Medium-risk protocol hardening: review whether daemon protocol can migrate off `bincode` or constrain/validate messages more tightly.
    4. Higher-risk but important: plan `git2` upgrade to a fixed compatible newer release (likely requires Cargo.toml bump and code/test pass).
  - Login-shell posture inference: after the stale tests were fixed, the largest remaining blockers are dependency hygiene rather than obvious correctness regressions.

  ---
  2026-04-24T06:36:35.655956+00:00
  Implemented low-risk dependency cleanup:
  - Removed direct `atty` dependency and replaced all current uses with `std::io::IsTerminal`-based checks.
    - main.rs: stdin interactive/login-shell checks
    - src/value/render.rs and src/executor/value/render.rs: stdout color gating
    - src/executor/commands.rs: stdout inherit/progress gating
  - Updated lockfile transitives:
    - `bytes` -> 1.11.1
    - `rustls-webpki` -> 0.103.13
  - Verification:
    - `cargo check -q` passed
    - targeted nested-loop lib tests still pass
    - cargo tree confirms `atty` is gone; `bytes` and `rustls-webpki` now resolve to the updated versions
  - One broad `cargo test -q --lib` attempt timed out due to package-cache/file-lock contention after dependency updates, but earlier full lib run was green after the stale-test patch and targeted tests still pass after this cleanup.

  ---
  2026-04-24T07:11:16.699673+00:00
  Daemon protocol hardening implemented:
  - Replaced `SessionInit.stdin_mode: String` with a typed `StdinMode` enum (`Inherit`, `Pipe`, `Null`) serialized with lowercase serde names.
  - Added semantic validation after bincode decode:
    - max working directory length 4096 bytes
    - max env vars 4096
    - max env key length 1024 bytes
    - max env value length 64KiB
    - max args 4096
    - max arg length 64KiB
    - max Execute command length 1MiB
    - max requested stats 128
    - max stat name length 256
    - ExecutionResult stdout_len/stderr_len must match actual byte lengths
    - StatsResponse map counts and string sizes are bounded
  - Updated client and protocol tests to use StdinMode::Inherit.
  - Adjusted stale protocol tests that used intentionally mismatched stdout_len/stderr_len values, because the new validation correctly rejects mismatched lengths.
  Verification:
  - `cargo check -q` passed
  - `cargo test -q daemon::protocol --lib` passed (25/25)
  - `cargo test -q executor::tests::test_for_loop_nested --lib` passed
  - Dependency status: atty gone; bytes=1.11.1; rustls-webpki=0.103.13; bincode and git2 remain.
  Notes:
  - `audit_scan deps` failed because the local Lua audit tool has imp.exec disabled in this runtime, not because the scan found issues.
  - Disk was briefly full during an earlier test attempt; after some system/cache movement by the environment, available disk was ~11GiB, but target remains 37GiB.

  ---
  2026-04-24T07:20:04.572979+00:00
  Git2 remediation completed:
  - Bumped `git2` from 0.19.0 to 0.20.4 and `libgit2-sys` from 0.17.0+1.8.1 to 0.18.3+1.9.2 via cargo update.
  - `cargo check -q` passed.
  - Targeted git builtins tests passed: `cargo test -q builtins::git --lib` -> 10 passed.
  - Targeted git module tests passed: `cargo test -q git:: --lib` -> 4 passed.
  - Broad `cargo test -q git_ --lib` had one unrelated flaky/environmental suggestion test failure (`executor::suggestions::tests::test_git_typo_in_git_repo`) where PATH suggestions did not include git; actual git builtins/module tests passed.
  - Full lib suite passed after upgrade: `cargo test -q --lib` -> 1006 passed, 0 failed, 8 ignored.
  - Direct OSV scan now reports only one remaining vulnerability: `bincode 1.3.3` / RUSTSEC-2025-0141, no fixed version. All previous atty/bytes/git2/rustls-webpki findings are gone.
  Disk cleanup note: user ran cargo clean and freed ~54GiB; df showed ~46GiB available before rebuild, then later ~? target was rebuilt enough for checks/tests.
labels:
- rush
- triage
- deps
- shell
verify: cd /Users/asher/rush && cargo tree --depth 1 >/dev/null && cargo test -q executor::tests::test_for_loop_nested || true
verify_timeout: 120
kind: job
---

Triage Rush shell readiness questions after Ghostty was pointed at the system-installed binary.

Current evidence:
- `cargo test -q` fails on two executor tests:
  - `executor::tests::test_for_loop_nested`
  - `executor::tests::test_while_nested`
  Output suggests nested loop bodies are concatenating variables without expected spaces (`1a` vs `1 a`, `00` vs `0 0`).
- Security source scan was clean, but OSV reported vulnerabilities in Cargo.lock affecting atty, bincode, bytes, git2, and rustls-webpki.
- User also wants context on dependency count relative to other shells like bash, fish, zsh.

Goals:
1. Inspect the failing tests and surrounding executor/parser behavior to determine whether the failure is a regression, a changed parsing rule, or stale tests.
2. Inventory Rush dependency counts from Cargo metadata/tree (direct + transitive if practical).
3. Triage each reported vulnerable dependency: direct vs transitive, likely reachable surface, fixed version if any, and probable next action.
4. Compare Rush dependency footprint to bash/fish/zsh using grounded external/package evidence where available.

Concrete steps:
- Read the failing test bodies and nearby executor code in src/executor/mod.rs.
- Reproduce the failing behavior narrowly if possible.
- Inspect parser/executor handling of words/arguments/loop bodies for adjacent-token concatenation and whitespace preservation.
- Use cargo metadata/tree to count direct and transitive deps.
- Use cargo tree -i / cargo tree to locate who pulls in the vulnerable crates.
- Use web/package-source research for dependency footprint of bash, fish, zsh; prefer repo/package manifests or build-system declarations over vague blog posts.
- Summarize evidence, inference, and recommendation separately.

Constraints:
- Analysis only unless a very small safe fix becomes obviously correct and requested later.
- Do not claim other-shell dependency counts without inspected evidence.

Verification:
- `cargo test -q executor::tests::test_for_loop_nested executor::tests::test_while_nested` or nearest valid targeted form
- `cargo tree --depth 1`
- `cargo tree -i atty -i bincode -i bytes -i git2 -i rustls-webpki` as applicable
