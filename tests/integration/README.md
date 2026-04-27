# AUSH Shell Integration Tests

This directory contains comprehensive integration tests for AUSH shell, specifically designed to verify login shell functionality.

## Quick Start

```bash
# Run all integration tests
./run_all.sh

# Run individual test suites
./login_shell_test.sh      # Comprehensive tests (30 tests)
./non_tty_test.sh          # Non-TTY tests (20 tests)
./signal_test.sh           # Signal handling (15 tests)
./redirection_test.sh      # File redirection (20 tests)
./pipeline_test.sh         # Pipelines (20 tests)
./job_control_test.sh      # Job control (20 tests)
```

## Requirements

1. Build aush in release mode:
   ```bash
   cargo build --release
   ```

2. Make scripts executable (already done):
   ```bash
   chmod +x *.sh ../fixtures/*.sh
   ```

## Test Suites

### login_shell_test.sh
Comprehensive integration tests covering:
- Non-interactive mode (piped input)
- Script execution
- Command substitution (-c flag)
- Exit codes and conditionals
- File redirection
- Pipelines
- Signal handling basics
- Stdin from files
- Builtin commands
- Error handling

### non_tty_test.sh
Tests for non-TTY scenarios:
- Piped input streams
- File redirection
- Here-documents
- Empty and whitespace handling
- Long input streams
- Cron job simulation
- CI/CD pipeline simulation

### signal_test.sh
Signal handling tests:
- SIGTERM handling
- SIGINT handling (Ctrl-C)
- SIGKILL handling
- Process cleanup
- Rapid start-stop cycles

### redirection_test.sh
File redirection tests:
- Output redirection (>)
- Append redirection (>>)
- Input redirection (<)
- Redirect with builtins
- Redirect with pipelines
- Multiple redirects

### pipeline_test.sh
Pipeline functionality:
- Simple pipelines (|)
- Multi-stage pipelines
- Pipeline with grep/wc/sort/uniq
- Pipeline exit codes
- Pipeline with redirects

### job_control_test.sh
Job control tests:
- Background jobs (&)
- Foreground job completion
- Job with pipelines/redirects
- Job isolation
- Process cleanup

## Output Format

Tests provide colored output:
- **GREEN**: Pass
- **RED**: Fail
- **YELLOW**: Warnings/notes

Each suite shows:
- Individual test results
- Summary (passed/failed/total)
- Exit code (0 = all pass, 1 = failures)

## CI Integration

These tests run automatically in GitHub Actions on every push/PR.

See: `.github/workflows/integration-tests.yml`

## Documentation

Full documentation: `../../docs/integration-testing.md`

## Test Fixtures

Sample scripts in `../fixtures/`:
- `simple_script.sh` - Basic script
- `exit_code_script.sh` - Exit code tests
- `pipeline_script.sh` - Pipeline scenarios
- `redirection_script.sh` - Redirection scenarios
- `conditional_script.sh` - Conditionals
- `variable_script.sh` - Variable handling
