# POSIX Compliance Test Suite

This directory contains comprehensive POSIX compliance tests for the AUSH shell.

## Quick Start

```bash
# Setup
./setup.sh

# Run all tests
./run_tests.sh
```

## Test Frameworks

The external harness currently uses:

1. **ShellSpec** - BDD-style executable POSIX behavior checks under `shellspec/`.
2. **Bats-core** - TAP-compliant tests when `.bats` files are present under `bats/`.

`run_tests.sh` builds/locates `target/release/aush`, exports `AUSH_BINARY`, runs ShellSpec from `tests/posix/shellspec`, and then runs any Bats files.

## Current Local External Harness Result

Latest complete `bash tests/posix/run_tests.sh` result:

- ShellSpec: 286 examples, 55 failures, 33 warnings, 5 skips.
- Bats: no `.bats` files currently present.

These are regression numbers, not POSIX certification.

## Test Coverage

ShellSpec currently covers:

- Builtin commands
- Control flow
- I/O redirection
- Variables and expansion
- Pipelines and jobs
- Signal handling

## Documentation

See `COMPLIANCE_REPORT.md` for detailed test results and compliance analysis.

See `/docs/POSIX_TEST_SUITE.md` for complete integration documentation.
