#!/bin/sh
set -u

WORK_DIR=$(mktemp -d)
trap 'rm -rf "$WORK_DIR"' EXIT INT TERM

mkdir -p "$WORK_DIR/src" "$WORK_DIR/logs" "$WORK_DIR/data"
printf 'fn main() {\n    println!("hello");\n    // TODO: tighten parser recovery\n}\n' > "$WORK_DIR/src/main.rs"
printf 'pub fn answer() -> i32 { 42 }\n// FIXME: replace stub\n' > "$WORK_DIR/src/lib.rs"
printf 'INFO boot\nWARN retrying request\nERROR failed to parse fixture\nINFO shutdown\n' > "$WORK_DIR/logs/test.log"
printf 'alpha 3\nbeta 1\ngamma 2\n' > "$WORK_DIR/data/items.txt"

cd "$WORK_DIR" || exit 1

cat logs/test.log | grep ERROR | wc -l | tr -d ' '
printf 'src/lib.rs\nsrc/main.rs\n' | wc -l | tr -d ' '
files=$(printf 'src/lib.rs\nsrc/main.rs\n' | wc -l | tr -d ' '); echo "rust files: $files"
false; echo after-false; true; echo done
for f in src/lib.rs src/main.rs; do echo "$f"; done
echo alpha > result.txt; echo beta >> result.txt; cat result.txt
if test -f src/main.rs; then echo exists; else echo missing; fi
echo src/lib.rs src/main.rs | wc -w | tr -d ' '
