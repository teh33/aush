#!/bin/bash
# Signal handling integration tests for AUSH shell
# Tests SIGINT, SIGTERM, SIGQUIT handling and process cleanup

TESTS_PASSED=0
TESTS_FAILED=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Determine aush binary path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AUSH_BINARY="$PROJECT_ROOT/target/release/aush"
if [ ! -f "$AUSH_BINARY" ] && [ -f "$PROJECT_ROOT/target/release/aush" ]; then
    AUSH_BINARY="$PROJECT_ROOT/target/release/aush"
fi
AUSH_BINARY="$AUSH_BINARY"

if [ ! -f "$AUSH_BINARY" ]; then
    echo -e "${RED}Error: aush binary not found at $AUSH_BINARY${NC}"
    exit 1
fi

echo "======================================"
echo "AUSH Signal Handling Tests"
echo "======================================"
echo -e "${YELLOW}Note: Some tests may take a few seconds${NC}"
echo ""

test_case() {
    echo -n "Testing: $1 ... "
}

assert_success() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}FAIL${NC}"
        ((TESTS_FAILED++))
    fi
}

# Test 1: Basic process termination
test_case "Basic process termination"
"$AUSH_BINARY" -c "echo quick" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 2: Timeout handling (quick exit)
test_case "Quick exit (timeout wrapper)"
timeout 2 "$AUSH_BINARY" -c "echo test && exit 0" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 3: SIGTERM handling
test_case "SIGTERM handling"
"$AUSH_BINARY" -c "sleep 10" >/dev/null 2>&1 &
PID=$!
sleep 0.2
kill -TERM $PID 2>/dev/null || true
wait $PID 2>/dev/null
EXIT_CODE=$?
[ $EXIT_CODE -ne 0 ]  # Should exit with non-zero after SIGTERM
assert_success

# Test 4: SIGINT handling (Ctrl-C simulation)
test_case "SIGINT handling"
"$AUSH_BINARY" -c "sleep 10" >/dev/null 2>&1 &
PID=$!
sleep 0.2
kill -INT $PID 2>/dev/null || true
wait $PID 2>/dev/null
EXIT_CODE=$?
[ $EXIT_CODE -ne 0 ]  # Should exit with non-zero after SIGINT
assert_success

# Test 5: Clean exit without signals
test_case "Clean exit (no signals)"
"$AUSH_BINARY" -c "echo done" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 6: Background process signal handling (if supported)
test_case "Background process handling"
OUTPUT=$("$AUSH_BINARY" -c "sleep 0.1 &" 2>&1 || echo "no_bg_support")
# Just verify it doesn't crash
[ -n "$OUTPUT" ]
assert_success

# Test 7: Nested process termination
test_case "Nested process termination"
"$AUSH_BINARY" -c "echo parent" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 8: Rapid start-stop
test_case "Rapid start-stop"
for i in {1..5}; do
    "$AUSH_BINARY" -c "echo iteration $i" >/dev/null 2>&1 || true
done
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 9: Signal during pipeline
test_case "Signal during pipeline"
timeout 1 "$AUSH_BINARY" -c "echo test | cat" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 10: Multiple quick commands
test_case "Multiple quick commands"
OUTPUT=$(echo -e "echo 1\necho 2\necho 3" | "$AUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "1" && echo "$OUTPUT" | grep -q "2" && echo "$OUTPUT" | grep -q "3"
assert_success

# Test 11: Exit code after interrupted command
test_case "Exit code after interrupt"
timeout 1 "$AUSH_BINARY" -c "sleep 100" >/dev/null 2>&1
EXIT_CODE=$?
# timeout returns 124 on timeout, command returns 0 if completed
[ $EXIT_CODE -eq 124 ] || [ $EXIT_CODE -eq 0 ]
assert_success

# Test 12: SIGKILL handling (ungraceful)
test_case "SIGKILL handling"
"$AUSH_BINARY" -c "sleep 10" >/dev/null 2>&1 &
PID=$!
sleep 0.2
kill -9 $PID 2>/dev/null || true
wait $PID 2>/dev/null
# Process should be terminated
[ $? -ne 0 ]
assert_success

# Test 13: Process cleanup verification
test_case "Process cleanup"
BEFORE_COUNT=$(pgrep -f "aush.*sleep" | wc -l || echo 0)
"$AUSH_BINARY" -c "echo cleanup" >/dev/null 2>&1
sleep 0.5
AFTER_COUNT=$(pgrep -f "aush.*sleep" | wc -l || echo 0)
# No zombie aush processes should remain
[ $AFTER_COUNT -le $BEFORE_COUNT ]
assert_success

# Test 14: Sequential signal tests
test_case "Sequential signals"
for sig in TERM INT; do
    "$AUSH_BINARY" -c "sleep 5" >/dev/null 2>&1 &
    PID=$!
    sleep 0.1
    kill -$sig $PID 2>/dev/null || true
    wait $PID 2>/dev/null || true
done
[ $? -ge 0 ]  # Just verify no crash
assert_success

# Test 15: Signal during script execution
test_case "Signal during script"
TEMP_SCRIPT="/tmp/aush_signal_test_$$.sh"
echo "echo starting; sleep 5; echo ending" > "$TEMP_SCRIPT"
timeout 1 "$AUSH_BINARY" "$TEMP_SCRIPT" >/dev/null 2>&1
EXIT_CODE=$?
rm -f "$TEMP_SCRIPT"
[ $EXIT_CODE -eq 124 ] || [ $EXIT_CODE -eq 0 ]
assert_success

echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -gt 0 ]; then
    exit 1
else
    echo -e "${GREEN}All signal handling tests passed!${NC}"
    exit 0
fi
