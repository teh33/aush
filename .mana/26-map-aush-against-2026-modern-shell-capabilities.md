---
id: '26'
title: Map AUSH against 2026+ modern shell capabilities
slug: map-aush-against-2026-modern-shell-capabilities
status: open
priority: 2
created_at: '2026-04-24T09:22:55.444500Z'
updated_at: '2026-04-24T09:31:00.378405Z'
notes: |-
  ---
  2026-04-24T09:27:43.637278+00:00
  Recommended integration sequence from planning discussion:
  1. Start with a small command metadata/effects registry for builtins, not a broad UI rewrite. This unlocks schema, risk labels, policy, approval, docs, autocomplete, and agent guardrails.
  2. Add receipts/audit ledger next: record command, cwd, start/end, exit, duration, declared effects, and touched paths where known. Build semantic history/timeline on top of receipts instead of extending string history ad hoc.
  3. Add approval/policy gates for high-risk declared effects after metadata + receipts exist.
  4. Add verification blocks and workflow/task graph later, once receipts can represent outcomes.
  5. Treat durable jobs, secrets hygiene, remote execution, and cross-system object URIs as later product layers.
  Near-term principle: deepen the existing AUSH differentiator (AI-native structured shell) by making commands inspectable, auditable, and policy-aware before adding more surface area.

  ---
  2026-04-24T09:31:00.378400+00:00
  Design constraint from user: any new AUSH effect/schema/receipt/policy output shown in terminal must be pretty and human-readable. Internal identifiers like `delete_file` are acceptable for JSON/receipts/schema, but interactive terminal output should render polished labels such as `Delete files`, concise descriptions, colors/tables as appropriate, and avoid exposing raw enum names unless explicitly requested with JSON/debug flags. This applies especially to approval gates, effect summaries, receipts, semantic history, and timeline views.
labels:
- roadmap
- aush
- modern-shell
- architecture
verify: cd /Users/asher/rush && rg -n 'Structured pipelines|Designed for AI coding agents|Command history|Undo Capability' README.md QUICKSTART.md && test -f src/value/mod.rs && test -f src/ai/agent.rs
kind: epic
feature: true
---

Goal: maintain a durable roadmap thread comparing AUSH/Rush's current implementation against a 2026+ shell vision: typed object pipelines, effect-aware execution, receipts, workflows, agent protocol, policies, durable jobs, verification, secrets hygiene, rich UI, remote/cross-system integrations, and memory.

Current inspected evidence:
- README.md claims AUSH differentiators: structured pipelines, built-in AI agent via `?`, Lua extensions, POSIX compatibility, daemon mode, structured JSON for agent workflows.
- src/value/mod.rs defines typed Value/Table, but src/executor/structured_ops.rs currently operates on serde_json::Value arrays for pipeline operators.
- src/history/mod.rs stores command + timestamp only; no semantic receipts/effects/outputs.
- src/undo/mod.rs tracks file create/delete/modify/move backups only under `.rush_undo`.
- src/jobs/mod.rs implements process job control, not durable jobs surviving terminal/laptop/session loss.
- src/ai/agent.rs has an interactive tool loop with confirmation for shell/write/edit, but no formal agent execution protocol with budgets/path/network policies/receipts/rollback gates.
- src/intent/mod.rs implements natural-language intent to suggested command with user confirmation.
- src/daemon/protocol.rs supports session init, execute, signal, shutdown, stats, but no persisted job/workflow/receipt model.

Use this epic for future decomposition after deciding which product direction matters most: compatibility-first daily shell, agent-safe ops shell, or typed workflow substrate.
