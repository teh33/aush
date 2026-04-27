---
id: '28'
title: Local Bash source is available as read-only performance/behavior reference
slug: local-bash-source-is-available-as-read-only-perfor
status: open
priority: 3
created_at: '2026-04-27T20:47:30.632264Z'
updated_at: '2026-04-27T20:47:30.632264Z'
labels:
- fact
verify: test -d /Users/asher/bash && test -d /Users/asher/aush
kind: epic
unit_type: fact
stale_after: '2026-07-26T20:47:30.648571Z'
paths:
- /Users/asher/bash
- /Users/asher/aush
---

The repository ~/bash contains Bash source (~185k LOC C, human-written standard implementation). Use it as read-only reference when auditing aush (~95k LOC Rust, LLM-written) for behavior, parser/runtime semantics, and benchmark performance improvements. Do not modify ~/bash; compare selectively rather than copying architecture blindly.
