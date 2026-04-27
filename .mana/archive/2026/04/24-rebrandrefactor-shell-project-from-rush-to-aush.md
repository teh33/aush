---
id: '24'
title: Rebrand/refactor shell project from rush to aush
slug: rebrandrefactor-shell-project-from-rush-to-aush
status: closed
priority: 2
created_at: '2026-04-24T07:53:27.991937Z'
updated_at: '2026-04-27T03:35:08.159059Z'
notes: |-
  ---
  2026-04-24T07:53:41.093924+00:00
  Initial audit: repo contains broad 'rush' identity coupling across Cargo package/binary names, Rust module comments/user-facing strings, env vars (RUSH_*), daemon/socket paths (~/.pi/rush.sock, /tmp/pi-rush-*.sock, ~/.rush), config paths (~/.config/rush), shell scripts, docs/benchmarks/status/quickstart, Homebrew tap/formula, tests/assertions, and pi-rush skill docs. Existing benchmark doc already references possible AUSH branding. Recommend phased rename to avoid breaking internal semantics and migration paths.

  ---
  2026-04-24T16:29:12.784983+00:00
  Continued Phase 2 runtime compatibility work: history now defaults to ~/.aush_history with fallback to existing ~/.rush_history; undo manager now defaults to ~/.aush_undo with fallback to ~/.rush_undo; temp fallback renamed from rush-undo to aush-undo. Verified with cargo check -q plus targeted cargo test -q history::tests and cargo test -q undo::tests. Existing worktree already had unrelated lexer/main and mana edits before this pass.

  ---
  2026-04-27T03:29:54.738894+00:00
  2026-04-27 rebrand remainder audit: AUSH rename is partially done. Cargo now has primary `aush` bin plus legacy `rush`, but package/lib remain `rush` by decision. Remaining visible/runtime surfaces include 260 files with case-insensitive `rush` references outside `.mana`, `target`, and `Cargo.lock`; key leftovers are Cargo authors/homepage/repository, `rushd` daemon binary/docs/socket paths, Homebrew docs/formula text, benchmark/test variable names and binary paths, legacy `.rush*` config/profile tests, `~/.config/rush/universal_vars` fallback docs, pi-rush extension/package surfaces, examples with `.rush` extension, and generated/research/historical reports. Prioritize closing packaging/distribution docs and daemon naming before broad internal cleanup.
labels:
- rebrand
- refactor
- shell
- root-link
closed_at: '2026-04-27T03:35:08.159059Z'
close_reason: 'Auto-closed: all children completed'
is_archived: true
kind: epic
decisions:
- Do not pursue a repo-wide global replacement for the AUSH rebrand. Preserve migration safety by separating visible copy, runtime compatibility, packaging/binary rename, and internal cleanup. Historical `.mana/**`, `research/**`, generated artifacts, and internal protocol/type names may retain Rush references until their scoped phase.
---

Plan and execute a comprehensive project rebrand/refactor from 'rush' to 'aush' ('actually usable shell'). Scope likely includes binary/package names, module/import paths, docs, user-facing text, repository metadata, tests/fixtures, and any semantic renames where 'rush' is part of the product identity rather than generic wording. Conversation started at user request; implementation should begin with an audit of current 'rush' references, risk areas, and a phased rename plan before code changes.

Linked durable coordination: see root-scope epic 126 for the cross-project migration plan and phased decisions.
