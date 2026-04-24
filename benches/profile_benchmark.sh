#!/bin/bash
# Profile Rush command execution to identify bottlenecks

set -e

AUSH="${AUSH:-./target/release/aush}"
if [ ! -x "$AUSH" ] && [ -x ./target/release/rush ]; then
    AUSH=./target/release/rush
fi
RUNS=100

if [ ! -x "$AUSH" ]; then
    echo "Error: Rush binary not found"
    exit 1
fi

echo "🔍 Profiling Rush Command Execution"
echo "   Finding bottlenecks in persistent session"
echo "   Runs: $RUNS"
echo ""

# Create a test script with various command types
TEST_SCRIPT=$(cat <<'EOF'
pwd
echo test
echo $HOME
ls > /dev/null
echo $(pwd)
git status > /dev/null
cat README.md > /dev/null
grep -r "use" src --files-with-matches > /dev/null
EOF
)

# Time the full execution
echo "📊 Baseline: Full execution time"
total_time=$(python3 -c "
import subprocess
import time

script = '''$TEST_SCRIPT'''
total = 0

for i in range($RUNS):
    start = time.perf_counter()
    subprocess.run(['$AUSH'], input=script, capture_output=True, text=True, timeout=10)
    end = time.perf_counter()
    total += (end - start) * 1000

avg = total / $RUNS
print(f'{avg:.2f}')
")

num_commands=$(echo "$TEST_SCRIPT" | wc -l | tr -d ' ')
per_command=$(python3 -c "print($total_time / $num_commands)")

echo "  Total time: ${total_time} ms for $num_commands commands"
echo "  Per command: ${per_command} ms"
echo ""

# Now test individual command types to see which are slow
echo "📊 Individual command timing:"

test_command() {
    local name=$1
    local cmd=$2

    local time=$(python3 -c "
import subprocess
import time

total = 0
for i in range($RUNS):
    start = time.perf_counter()
    subprocess.run(['$AUSH'], input='$cmd', capture_output=True, text=True, timeout=10)
    end = time.perf_counter()
    total += (end - start) * 1000

print(f'{total / $RUNS:.2f}')
")

    printf "  %-40s %6.2f ms\n" "$name:" "$time"
}

test_command "Empty (just startup)" "exit"
test_command "Simple echo" "echo test"
test_command "Echo variable" "echo \$HOME"
test_command "pwd (builtin)" "pwd"
test_command "ls (builtin)" "ls > /dev/null"
test_command "cat (builtin)" "cat README.md > /dev/null"
test_command "grep (builtin)" "grep -r 'use' src --files-with-matches > /dev/null"
test_command "git status (builtin)" "git status > /dev/null"
test_command "Command substitution" "echo \$(pwd)"
test_command "Pipe" "echo test | cat"
test_command "Two commands" "pwd; echo test"
test_command "Five commands" "pwd; echo test; echo \$HOME; ls > /dev/null; echo done"

echo ""
echo "🔬 Profiling with perf (Linux) or Instruments (macOS)..."
echo ""

# Try to use system profiler
if command -v perf &> /dev/null; then
    echo "Using perf (Linux):"
    perf record -g --call-graph dwarf -- $AUSH -c "pwd; echo test; git status" > /dev/null 2>&1 || true
    perf report --stdio | head -50 || true
elif command -v sample &> /dev/null; then
    echo "Using sample (macOS):"
    echo "Run: sudo sample $AUSH 5 -file rush_profile.txt"
    echo "Then execute: $AUSH -c 'pwd; echo test; git status' in another terminal"
else
    echo "No profiler available. Install perf (Linux) or use Instruments (macOS)"
fi

echo ""
echo "💡 Potential optimizations to investigate:"
echo "  1. Parser caching - Cache AST for repeated commands"
echo "  2. Git context caching - Don't check git on every command"
echo "  3. Lazy module loading - Defer initialization"
echo "  4. Variable lookup optimization - HashMap vs Vec"
echo "  5. String allocation reduction - Use Cow/Arc"
echo "  6. Executor optimization - Reduce cloning"
echo ""
echo "✅ Profiling complete"
