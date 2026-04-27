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
if [[ ! -x "$TIME_BIN" ]] || ! "$TIME_BIN" -l true >/dev/null 2>/tmp/aush-large-time-probe.log; then
  echo "Error: macOS/BSD /usr/bin/time -l is required" >&2
  exit 1
fi

OUT_DIR="${AUSH_BENCH_OUT_DIR:-/tmp/aush-large-resource}"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
RUN_DIR="$OUT_DIR/$RUN_ID"
WORK_DIR="$RUN_DIR/work"
mkdir -p "$WORK_DIR/data" "$RUN_DIR"

for i in $(seq 1 200); do
  cat >"$WORK_DIR/data/file$i.txt" <<DATA
alpha $i
beta $i
gamma $i
TODO item $i
DATA
  cat >"$WORK_DIR/data/file$i.json" <<DATA
{"id":$i,"name":"item_$i","active":$((i % 2))}
DATA
done

TEXT_FILES=""
for file in "$WORK_DIR"/data/*.txt; do
  TEXT_FILES="$TEXT_FILES '$file'"
done

shells=("aush")
bins=("$AUSH_BIN")
modes=("__AUSH__")
for shell_name in bash zsh dash; do
  if command -v "$shell_name" >/dev/null 2>&1; then
    shells+=("$shell_name")
    bins+=("$(command -v "$shell_name")")
    modes+=("__POSIX__")
  else
    echo "skip shell: $shell_name not found" >&2
  fi
done

workload_command() {
  local mode="$1"
  local workload="$2"
  case "$workload" in
    agentic_core)
      if [[ "$mode" == "__AUSH__" ]]; then
        printf '%q --no-rc ./benches/workloads/agentic_core.sh' "$AUSH_BIN"
      else
        printf './benches/workloads/agentic_core.sh'
      fi
      ;;
    find_200_files)
      if [[ "$mode" == "__AUSH__" ]]; then
        printf "find '%s/data' -name '*.txt' -print >/dev/null" "$WORK_DIR"
      else
        printf "find '%s/data' -name '*.txt' -print >/dev/null" "$WORK_DIR"
      fi
      ;;
    grep_200_files)
      printf "grep -c TODO%s >/dev/null" "$TEXT_FILES"
      ;;
    loop_1000)
      printf 'i=0; while [ $i -lt 1000 ]; do i=$((i + 1)); :; done'
      ;;
    pipeline_1000)
      printf "seq 1000 | sort -nr | head -n 20 | wc -l >/dev/null"
      ;;
    *)
      echo "unknown workload: $workload" >&2
      return 1
      ;;
  esac
}

run_case() {
  local label="$1"
  local bin="$2"
  local mode="$3"
  local workload="$4"
  local safe="${label}_${workload}"
  local time_file="$RUN_DIR/$safe.time"
  local stdout_file="$RUN_DIR/$safe.stdout"
  local command_script
  command_script="$(workload_command "$mode" "$workload")"

  if [[ "$workload" == "agentic_core" && "$mode" == "__AUSH__" ]]; then
    "$TIME_BIN" -l "$AUSH_BIN" --no-rc ./benches/workloads/agentic_core.sh >"$stdout_file" 2>"$time_file" || {
      local status=$?
      cat "$time_file" >&2 || true
      echo "FAIL $label $workload exit=$status" >&2
      return "$status"
    }
  elif [[ "$workload" == "agentic_core" ]]; then
    if [[ "$label" == "zsh" ]]; then
      "$TIME_BIN" -l "$bin" -f ./benches/workloads/agentic_core.sh >"$stdout_file" 2>"$time_file" || {
        local status=$?
        cat "$time_file" >&2 || true
        echo "FAIL $label $workload exit=$status" >&2
        return "$status"
      }
    else
      "$TIME_BIN" -l "$bin" ./benches/workloads/agentic_core.sh >"$stdout_file" 2>"$time_file" || {
        local status=$?
        cat "$time_file" >&2 || true
        echo "FAIL $label $workload exit=$status" >&2
        return "$status"
      }
    fi
  elif [[ "$mode" == "__AUSH__" ]]; then
    "$TIME_BIN" -l "$bin" --no-rc -c "$command_script" >"$stdout_file" 2>"$time_file" || {
      local status=$?
      cat "$time_file" >&2 || true
      echo "FAIL $label $workload exit=$status" >&2
      return "$status"
    }
  elif [[ "$label" == "zsh" ]]; then
    "$TIME_BIN" -l "$bin" -f -c "$command_script" >"$stdout_file" 2>"$time_file" || {
      local status=$?
      cat "$time_file" >&2 || true
      echo "FAIL $label $workload exit=$status" >&2
      return "$status"
    }
  else
    "$TIME_BIN" -l "$bin" -c "$command_script" >"$stdout_file" 2>"$time_file" || {
      local status=$?
      cat "$time_file" >&2 || true
      echo "FAIL $label $workload exit=$status" >&2
      return "$status"
    }
  fi

  /usr/bin/python3 - "$label" "$workload" "$time_file" <<'PY'
import re
import sys
label, workload, path = sys.argv[1:4]
text = open(path, encoding='utf-8', errors='replace').read()

def find(pattern):
    m = re.search(pattern, text)
    return m.group(1) if m else 'n/a'

real = find(r'([0-9.]+) real')
user = find(r'([0-9.]+) user')
sys_time = find(r'([0-9.]+) sys')
rss = find(r'(\d+)\s+maximum resident set size')
reclaims = find(r'(\d+)\s+page reclaims')
vol = find(r'(\d+)\s+voluntary context switches')
invol = find(r'(\d+)\s+involuntary context switches')
rss_mib = 'n/a' if rss == 'n/a' else f'{int(rss) / 1024 / 1024:.1f}'
print('\t'.join([label, workload, real, user, sys_time, rss_mib, reclaims, vol, invol]))
PY
}

workloads=(agentic_core find_200_files grep_200_files loop_1000 pipeline_1000)

echo "AUSH large workload resource benchmark"
echo "Binary: $AUSH_BIN"
echo "Output: $RUN_DIR"
echo
echo -e "shell\tworkload\treal_s\tuser_s\tsys_s\tmax_rss_mib\tpage_reclaims\tvoluntary_ctx\tinvoluntary_ctx"
echo "# Note: /usr/bin/time -l has coarse wall-time precision for short commands; use hyperfine/Criterion for latency."
for i in "${!shells[@]}"; do
  for workload in "${workloads[@]}"; do
    run_case "${shells[$i]}" "${bins[$i]}" "${modes[$i]}" "$workload"
  done
done

echo
echo "Raw logs: $RUN_DIR"
