#!/usr/bin/env bash

set -euo pipefail

RUSH_BIN="${1:-./target/release/rush}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -x "$RUSH_BIN" ]]; then
  echo "Error: Rush binary not found at $RUSH_BIN"
  echo "Run: cargo build --release"
  exit 1
fi

PASS=0
FAIL=0
SKIP=0

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

run_check() {
  local name="$1"
  local cmd="$2"

  echo -e "${BLUE}→${NC} $name"
  if bash -lc "$cmd"; then
    echo -e "${GREEN}✓${NC} $name"
    PASS=$((PASS + 1))
  else
    echo -e "${RED}✗${NC} $name"
    FAIL=$((FAIL + 1))
  fi
  echo
}

skip_check() {
  local name="$1"
  local reason="$2"
  echo -e "${YELLOW}○${NC} $name (skip: $reason)"
  SKIP=$((SKIP + 1))
  echo
}

echo "AUSH Fast Smoke Benchmark"
echo "Binary: $RUSH_BIN"
echo

run_check "startup smoke" "'$RUSH_BIN' --no-rc -c 'echo startup-ok' | grep -qx 'startup-ok'"
run_check "non-interactive pipe" "cargo test -q --test non_tty_tests test_piped_input_single_command -- --exact"
run_check "login/no-rc behavior" "cargo test -q --test login_shell_tests test_no_rc_flag -- --exact"
run_check "signal interrupt behavior" "cargo test -q --test signal_handling_tests test_command_flag_with_signal -- --exact"
run_check "targeted core shell smoke" "tests/smoke_test.sh '$RUSH_BIN' >/tmp/aush-fast-smoke.log 2>&1 && tail -n 5 /tmp/aush-fast-smoke.log >/dev/null"

skip_check "first-prompt visibility" "needs PTY-aware automated harness rather than plain shell script"
skip_check "terminal recovery after revoked PTY" "needs terminal/PTY integration test harness"

echo "AUSH fast summary: pass=$PASS fail=$FAIL skip=$SKIP"

if [[ $FAIL -ne 0 ]]; then
  exit 1
fi
