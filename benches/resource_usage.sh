#!/usr/bin/env bash

set -euo pipefail

AUSH_BIN="${1:-./target/release/aush}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

if [[ ! -x "$AUSH_BIN" ]]; then
  echo "Error: AUSH binary not found at $AUSH_BIN" >&2
  echo "Run: cargo build --release --bins" >&2
  exit 1
fi

TIME_BIN="/usr/bin/time"
if [[ ! -x "$TIME_BIN" ]]; then
  echo "Error: /usr/bin/time is required for resource benchmarking" >&2
  exit 1
fi

if ! "$TIME_BIN" -l true >/dev/null 2>/tmp/aush-time-probe.log; then
  echo "Error: /usr/bin/time -l is required for this resource benchmark" >&2
  echo "This script currently targets macOS/BSD time output." >&2
  exit 1
fi

OUT_DIR="${AUSH_BENCH_OUT_DIR:-/tmp/aush-resource}"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
RUN_DIR="$OUT_DIR/$RUN_ID"
mkdir -p "$RUN_DIR"

WORK_DIR="$RUN_DIR/work"
mkdir -p "$WORK_DIR"
cat >"$WORK_DIR/sample.txt" <<'DATA'
alpha
beta
gamma
beta
alpha
DATA

shells=()
labels=()
commands=()

add_shell() {
  local label="$1"
  local bin="$2"
  local template="$3"

  if command -v "$bin" >/dev/null 2>&1; then
    labels+=("$label")
    shells+=("$(command -v "$bin")")
    commands+=("$template")
  else
    echo "skip shell: $label ($bin not found)" >&2
  fi
}

labels+=("aush")
shells+=("$AUSH_BIN")
commands+=("__AUSH__")
add_shell "bash" "bash" "__POSIX__"
add_shell "zsh" "zsh" "__POSIX__"
add_shell "dash" "dash" "__POSIX__"

workload_names=(
  "startup_true"
  "echo"
  "pipeline_tail"
  "small_loop"
  "grep_count_file"
)

workload_command() {
  local mode="$1"
  local workload="$2"

  case "$workload" in
    startup_true)
      printf 'true'
      ;;
    echo)
      printf 'echo hello >/dev/null'
      ;;
    pipeline_tail)
      printf "printf 'one\\ntwo\\nthree\\n' | tail -n 2 >/dev/null"
      ;;
    small_loop)
      printf 'i=0; while [ $i -lt 20 ]; do i=$((i + 1)); echo $i >/dev/null; done'
      ;;
    grep_count_file)
      if [[ "$mode" == "__AUSH__" ]]; then
        printf "grep -c beta '%s' >/dev/null" "$WORK_DIR/sample.txt"
      else
        printf "grep -c beta '%s' >/dev/null" "$WORK_DIR/sample.txt"
      fi
      ;;
    *)
      echo "unknown workload: $workload" >&2
      return 1
      ;;
  esac
}

run_case() {
  local label="$1"
  local shell_bin="$2"
  local mode="$3"
  local workload="$4"
  local script
  script="$(workload_command "$mode" "$workload")"
  local safe_name="${label}_${workload}"
  local stdout_file="$RUN_DIR/$safe_name.stdout"
  local stderr_file="$RUN_DIR/$safe_name.stderr"
  local time_file="$RUN_DIR/$safe_name.time"

  if [[ "$mode" == "__AUSH__" ]]; then
    "$TIME_BIN" -l "$shell_bin" --no-rc -c "$script" >"$stdout_file" 2>"$stderr_file" || {
      local status=$?
      cat "$stderr_file" >&2 || true
      echo "FAIL $label $workload exit=$status" >&2
      return "$status"
    }
  else
    "$TIME_BIN" -l "$shell_bin" -c "$script" >"$stdout_file" 2>"$stderr_file" || {
      local status=$?
      cat "$stderr_file" >&2 || true
      echo "FAIL $label $workload exit=$status" >&2
      return "$status"
    }
  fi

  # /usr/bin/time writes resource data to stderr. Preserve the original file name
  # expected by the parser and keep command stderr in the same raw log.
  mv "$stderr_file" "$time_file"

  /usr/bin/python3 - "$label" "$workload" "$time_file" <<'PY'
import re
import sys

label, workload, path = sys.argv[1:4]
text = open(path, encoding="utf-8", errors="replace").read()

def find(pattern):
    match = re.search(pattern, text)
    return match.group(1) if match else "n/a"

real = find(r"([0-9.]+) real")
user = find(r"([0-9.]+) user")
sys_time = find(r"([0-9.]+) sys")
rss = find(r"(\d+)\s+maximum resident set size")
page_faults = find(r"(\d+)\s+page reclaims")
voluntary = find(r"(\d+)\s+voluntary context switches")
involuntary = find(r"(\d+)\s+involuntary context switches")

rss_mib = "n/a"
if rss != "n/a":
    # macOS reports bytes for maximum resident set size.
    rss_mib = f"{int(rss) / 1024 / 1024:.1f}"

print("\t".join([label, workload, real, user, sys_time, rss_mib, page_faults, voluntary, involuntary]))
PY
}

echo "AUSH resource benchmark"
echo "Binary: $AUSH_BIN"
echo "Output: $RUN_DIR"
echo
echo -e "shell\tworkload\treal_s\tuser_s\tsys_s\tmax_rss_mib\tpage_reclaims\tvoluntary_ctx\tinvoluntary_ctx"
echo "# Note: /usr/bin/time -l has coarse wall-time precision for very short commands; use Criterion/hyperfine for latency."

for index in "${!labels[@]}"; do
  for workload in "${workload_names[@]}"; do
    run_case "${labels[$index]}" "${shells[$index]}" "${commands[$index]}" "$workload"
  done
done

echo
echo "Raw logs: $RUN_DIR"
