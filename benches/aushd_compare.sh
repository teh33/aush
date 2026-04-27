#!/usr/bin/env bash

set -euo pipefail

AUSH_BIN="${AUSH_BIN:-${1:-./target/release/aush}}"
AUSHD_BIN="${AUSHD_BIN:-${2:-./target/release/aushd}}"
OUT_DIR="${AUSH_BENCH_OUT_DIR:-reports/benchmarks}/daemon-compare-$(date +%Y%m%d-%H%M%S)"
SOCKET_PATH="${HOME}/.aush/daemon.sock"

if [[ ! -x "$AUSH_BIN" || ! -x "$AUSHD_BIN" ]]; then
  echo "Error: expected executable AUSH_BIN=$AUSH_BIN and AUSHD_BIN=$AUSHD_BIN"
  echo "Run: cargo build --release --bins"
  exit 1
fi

if ! command -v hyperfine >/dev/null 2>&1; then
  echo "Error: hyperfine is required for daemon comparison"
  exit 1
fi

mkdir -p "$OUT_DIR"

cleanup() {
  "$AUSHD_BIN" stop >/dev/null 2>&1 || true
}
trap cleanup EXIT

"$AUSHD_BIN" stop >/dev/null 2>&1 || true
"$AUSHD_BIN" start >"$OUT_DIR/aushd.log" 2>&1 &

for _ in $(seq 1 50); do
  [[ -S "$SOCKET_PATH" ]] && break
  sleep 0.1
done

if [[ ! -S "$SOCKET_PATH" ]]; then
  echo "Error: daemon did not start; see $OUT_DIR/aushd.log"
  exit 1
fi

echo "=== aush frontend with running aushd ==="
hyperfine \
  --warmup 5 \
  --min-runs 30 \
  --export-markdown "$OUT_DIR/frontend.md" \
  --export-json "$OUT_DIR/frontend.json" \
  "$AUSH_BIN --no-rc -c true" \
  "bash -c true" \
  "zsh -f -c true"

echo

echo "=== resident daemon protocol via Criterion ==="
# Restrict the default run to the warm protocol path. The daemon_latency bench
# also contains slower context benchmarks, but this script is intended to answer
# whether a resident aushd execution path can beat shell startup.
cargo bench --bench daemon_latency daemon/exec -- --quiet 2>&1 | tee "$OUT_DIR/daemon-criterion.log"

echo
echo "Daemon comparison artifacts: $OUT_DIR"
