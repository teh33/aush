---
id: '20'
title: 'bug: Parser/runtime bug cluster: existing suites still fail for case-pattern parsing and some while/until script forms (e.g. case bracket/dash/variable patterns, while loop scripts with compound conditions, until complex condition). Investigate and fix the parser/executor mismatch without touching the completed executor module extraction.'
slug: bug-parserruntime-bug-cluster-existing-suites-stil
status: in_progress
priority: 2
created_at: '2026-03-25T03:58:10.532766Z'
updated_at: '2026-03-25T04:02:25.575987Z'
verify: cd /Users/asher/rush && cargo test --test case_statement_tests && cargo test --test while_loop_tests && cargo test --test until_loop_tests
fail_first: true
checkpoint: '76da292c364175156adcefe8508b4f46bea6296f'
claimed_by: pi-agent
claimed_at: '2026-03-25T04:02:25.575987Z'
attempt_log:
- num: 1
  outcome: abandoned
  agent: pi-agent
  started_at: '2026-03-25T04:02:25.575987Z'
---

Parser/runtime bug cluster: existing suites still fail for case-pattern parsing and some while/until script forms (e.g. case bracket/dash/variable patterns, while loop scripts with compound conditions, until complex condition). Investigate and fix the parser/executor mismatch without touching the completed executor module extraction.

## Files
- tests/case_statement_tests.rs
- tests/while_loop_tests.rs
- tests/until_loop_tests.rs
- src/parser
- src/executor
