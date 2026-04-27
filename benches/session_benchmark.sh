#!/bin/bash
# Session-based Benchmark: AUSH vs Zsh
# Compares running commands in a single session vs spawning for each command

set -e

AUSH="${AUSH:-./target/release/aush}"
if [ ! -x "$AUSH" ] && [ -x ./target/release/aush ]; then
    AUSH=./target/release/aush
fi
ZSH="/bin/zsh"
RUNS=10

if [ ! -x "$AUSH" ]; then
    echo "Error: AUSH binary not found at $AUSH"
    echo "Run: cargo build --release"
    exit 1
fi

echo "🚀 Session-Based Shell Benchmark"
echo "   Compares: Single session vs spawning for each command"
echo "   Runs: $RUNS"
echo ""

# Test 1: Spawn shell for EACH command (current benchmark method)
echo "📊 Method 1: Spawn shell per command (aush -c \"cmd\")"
echo "   This includes shell startup overhead for EVERY command"
echo ""

test_spawn_per_command() {
    local shell=$1
    local total=0

    for ((i=1; i<=$RUNS; i++)); do
        local start=$(python3 -c "import time; print(time.perf_counter())")

        # Run 5 commands, spawning shell each time
        $shell -c "pwd" > /dev/null 2>&1
        $shell -c "git status" > /dev/null 2>&1
        $shell -c "ls" > /dev/null 2>&1
        $shell -c "echo \$HOME" > /dev/null 2>&1
        $shell -c "echo test" > /dev/null 2>&1

        local end=$(python3 -c "import time; print(time.perf_counter())")
        local elapsed=$(python3 -c "print(($end - $start) * 1000)")
        total=$(python3 -c "print($total + $elapsed)")
    done

    local avg=$(python3 -c "print($total / $RUNS)")
    echo "$avg"
}

printf "  AUSH (5 commands, spawn each): "
rush_spawn_time=$(test_spawn_per_command "$AUSH")
printf "%8.2f ms total (%6.2f ms per command)\n" "$rush_spawn_time" "$(python3 -c "print($rush_spawn_time / 5)")"

printf "  Zsh  (5 commands, spawn each): "
zsh_spawn_time=$(test_spawn_per_command "$ZSH")
printf "%8.2f ms total (%6.2f ms per command)\n" "$zsh_spawn_time" "$(python3 -c "print($zsh_spawn_time / 5)")"

speedup_spawn=$(python3 -c "print($zsh_spawn_time / $rush_spawn_time)")
printf "  Speedup: %.2fx " "$speedup_spawn"
if [ $(python3 -c "print(1 if $speedup_spawn > 1.0 else 0)") -eq 1 ]; then
    printf "🏆 AUSH\n"
else
    printf "(Zsh faster)\n"
fi

echo ""

# Test 2: Run all commands in a SINGLE session (claude-code method)
echo "📊 Method 2: Single session (shell << EOF)"
echo "   This is how claude-code actually works - one persistent session"
echo ""

test_single_session() {
    local shell=$1
    local script="pwd > /dev/null
git status > /dev/null
ls > /dev/null
echo \$HOME > /dev/null
echo test > /dev/null"

    local total=0
    for ((i=1; i<=$RUNS; i++)); do
        local start=$(python3 -c "import time; print(time.perf_counter())")

        # Run all 5 commands in a single shell session
        echo "$script" | $shell > /dev/null 2>&1

        local end=$(python3 -c "import time; print(time.perf_counter())")
        local elapsed=$(python3 -c "print(($end - $start) * 1000)")
        total=$(python3 -c "print($total + $elapsed)")
    done

    local avg=$(python3 -c "print($total / $RUNS)")
    echo "$avg"
}

printf "  AUSH (5 commands, 1 session):  "
rush_session_time=$(test_single_session "$AUSH")
printf "%8.2f ms total (%6.2f ms per command)\n" "$rush_session_time" "$(python3 -c "print($rush_session_time / 5)")"

printf "  Zsh  (5 commands, 1 session):  "
zsh_session_time=$(test_single_session "$ZSH")
printf "%8.2f ms total (%6.2f ms per command)\n" "$zsh_session_time" "$(python3 -c "print($zsh_session_time / 5)")"

speedup_session=$(python3 -c "print($zsh_session_time / $rush_session_time)")
printf "  Speedup: %.2fx " "$speedup_session"
if [ $(python3 -c "print(1 if $speedup_session > 1.0 else 0)") -eq 1 ]; then
    printf "🏆 AUSH\n"
else
    printf "(Zsh faster)\n"
fi

echo ""
echo "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "=" "="
echo ""
echo "📈 Analysis:"
echo ""

# Calculate shell startup overhead
rush_startup=$(python3 -c "print(($rush_spawn_time - $rush_session_time) / 5)")
zsh_startup=$(python3 -c "print(($zsh_spawn_time - $zsh_session_time) / 5)")

printf "  AUSH shell startup overhead: %6.2f ms per spawn\n" "$rush_startup"
printf "  Zsh  shell startup overhead: %6.2f ms per spawn\n" "$zsh_startup"
printf "  Difference:                  %6.2f ms (%.1fx slower)\n" \
    "$(python3 -c "print($rush_startup - $zsh_startup)")" \
    "$(python3 -c "print($rush_startup / $zsh_startup)")"

echo ""
printf "  AUSH command execution:      %6.2f ms per command (in session)\n" "$(python3 -c "print($rush_session_time / 5)")"
printf "  Zsh  command execution:      %6.2f ms per command (in session)\n" "$(python3 -c "print($zsh_session_time / 5)")"
printf "  Difference:                  %6.2f ms (%.2fx)\n" \
    "$(python3 -c "print(($rush_session_time - $zsh_session_time) / 5)")" \
    "$(python3 -c "print(($rush_session_time / 5) / ($zsh_session_time / 5))")"

echo ""
echo "💡 For Claude Code:"
echo "   Claude-code uses a PERSISTENT shell session (Method 2)"
echo "   - Shell startup cost is paid ONCE at the beginning"
echo "   - Every command after that uses the session speed"

if [ $(python3 -c "print(1 if $speedup_session >= 1.0 else 0)") -eq 1 ]; then
    percent_faster=$(python3 -c "print(int(($speedup_session - 1.0) * 100))")
    echo "   - AUSH is ${percent_faster}% FASTER than Zsh for in-session commands 🚀"
else
    percent_slower=$(python3 -c "print(int((1.0 - $speedup_session) * 100))")
    echo "   - AUSH is ${percent_slower}% slower than Zsh for in-session commands"
fi

echo ""
echo "✅ Benchmark complete!"
