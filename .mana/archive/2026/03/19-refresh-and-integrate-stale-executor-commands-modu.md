---
id: '19'
title: Refresh and integrate stale executor commands module
slug: refresh-and-integrate-stale-executor-commands-modu
status: closed
priority: 2
created_at: '2026-03-25T03:43:44.666935Z'
updated_at: '2026-03-25T08:23:07.251241Z'
labels:
- refactor
- executor
- rust
closed_at: '2026-03-25T08:23:07.251241Z'
close_reason: verify passed (tidy sweep)
verify: cd /Users/asher/rush && cargo check && cargo test --test function_calling_test test_return && cargo test --lib test_until_with_break && cargo test --lib test_while_true_break
is_archived: true
history:
- attempt: 1
  started_at: '2026-03-25T08:23:06.540173Z'
  finished_at: '2026-03-25T08:23:07.236631Z'
  duration_secs: 0.696
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 7 tests
    test test_return_in_nested_function_calls ... ok
    test test_return_early_from_function ... ok
    test test_return_with_no_argument_defaults_to_zero ... ok
    test test_return_with_exit_code_42 ... ok
    test test_return_with_conditional_logic ... ok
    test test_return_preserves_function_output ... ok
    test test_return_with_various_exit_codes ... ok

    test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.00s


    running 1 test
    test executor::tests::test_until_with_break ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1001 filtered out; finished in 0.00s


    running 1 test
    test executor::tests::test_while_true_break ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1001 filtered out; finished in 0.00s
---

`src/executor/commands.rs` exists but is stale against the current AST/runtime and cannot be wired in directly. Update it to match the current parser/runtime/executor shape (remove references to missing AST variants and fields, align helper signatures, and preserve current command execution behavior), then replace the duplicated live command/subshell/background implementations in `src/executor/mod.rs` with the module version. Verify with targeted executor command/background/subshell/function tests and cargo check.
