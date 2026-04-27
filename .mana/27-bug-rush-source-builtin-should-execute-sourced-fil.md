---
id: '27'
title: 'bug: rush source builtin should execute sourced files in current executor context'
slug: bug-rush-source-builtin-should-execute-sourced-fil
status: open
priority: 2
created_at: '2026-04-24T16:15:10.537957Z'
updated_at: '2026-04-24T16:16:26.203011Z'
acceptance: '`source file; command-from-file` works in one rush invocation without requiring a later manual source; aliases/functions/variables defined by a sourced file are usable immediately; missing file reports a useful source error; existing source_file tests continue passing.'
notes: |-
  ---
  2026-04-24T16:16:26.203009+00:00
  Applied narrow user-facing fix in src/main.rs: interactive rush/aush startup now falls back to ~/.zshrc when neither ~/.aushrc nor ~/.rushrc exists, so users with existing zsh config get commands loaded without manually running `source ~/.zshrc`. Verified with `cargo test source --quiet`. Deeper cleanup remains: builtin_source in src/builtins/mod.rs is still TODO-style and should eventually be moved into executor context rather than temp executor per line.
labels:
- bug
- source
- shell-compat
verify: cargo test source --quiet
kind: job
---

User reports needing to run `source ~/.zshrc` manually before commands work in rush. Initial inspection shows `src/builtins/mod.rs::builtin_source` is a TODO-style implementation that creates a temporary Executor per line and copies runtime state, which is fragile and cannot preserve executor-owned state consistently. `src/executor/mod.rs::source_file` also executes config line-by-line and is used for startup rc files. Fix should make `source`/`.` execute a sourced file in the current executor/runtime context, preserving variables, aliases, functions, cwd/env, and return behavior. Avoid broad parser/runtime refactors; keep change focused. Existing dirty files before this work: .mana/25.4.2..., .mana/index.yaml, src/lexer/mod.rs (do not overwrite unrelated changes).
