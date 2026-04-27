#!/usr/bin/env bash

set -euo pipefail

AUSH_BIN="${1:-./target/release/aush}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -x "$AUSH_BIN" && -x ./target/release/aush ]]; then
  AUSH_BIN=./target/release/aush
fi
AUSH_BIN="$(cd "$(dirname "$AUSH_BIN")" && pwd)/$(basename "$AUSH_BIN")"

if [[ ! -x "$AUSH_BIN" ]]; then
  echo "Error: AUSH binary not found at $AUSH_BIN"
  exit 1
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf "$WORK_DIR"' EXIT

mkdir -p "$WORK_DIR/src" "$WORK_DIR/logs" "$WORK_DIR/data"
cat > "$WORK_DIR/src/main.rs" <<'RS'
fn main() {
    println!("hello");
    // TODO: tighten parser recovery
}
RS
cat > "$WORK_DIR/src/lib.rs" <<'RS'
pub fn answer() -> i32 { 42 }
// FIXME: replace stub
RS
cat > "$WORK_DIR/logs/test.log" <<'LOG'
INFO boot
WARN retrying request
ERROR failed to parse fixture
INFO shutdown
LOG
cat > "$WORK_DIR/data/items.txt" <<'DATA'
alpha 3
beta 1
gamma 2
DATA

PASS=0
FAIL=0

run_case() {
  local name="$1"
  local script="$2"
  local expected="$3"
  local stdout_file="$WORK_DIR/$name.out"
  local stderr_file="$WORK_DIR/$name.err"

  printf '→ %s\n' "$name"
  if (cd "$WORK_DIR" && "$AUSH_BIN" --no-rc -c "$script" >"$stdout_file" 2>"$stderr_file"); then
    if diff -u <(printf '%s\n' "$expected") "$stdout_file"; then
      printf '✓ %s\n\n' "$name"
      PASS=$((PASS + 1))
    else
      printf '✗ %s output mismatch\n' "$name"
      cat "$stderr_file"
      printf '\n'
      FAIL=$((FAIL + 1))
    fi
  else
    local status=$?
    printf '✗ %s exited %s\n' "$name" "$status"
    cat "$stderr_file"
    printf '\n'
    FAIL=$((FAIL + 1))
  fi
}

run_case \
  "pipeline_log_triage" \
  "cat logs/test.log | grep ERROR | wc -l | tr -d ' '" \
  "1"

run_case \
  "repo_todo_scan" \
  "grep -R TODO src | wc -l | tr -d ' '" \
  "1"

run_case \
  "command_substitution_and_quoting" \
  "files=\$(find src -name '*.rs' | wc -l | tr -d ' '); echo \"rust files: \$files\"" \
  "rust files: 2"

run_case \
  "semicolon_and_status_sequence" \
  "false; echo after-false; true; echo done" \
  "after-false
done"

run_case \
  "loop_filter_sort" \
  "for f in src/*.rs; do echo \"\$f\"; done" \
  "src/lib.rs
src/main.rs"

run_case \
  "redirect_append_readback" \
  "echo alpha > result.txt; echo beta >> result.txt; cat result.txt" \
  "alpha
beta"

run_case \
  "if_test_and_glob" \
  "if test -f src/main.rs; then echo exists; else echo missing; fi; echo src/*.rs | wc -w | tr -d ' '" \
  "exists
2"

echo "agentic compatibility summary: pass=$PASS fail=$FAIL"

if [[ $FAIL -ne 0 ]]; then
  exit 1
fi
