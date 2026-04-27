# Contributing to AUSH

Thank you for your interest in contributing to AUSH! This document provides guidelines for contributing to the project.

## How to Report Bugs

Please use [GitHub Issues](https://github.com/opus-workshop/aush/issues) to report bugs. Include the following:

- AUSH version (`aush --version`)
- Operating system and version
- Steps to reproduce the issue
- Expected vs actual behavior
- Any relevant error output

## How to Submit Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run the test suite (see below)
5. Commit your changes with a clear message
6. Push to your fork and submit a pull request

Please keep PRs focused on a single change. If you have multiple unrelated fixes, submit them as separate PRs.

## Building

```bash
# Debug build
cargo build

# Optimized release build
cargo build --release
```

## Testing

```bash
# Run all unit and integration tests
cargo test

# Run all integration tests
cargo test --test '*'

# Run POSIX compliance tests
cd tests/posix && ./run_tests.sh

# Run a specific test file
cargo test --test pipeline_tests
```

Please ensure all tests pass before submitting a PR.

## Code Style

AUSH uses standard Rust tooling for code quality:

- **Formatting**: `cargo fmt` (enforced by `rustfmt`)
- **Linting**: `cargo clippy`

Run both before submitting:

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Architecture

AUSH is organized into several key modules:

| Module | Purpose |
|--------|---------|
| `src/lexer/` | Tokenization (powered by Logos) |
| `src/parser/` | AST construction (powered by nom) |
| `src/executor/` | Command execution engine |
| `src/runtime/` | Variable scoping and environment |
| `src/builtins/` | 45+ native Rust built-in commands |
| `src/daemon/` | Client-server architecture for fast startup |
| `src/jobs/` | Job control subsystem |
| `src/signal.rs` | POSIX signal handling |

For detailed architecture documentation, see the [docs/](docs/) directory.

## Areas Where Help is Appreciated

- POSIX compliance edge cases
- Performance optimizations
- Documentation improvements
- Platform support (BSD, Windows via WSL)
- Bug reports and test cases

## License

By contributing, you agree that your contributions will be dual-licensed under MIT or Apache-2.0, consistent with the project license.
