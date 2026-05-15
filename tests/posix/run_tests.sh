#!/bin/bash
# POSIX Compliance Test Runner for AUSH Shell

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AUSH_BINARY="${PROJECT_ROOT}/target/release/aush"
if [[ ! -f "$AUSH_BINARY" && -f "${PROJECT_ROOT}/target/release/aush" ]]; then
    AUSH_BINARY="${PROJECT_ROOT}/target/release/aush"
fi
AUSH_BINARY="$AUSH_BINARY"

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║          POSIX Compliance Test Suite for AUSH Shell           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if aush binary exists
if [[ ! -f "$AUSH_BINARY" ]]; then
    echo "Warning: AUSH binary not found at $AUSH_BINARY"
    echo "Building aush..."
    cd "$PROJECT_ROOT"
    cargo build --release || {
        echo "Failed to build aush"
        exit 1
    }
fi

echo "AUSH binary: $AUSH_BINARY"
echo ""

# Export for tests
export AUSH_BINARY
export PATH="${PROJECT_ROOT}/target/release:$PATH"

# Run ShellSpec tests
if command -v shellspec &> /dev/null; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  ShellSpec Tests"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cd "$SCRIPT_DIR"
    if [[ -d "shellspec" ]] && ls shellspec/*_spec.sh &> /dev/null; then
        (
            cd "$SCRIPT_DIR/shellspec"
            shellspec *_spec.sh --format documentation --shell /bin/sh || true
        )
    else
        echo "No ShellSpec tests found"
    fi
    echo ""
else
    echo "ShellSpec not installed"
    echo ""
fi

# Run Bats tests
if command -v bats &> /dev/null; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Bats Tests"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cd "$SCRIPT_DIR"
    if [[ -d "bats" ]] && ls bats/*.bats &> /dev/null; then
        bats bats/*.bats || true
    else
        echo "No Bats tests found"
    fi
    echo ""
else
    echo "Bats not installed"
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test run complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "For detailed compliance report, see:"
echo "  $SCRIPT_DIR/COMPLIANCE_REPORT.md"
