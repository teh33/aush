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

echo "AUSH Full Benchmark"
echo "Binary: $RUSH_BIN"
echo

echo "=== Fast smoke gate ==="
bash ./benches/aush_smoke_fast.sh "$RUSH_BIN"

echo "=== Persistent interactive benchmark ==="
bash ./benches/interactive_benchmark.sh

echo
echo "=== Persistent session benchmark ==="
bash ./benches/session_benchmark.sh

echo
echo "AUSH full benchmark complete"
