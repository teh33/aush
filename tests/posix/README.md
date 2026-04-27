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

We use three complementary testing frameworks:

1. **ShellSpec** - BDD-style testing (113 tests)
2. **Bats-core** - TAP-compliant testing (25 tests)
3. **ShellCheck** - Static analysis

## Test Coverage

- Builtin Commands (33 tests)
- Control Flow (25 tests)
- I/O Redirection (20 tests)
- Variables/Expansion (35 tests)
- Pipelines (20 tests)

**Total**: 133 tests

## Documentation

See `COMPLIANCE_REPORT.md` for detailed test results and compliance analysis.

See `/docs/POSIX_TEST_SUITE.md` for complete integration documentation.
