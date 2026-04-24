#!/bin/bash
# Non-TTY specific integration tests for rush shell
# Tests scenarios where stdin is not a TTY (piped input, file redirection, cron jobs, CI/CD)

TESTS_PASSED=0
TESTS_FAILED=0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Determine aush binary path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AUSH_BINARY="$PROJECT_ROOT/target/release/aush"
if [ ! -f "$AUSH_BINARY" ] && [ -f "$PROJECT_ROOT/target/release/rush" ]; then
    AUSH_BINARY="$PROJECT_ROOT/target/release/rush"
fi
RUSH_BINARY="$AUSH_BINARY"

if [ ! -f "$RUSH_BINARY" ]; then
    echo -e "${RED}Error: aush binary not found at $RUSH_BINARY${NC}"
    exit 1
fi

echo "======================================"
echo "Rush Non-TTY Integration Tests"
echo "======================================"
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

# Test 1: Piped input (non-TTY stdin)
test_case "Piped input to rush"
OUTPUT=$(echo "echo piped" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "piped"
assert_success

# Test 2: Multiple commands via pipe
test_case "Multiple commands via pipe"
OUTPUT=$(echo -e "echo first\necho second" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "first" && echo "$OUTPUT" | grep -q "second"
assert_success

# Test 3: Stdin redirection from file
test_case "Stdin from file"
TEMP_FILE="/tmp/rush_nontty_$$.sh"
echo "echo from file" > "$TEMP_FILE"
OUTPUT=$("$RUSH_BINARY" < "$TEMP_FILE" 2>&1)
rm -f "$TEMP_FILE"
echo "$OUTPUT" | grep -q "from file"
assert_success

# Test 4: Here-document simulation
test_case "Here-document simulation"
OUTPUT=$("$RUSH_BINARY" <<EOF
echo line1
echo line2
pwd
EOF
)
echo "$OUTPUT" | grep -q "line1" && echo "$OUTPUT" | grep -q "line2"
assert_success

# Test 5: Empty input via pipe
test_case "Empty input via pipe"
OUTPUT=$(echo "" | "$RUSH_BINARY" 2>&1)
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 6: Whitespace-only input
test_case "Whitespace-only input"
OUTPUT=$(echo -e "   \n\t\n   " | "$RUSH_BINARY" 2>&1)
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 7: Comments-only input
test_case "Comments-only input"
OUTPUT=$(echo -e "# Comment 1\n# Comment 2" | "$RUSH_BINARY" 2>&1)
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 8: Mixed commands and comments
test_case "Mixed commands and comments"
OUTPUT=$(echo -e "# Start\necho test\n# End" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "test"
assert_success

# Test 9: Long input stream
test_case "Long input stream (50 commands)"
COMMANDS=""
for i in {1..50}; do
    COMMANDS="${COMMANDS}echo line${i}\n"
done
OUTPUT=$(echo -e "$COMMANDS" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "line1" && echo "$OUTPUT" | grep -q "line50"
assert_success

# Test 10: Builtin commands via pipe
test_case "Builtin commands via pipe"
OUTPUT=$(echo "pwd" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "/"
assert_success

# Test 11: External commands via pipe
test_case "External commands via pipe"
OUTPUT=$(echo "echo external" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "external"
assert_success

# Test 12: Pipeline within non-TTY mode
test_case "Pipeline in non-TTY mode"
OUTPUT=$(echo "echo pipe | cat" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "pipe"
assert_success

# Test 13: Redirection within non-TTY mode
test_case "Redirection in non-TTY mode"
TEMP_OUT="/tmp/rush_nontty_redirect_$$.txt"
echo "echo redirected > $TEMP_OUT" | "$RUSH_BINARY" 2>&1
CONTENT=$(cat "$TEMP_OUT" 2>/dev/null || echo "")
rm -f "$TEMP_OUT"
echo "$CONTENT" | grep -q "redirected"
assert_success

# Test 14: Exit codes preserved in non-TTY mode
test_case "Exit codes in non-TTY mode"
echo "echo success" | "$RUSH_BINARY" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 15: Conditional execution in non-TTY
test_case "Conditional in non-TTY mode"
OUTPUT=$(echo "echo first && echo second" | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "first" && echo "$OUTPUT" | grep -q "second"
assert_success

# Test 16: Simulate cron job scenario
test_case "Cron job simulation"
CRON_SCRIPT="/tmp/rush_cron_$$.sh"
cat > "$CRON_SCRIPT" <<'CRONEOF'
# Simulated cron job
echo "Cron job started"
pwd
echo "Cron job completed"
CRONEOF
OUTPUT=$("$RUSH_BINARY" < "$CRON_SCRIPT" 2>&1)
rm -f "$CRON_SCRIPT"
echo "$OUTPUT" | grep -q "Cron job started" && echo "$OUTPUT" | grep -q "Cron job completed"
assert_success

# Test 17: Simulate CI/CD pipeline
test_case "CI/CD pipeline simulation"
CI_SCRIPT="/tmp/rush_ci_$$.sh"
cat > "$CI_SCRIPT" <<'CIEOF'
# CI/CD Test Script
echo "Running tests..."
echo "All tests passed"
CIEOF
OUTPUT=$("$RUSH_BINARY" < "$CI_SCRIPT" 2>&1)
rm -f "$CI_SCRIPT"
echo "$OUTPUT" | grep -q "Running tests" && echo "$OUTPUT" | grep -q "All tests passed"
assert_success

# Test 18: EOF handling
test_case "EOF handling"
OUTPUT=$(echo "echo before EOF" | "$RUSH_BINARY" 2>&1)
EXIT_CODE=$?
echo "$OUTPUT" | grep -q "before EOF" && [ $EXIT_CODE -eq 0 ]
assert_success

# Test 19: Rapid input stream
test_case "Rapid input stream"
OUTPUT=$(for i in {1..10}; do echo "echo cmd$i"; done | "$RUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "cmd1" && echo "$OUTPUT" | grep -q "cmd10"
assert_success

# Test 20: Input with various line endings
test_case "Various line endings"
printf "echo unix\necho test\n" | "$RUSH_BINARY" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
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
    echo -e "${GREEN}All non-TTY tests passed!${NC}"
    exit 0
fi
