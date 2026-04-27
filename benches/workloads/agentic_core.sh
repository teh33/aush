#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
cd "$ROOT_DIR"

WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT INT TERM

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

cd "$WORK_DIR"

cat logs/test.log | grep ERROR | wc -l
find src -name '*.rs' | sort | wc -l
files=$(find src -name '*.rs' | wc -l); echo "rust files: $files"
false; echo after-false; true; echo done
for f in src/*.rs; do grep -q FIXME "$f" && echo "$f"; done | sort
echo alpha > result.txt; echo beta >> result.txt; cat result.txt
if test -f src/main.rs; then echo exists; else echo missing; fi
echo src/*.rs | wc -w
