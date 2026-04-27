#!/usr/bin/env bash
#
# AUSH Daemon Benchmark Runner
# =============================
#
# Thin wrapper around `cargo bench --bench daemon_latency`.
# Builds release binaries, starts the daemon, runs Criterion benchmarks,
# and cleans up.
#
# Usage:
#   ./benches/daemon_bench.sh              # Full suite
#   ./benches/daemon_bench.sh --quick      # Shorter measurement times
#   ./benches/daemon_bench.sh --no-build   # Skip cargo build
#   ./benches/daemon_bench.sh --help
#
# The Criterion benchmark (benches/daemon_latency.rs) measures:
#   1. Daemon execution  — warm workers, bincode IPC (primary metric)
#   2. Daemon throughput  — sequential burst (cmds/sec)
#   3. Daemon cold start  — first command after restart
#   4. Cold startup       — aush -c process spawn (reference only)
#   5. Shell comparison   — aush vs bash vs zsh -c (context only)
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RUSHD="${PROJECT_DIR}/target/release/aushd"
SOCKET_PATH="${HOME}/.aush/daemon.sock"

# Colors
if [ -t 1 ]; then
    BOLD='\033[1m' GREEN='\033[0;32m' BLUE='\033[0;34m'
    RED='\033[0;31m' NC='\033[0m'
else
    BOLD='' GREEN='' BLUE='' RED='' NC=''
fi

info()   { printf "${BLUE}[INFO]${NC}  %s\n" "$*"; }
ok()     { printf "${GREEN}[OK]${NC}    %s\n" "$*"; }
err()    { printf "${RED}[ERR]${NC}   %s\n" "$*" >&2; }
header() { printf "\n${BOLD}=== %s ===${NC}\n\n" "$*"; }

DAEMON_PID=""

cleanup() {
    if [ -n "$DAEMON_PID" ]; then
        info "Stopping daemon (PID $DAEMON_PID)..."
        kill "$DAEMON_PID" 2>/dev/null || true
        wait "$DAEMON_PID" 2>/dev/null || true
        rm -f "$SOCKET_PATH"
    fi
}
trap cleanup EXIT

usage() {
    sed -n '2,/^$/s/^# \?//p' "$0"
}

main() {
    local skip_build=false
    local bench_args=()

    while [ $# -gt 0 ]; do
        case "$1" in
            --quick)    bench_args+=("--" "--quick"); shift ;;
            --no-build) skip_build=true; shift ;;
            --help|-h)  usage; exit 0 ;;
            --)         shift; bench_args+=("--" "$@"); break ;;
            *)          err "Unknown option: $1"; usage; exit 1 ;;
        esac
    done

    header "AUSH Daemon Benchmark"
    info "Date: $(date '+%Y-%m-%d %H:%M:%S')"
    info "System: $(uname -srm)"
    info "CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || echo unknown)"

    # Build
    if ! $skip_build; then
        header "Building release binaries"
        (cd "$PROJECT_DIR" && cargo build --release 2>&1) || {
            err "cargo build --release failed"
            exit 1
        }
        ok "Build complete"
    fi

    # Verify binaries
    for bin in "$PROJECT_DIR/target/release/aush" "$RUSHD"; do
        [ -x "$bin" ] || { err "Missing: $bin"; exit 1; }
    done

    # Start daemon (Criterion benchmarks expect it running)
    header "Starting daemon"
    if [ -S "$SOCKET_PATH" ]; then
        "$RUSHD" stop 2>/dev/null || true
        sleep 0.5
    fi

    "$RUSHD" start &
    DAEMON_PID=$!

    # Wait for socket
    for _ in $(seq 1 50); do
        [ -S "$SOCKET_PATH" ] && break
        sleep 0.1
    done
    [ -S "$SOCKET_PATH" ] || { err "Daemon failed to start"; exit 1; }
    ok "Daemon running (PID $DAEMON_PID)"

    # Run benchmarks
    header "Running Criterion benchmarks"
    (cd "$PROJECT_DIR" && cargo bench --bench daemon_latency "${bench_args[@]}")

    ok "Benchmarks complete"
    info "Results in: ${PROJECT_DIR}/target/criterion/"
    info "HTML report: ${PROJECT_DIR}/target/criterion/report/index.html"
}

main "$@"
