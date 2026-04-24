# AUSH Phase 1 Rename Audit

Generated: 2026-04-24
Repo: /Users/asher/rush
Goal: categorize current `rush` references into rename-now, alias-for-compatibility, and leave-internal-for-now for a migration-safe rebrand to AUSH (Actually Usable Shell).

## Summary

- Raw search hits: 4189
- Files with matches: 437
- Significant noise sources: `.mana/`, research artifacts, generated docs, target/build outputs
- High-signal implementation surfaces:
  - runtime compatibility: env vars, config paths, sockets, daemon dirs
  - user-facing brand strings in docs/help/banner
  - packaging/distribution names in Cargo/Homebrew/scripts/CI
  - tests and benches hardcoding binary/path names
  - pi-rush integration surfaces

## Category map

### 1) Rename now
These are the safest/highest-value first-pass changes because they mostly affect user perception and visible product identity.

#### User-facing docs and product copy
- `README.md`
- `QUICKSTART.md`
- `STATUS.md`
- `BENCHMARKS.md`
- `ARCHITECTURE.md` where branding is explanatory rather than protocol-specific
- `docs/INSTALLATION.md`
- `docs/DISTRIBUTION.md`
- `docs/PERFORMANCE.md`
- `docs/benchmarking.md`
- `docs/login-shell-init.md`
- `docs/error-recovery.md`
- `examples/README.md`
- most other docs pages that describe the shell by name

Rename guidance:
- Change visible product name `Rush` -> `AUSH`
- Expand first-use phrasing to `AUSH (Actually Usable Shell)` where helpful
- Preserve historical references only where discussing prior naming/history

#### In-product branding/help text
- `src/banner/mod.rs`
- `src/config/banner.rs`
- `src/main.rs` help/version/about strings
- any `println!`/`eprintln!`/usage text that calls the shell `Rush`

Rename guidance:
- visible banner text should say `aush` / `AUSH`
- help output and error/help copy should refer to AUSH
- if a compatibility alias remains, help can mention `rush` as a legacy alias during migration

#### Obvious repository-facing brand surfaces
- `homebrew/README.md`
- top-level shell/debug scripts names or comments, if kept user-facing
- release-facing markdown and installation instructions

## 2) Alias for compatibility
These should support both old and new names during the migration window. Renaming them outright first would break existing users and automation.

#### Environment variables
Examples found:
- `RUSH_BANNER_STYLE`
- `RUSH_BANNER_COLOR`
- `RUSH_BANNER_SHOW`
- `RUSH_BANNER_STATS`
- `RUSH_LEVEL`
- `RUSH_MAX_SUBST_OUTPUT`
- `RUSH_PI_SOCKET`
- `RUSH_PI_PATH`
- `RUSH_AGENT_MODE`

Primary files:
- `src/banner/mod.rs`
- `src/executor/expansion.rs`
- `src/daemon/pi_client.rs`
- `src/daemon/pi_rpc.rs`
- `src/runtime/mod.rs`

Migration rule:
- read `AUSH_*` first, then fall back to `RUSH_*` (or explicitly document the precedence)
- for shell-set values like nesting level, consider exporting both during transition if low-risk
- document deprecation rather than removing `RUSH_*` immediately

#### Config/runtime paths
Examples found:
- `~/.config/rush/universal_vars`
- `~/.rush`
- `~/.pi/rush.sock`
- `/tmp/pi-rush-$USER.sock`
- temp dirs like `rush-undo`

Primary files:
- `src/runtime/universal_vars.rs`
- `src/daemon/server.rs`
- `src/daemon/pi_client.rs`
- `src/runtime/mod.rs`

Migration rule:
- prefer new paths for fresh installs where safe
- read from both old and new paths during transition
- either migrate-on-read/startup or keep dual lookup with a clear precedence order
- do not strand existing data in `~/.config/rush` without a migration path

#### Invocation/binary compatibility
Examples found:
- `rush` hardcoded in scripts, docs, tests, Makefile, and Homebrew formula
- build outputs like `target/release/rush`

Migration rule:
- if binary becomes `aush`, keep a `rush` shim/symlink/compat install path for a transition period
- release docs should explicitly state compatibility story
- tests can gradually move to `aush` while keeping targeted legacy coverage for `rush`

#### Install/distribution compatibility
- Homebrew formula naming
- release tarball names
- GitHub release artifact names
- tap/repo references

Migration rule:
- likely needs a compatibility window and coordinated release change
- may require both formula names or a migration note depending on Homebrew constraints

