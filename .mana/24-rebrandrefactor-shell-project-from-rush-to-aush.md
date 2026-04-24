---
id: '24'
title: Rebrand/refactor shell project from rush to aush
slug: rebrandrefactor-shell-project-from-rush-to-aush
status: open
priority: 2
created_at: '2026-04-24T07:53:27.991937Z'
updated_at: '2026-04-24T16:33:57.977234Z'
notes: |-
  ---
  2026-04-24T07:53:41.093924+00:00
  Initial audit: repo contains broad 'rush' identity coupling across Cargo package/binary names, Rust module comments/user-facing strings, env vars (RUSH_*), daemon/socket paths (~/.pi/rush.sock, /tmp/pi-rush-*.sock, ~/.rush), config paths (~/.config/rush), shell scripts, docs/benchmarks/status/quickstart, Homebrew tap/formula, tests/assertions, and pi-rush skill docs. Existing benchmark doc already references possible AUSH branding. Recommend phased rename to avoid breaking internal semantics and migration paths.

  ---
  2026-04-24T16:29:12.784983+00:00
  Continued Phase 2 runtime compatibility work: history now defaults to ~/.aush_history with fallback to existing ~/.rush_history; undo manager now defaults to ~/.aush_undo with fallback to ~/.rush_undo; temp fallback renamed from rush-undo to aush-undo. Verified with cargo check -q plus targeted cargo test -q history::tests and cargo test -q undo::tests. Existing worktree already had unrelated lexer/main and mana edits before this pass.
labels:
- rebrand
- refactor
- shell
- root-link
kind: epic
decisions:
- Do not pursue a repo-wide global replacement for the AUSH rebrand. Preserve migration safety by separating visible copy, runtime compatibility, packaging/binary rename, and internal cleanup. Historical `.mana/**`, `research/**`, generated artifacts, and internal protocol/type names may retain Rush references until their scoped phase.
---

Plan and execute a comprehensive project rebrand/refactor from 'rush' to 'aush' ('actually usable shell'). Scope likely includes binary/package names, module/import paths, docs, user-facing text, repository metadata, tests/fixtures, and any semantic renames where 'rush' is part of the product identity rather than generic wording. Conversation started at user request; implementation should begin with an audit of current 'rush' references, risk areas, and a phased rename plan before code changes.

Linked durable coordination: see root-scope epic 126 for the cross-project migration plan and phased decisions.
