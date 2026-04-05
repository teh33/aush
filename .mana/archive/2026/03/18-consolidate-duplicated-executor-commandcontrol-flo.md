---
id: '18'
title: Consolidate duplicated executor command/control-flow logic
slug: consolidate-duplicated-executor-commandcontrol-flo
status: closed
priority: 2
created_at: '2026-03-25T03:31:08.084197Z'
updated_at: '2026-03-25T08:23:06.505199Z'
labels:
- refactor
- executor
- rust
closed_at: '2026-03-25T08:23:06.505199Z'
close_reason: verify passed (tidy sweep)
verify: cd /Users/asher/rush && cargo fmt --check && cargo test --lib test_until_with_break && cargo test --lib test_while_true_break && cargo test --lib test_while_loop_continue && cargo test --lib test_for_loop_continue && cargo test --test function_calling_test test_return
is_archived: true
history:
- attempt: 1
  started_at: '2026-03-25T08:23:01.500378Z'
  finished_at: '2026-03-25T08:23:06.476407Z'
  duration_secs: 4.976
  result: pass
  exit_code: 0
outputs:
  text: |-
    running 1 test
    test executor::tests::test_until_with_break ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1001 filtered out; finished in 0.02s


    running 1 test
    test executor::tests::test_while_true_break ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1001 filtered out; finished in 0.00s


    running 1 test
    test executor::tests::test_while_loop_continue ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1001 filtered out; finished in 0.00s


    running 1 test
    test executor::tests::test_for_loop_continue ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1001 filtered out; finished in 0.00s


    running 7 tests
    test test_return_with_conditional_logic ... ok
    test test_return_with_exit_code_42 ... ok
    test test_return_with_no_argument_defaults_to_zero ... ok
    test test_return_early_from_function ... ok
    test test_return_with_various_exit_codes ... ok
    test test_return_preserves_function_output ... ok
    test test_return_in_nested_function_calls ... ok

    test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 25 filtered out; finished in 0.02s
---

Refactor src/executor so the active executor path uses the split modules instead of duplicated logic in src/executor/mod.rs. Focus on command dispatch and control-flow handling first. Preserve current behavior, keep changes incremental, and verify with targeted executor loop/function/pipeline tests plus formatting. Do not do broad unrelated cleanup.