## 3) Leave internal for now
These are low user value and high churn if renamed early.

#### Internal protocol/type names
Examples:
- `RushToPi`
- `PiToRush`
- `RushError`
- `rush-rpc-*` IDs
- `rush-*` request IDs

Primary files:
- `src/daemon/protocol.rs`
- `src/daemon/pi_client.rs`
- `src/daemon/pi_rpc.rs`
- `src/error.rs`
- internal references across executor/runtime

Reason to defer:
- broad Rust symbol churn
- little end-user value in phase 1/2
- higher chance of distracting breakage and review noise

#### Research/history/generated material
- `research/**`
- historical notes and comparison artifacts
- most `.mana/**`
- generated benchmark/report assets

Reason to defer:
- not part of shipping product surface
- can be updated selectively later if needed for presentation

#### Deep compatibility matrices and internal docs
- `src/compat/**`
- internal design docs that reference historical Rush naming

Reason to defer:
- some are intentionally about legacy compatibility or prior architecture
- update only if the wording becomes user-visible or materially confusing

## File-group recommendations

### Phase 1 deliverables
Produce and use these working sets:
1. `brand-visible`
   - README, quickstart, benchmarks, top-level docs, banner/help text
2. `runtime-compat`
   - env vars, config dirs, socket paths, temp dirs, daemon paths
3. `packaging-distribution`
   - Cargo.toml, Makefile, scripts, Homebrew, CI release workflow
4. `tests-benches`
   - integration tests, smoke tests, benches referencing `target/.../rush`
5. `internal-later`
   - protocol types, error types, internal symbols
6. `ignore-for-now`
   - `.mana`, `research`, historical/generated material unless specifically needed

## Main migration risks

1. Shell path breakage
- Existing users may have `/.../rush` in `/etc/shells`, `chsh`, scripts, dotfiles, or editor integrations.

2. Lost config/state
- Renaming `~/.config/rush` or `~/.rush` without fallback/migration risks silent loss of universal vars/history/runtime behavior.

3. Daemon/socket breakage
- Existing tools or integrations may expect `~/.pi/rush.sock` or `/tmp/pi-rush-$USER.sock`.

4. Automation breakage
- CI scripts, docs, Homebrew installs, local scripts, and tests assume the `rush` binary and artifact names.

5. Overscoping the brand change into a deep refactor
- Renaming every internal `Rush*` symbol early adds churn without user value.

6. Release/distribution coordination risk
- GitHub repo names, artifact names, Homebrew formula naming, and tap paths need a coordinated cutover.

## Recommended sequencing

### Phase 1: audit + mapping
- done here: categorize references and identify risk areas
- next output should be a concrete checklist/file set for implementation

### Phase 2: user-facing AUSH rebrand + compatibility shims
- rename visible product strings and docs
- update banner/help/version copy
- add `AUSH_*` + fallback to `RUSH_*`
- add new config/socket path lookup with fallback to old rush paths
- keep internal symbols unchanged unless needed

### Phase 3: packaging/tooling/distribution rename
- rename Cargo package/bin as decided
- adjust Makefile/scripts/tests/CI/Homebrew/release names
- define and implement binary alias/shim strategy
- publish migration guidance

### Phase 4: internal cleanup
- rename protocol/types/modules only after external migration stabilizes
- remove compatibility shims on an explicit deprecation timeline

## Suggested implementation rules

- Prefer additive compatibility over destructive rename in runtime/config surfaces.
- Treat binary/package renames as a separate decision gate.
- Exclude `.mana/`, `research/`, and `target/` from implementation passes unless intentionally updating durable project docs.
- Use focused grep scopes per phase rather than global replace.

## High-signal files to inspect first in Phase 2

### Branding
- `src/banner/mod.rs`
- `src/main.rs`
- `README.md`
- `QUICKSTART.md`
- `BENCHMARKS.md`
- `docs/INSTALLATION.md`
- `docs/DISTRIBUTION.md`

### Compatibility
- `src/runtime/universal_vars.rs`
- `src/daemon/server.rs`
- `src/daemon/pi_client.rs`
- `src/daemon/pi_rpc.rs`
- `src/runtime/mod.rs`
- `src/executor/expansion.rs`

### Packaging/distribution later
- `Cargo.toml`
- `Makefile`
- `homebrew/Formula/rush.rb`
- `.github/workflows/release.yml`
- integration tests and shell scripts referencing `target/release/rush`
