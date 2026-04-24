---
id: '24'
title: Rebrand/refactor shell project from rush to aush
slug: rebrandrefactor-shell-project-from-rush-to-aush
status: open
priority: 2
created_at: '2026-04-24T07:53:27.991937Z'
updated_at: '2026-04-24T07:53:41.093932Z'
notes: |-
  ---
  2026-04-24T07:53:41.093924+00:00
  Initial audit: repo contains broad 'rush' identity coupling across Cargo package/binary names, Rust module comments/user-facing strings, env vars (RUSH_*), daemon/socket paths (~/.pi/rush.sock, /tmp/pi-rush-*.sock, ~/.rush), config paths (~/.config/rush), shell scripts, docs/benchmarks/status/quickstart, Homebrew tap/formula, tests/assertions, and pi-rush skill docs. Existing benchmark doc already references possible AUSH branding. Recommend phased rename to avoid breaking internal semantics and migration paths.
labels:
- rebrand
- refactor
- shell
- root-link
kind: epic
---

Plan and execute a comprehensive project rebrand/refactor from 'rush' to 'aush' ('actually usable shell'). Scope likely includes binary/package names, module/import paths, docs, user-facing text, repository metadata, tests/fixtures, and any semantic renames where 'rush' is part of the product identity rather than generic wording. Conversation started at user request; implementation should begin with an audit of current 'rush' references, risk areas, and a phased rename plan before code changes.

Linked durable coordination: see root-scope epic 126 for the cross-project migration plan and phased decisions.
