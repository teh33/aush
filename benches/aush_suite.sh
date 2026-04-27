#!/usr/bin/env bash

set -euo pipefail

AUSH_BIN="${AUSH_BIN:-${1:-./target/release/aush}}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -x "$AUSH_BIN" && -x ./target/release/aush ]]; then
  AUSH_BIN=./target/release/aush
fi

MODE="${2:-${AUSH_BENCH_MODE:-full}}"
OUT_DIR="${AUSH_BENCH_OUT_DIR:-reports/benchmarks}"
STAMP="$(date +%Y%m%d-%H%M%S)"
RUN_DIR="$OUT_DIR/$STAMP"

usage() {
  cat <<'USAGE'
Usage: benches/aush_suite.sh [AUSH_BIN] [compat|perf-report|perf-regress|full]

Modes:
  compat       Release gate. Runs POSIX-core + agentic shell compatibility checks.
  perf-report  Generates human-readable bash/zsh comparison artifacts.
  perf-regress Runs performance checks intended for baseline/regression tracking.
  full         Runs compat, perf-report, and perf-regress.

Environment:
  AUSH_BIN             Override AUSH binary path.
  AUSH_BENCH_OUT_DIR   Output directory for generated artifacts (default: reports/benchmarks).
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || "${2:-}" == "-h" || "${2:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ ! -x "$AUSH_BIN" ]]; then
  echo "Error: AUSH binary not found at $AUSH_BIN"
  echo "Run: cargo build --release"
  exit 1
fi

mkdir -p "$RUN_DIR"

run_step() {
  local name="$1"
  shift
  local log="$RUN_DIR/${name}.log"
  local start end elapsed

  echo
  echo "=== $name ==="
  start=$(date +%s)
  if "$@" 2>&1 | tee "$log"; then
    end=$(date +%s)
    elapsed=$((end - start))
    echo "PASS $name (${elapsed}s)" | tee -a "$RUN_DIR/summary.txt" >/dev/null
  else
    local status=${PIPESTATUS[0]}
    end=$(date +%s)
    elapsed=$((end - start))
    echo "FAIL $name (${elapsed}s, exit $status)" | tee -a "$RUN_DIR/summary.txt" >/dev/null
    return "$status"
  fi
}

run_compat() {
  echo "AUSH compatibility gate"
  echo "Binary: $AUSH_BIN"
  echo "Output: $RUN_DIR"

  run_step compat-fast-smoke bash ./benches/aush_smoke_fast.sh "$AUSH_BIN"
  run_step compat-posix-core bash ./tests/posix/run_tests.sh
  run_step compat-agentic-workloads bash ./benches/agentic_compat.sh "$AUSH_BIN"
}

run_perf_report() {
  echo "AUSH performance report suite"
  echo "Binary: $AUSH_BIN"
  echo "Output: $RUN_DIR"

  if command -v hyperfine >/dev/null 2>&1; then
    run_step perf-startup hyperfine \
      --warmup 5 \
      --min-runs 30 \
      --export-markdown "$RUN_DIR/startup.md" \
      --export-json "$RUN_DIR/startup.json" \
      "$AUSH_BIN --no-rc -c exit" \
      "bash -c exit" \
      "zsh -f -c exit"

    run_step perf-agentic-hyperfine hyperfine \
      --warmup 2 \
      --min-runs 10 \
      --export-markdown "$RUN_DIR/agentic-workloads.md" \
      --export-json "$RUN_DIR/agentic-workloads.json" \
      "$AUSH_BIN --no-rc ./benches/workloads/agentic_core.sh" \
      "bash ./benches/workloads/agentic_core.sh" \
      "zsh -f ./benches/workloads/agentic_core.sh"
  else
    echo "SKIP hyperfine performance report: install with 'brew install hyperfine' or 'cargo install hyperfine'" | tee -a "$RUN_DIR/summary.txt"
  fi

  run_step perf-session bash ./benches/session_benchmark.sh
}

run_perf_regress() {
  echo "AUSH performance regression suite"
  echo "Binary: $AUSH_BIN"
  echo "Output: $RUN_DIR"

  run_step regress-criterion-startup cargo bench --bench startup
  run_step regress-criterion-builtins cargo bench --bench builtins
  run_step regress-criterion-agentic cargo bench --bench ai_agent_workloads
}

case "$MODE" in
  compat)
    run_compat
    ;;
  perf-report)
    run_perf_report
    ;;
  perf-regress)
    run_perf_regress
    ;;
  full)
    run_compat
    run_perf_report
    run_perf_regress
    ;;
  *)
    echo "Unknown mode: $MODE"
    usage
    exit 2
    ;;
esac

{
  echo "# AUSH benchmark run $STAMP"
  echo
  echo "- Binary: \`$AUSH_BIN\`"
  echo "- Mode: \`$MODE\`"
  echo "- Output: \`$RUN_DIR\`"
  echo
  echo '```'
  cat "$RUN_DIR/summary.txt" 2>/dev/null || true
  echo '```'
} > "$RUN_DIR/summary.md"

echo
echo "AUSH benchmark suite complete"
echo "Summary: $RUN_DIR/summary.md"
