#!/usr/bin/env bash

set -euo pipefail

AUSH_BIN="${AUSH_BIN:-${1:-./target/release/aush}}"
if [[ ! -x "$AUSH_BIN" && -x ./target/release/aush ]]; then
  AUSH_BIN=./target/release/aush
fi

if [[ ! -x "$AUSH_BIN" ]]; then
  echo "Error: AUSH binary not found at $AUSH_BIN"
  echo "Run: cargo build --release"
  exit 1
fi

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "Error: hyperfine is required for shell comparison"
  exit 1
fi

OUT_DIR="${AUSH_BENCH_OUT_DIR:-reports/benchmarks}/shell-comparison-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$OUT_DIR"

run_compare() {
  local name="$1"
  shift
  echo
  echo "=== $name ==="
  hyperfine \
    --warmup 2 \
    --min-runs 10 \
    --export-markdown "$OUT_DIR/${name}.md" \
    --export-json "$OUT_DIR/${name}.json" \
    "$@"
}

run_compare startup \
  "$AUSH_BIN --no-rc -c exit" \
  "bash -c exit" \
  "zsh -f -c exit"

run_compare agentic-core-script \
  "$AUSH_BIN --no-rc ./benches/workloads/agentic_core.sh" \
  "bash ./benches/workloads/agentic_core.sh" \
  "zsh -f ./benches/workloads/agentic_core.sh"

run_compare text-sort-uniq-pipeline \
  "$AUSH_BIN --no-rc -c \"printf '3\\n1\\n4\\n1\\n5\\n9\\n2\\n6\\n' | sort | uniq -c | wc -l | tr -d ' '\"" \
  "bash -c \"printf '3\\n1\\n4\\n1\\n5\\n9\\n2\\n6\\n' | sort | uniq -c | wc -l | tr -d ' '\"" \
  "zsh -f -c \"printf '3\\n1\\n4\\n1\\n5\\n9\\n2\\n6\\n' | sort | uniq -c | wc -l | tr -d ' '\""

echo
echo "Shell comparison artifacts: $OUT_DIR"
