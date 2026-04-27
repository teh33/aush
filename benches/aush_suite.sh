#!/usr/bin/env bash

set -euo pipefail

AUSH_BIN="${1:-./target/release/aush}"
if [[ ! -x "$AUSH_BIN" && -x ./target/release/aush ]]; then
  AUSH_BIN=./target/release/aush
fi
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -x "$AUSH_BIN" ]]; then
  echo "Error: AUSH binary not found at $AUSH_BIN"
  echo "Run: cargo build --release"
  exit 1
fi

echo "AUSH Full Benchmark"
echo "Binary: $AUSH_BIN"
echo

echo "=== Fast smoke gate ==="
bash ./benches/aush_smoke_fast.sh "$AUSH_BIN"

echo "=== Persistent interactive benchmark ==="
bash ./benches/interactive_benchmark.sh

echo
echo "=== Persistent session benchmark ==="
bash ./benches/session_benchmark.sh

echo
echo "AUSH full benchmark complete"
