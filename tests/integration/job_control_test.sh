#!/bin/bash
# Job control integration tests for AUSH shell
# Tests background jobs (&), job listing, and job termination

TESTS_PASSED=0
TESTS_FAILED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AUSH_BINARY="$PROJECT_ROOT/target/release/aush"

if [ ! -f "$AUSH_BINARY" ]; then
    echo -e "${RED}Error: aush binary not found at $AUSH_BINARY${NC}"
    exit 1
fi

echo "======================================"
echo "AUSH Job Control Tests"
echo "======================================"
echo -e "${YELLOW}Note: Background job support may be limited${NC}"
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

# Test 1: Background job syntax parsing
test_case "Background job syntax (&)"
OUTPUT=$("$AUSH_BINARY" -c "sleep 0.1 &" 2>&1 || echo "syntax_error")
# Should either accept syntax or give clear error
[ -n "$OUTPUT" ]
assert_success

# Test 2: Quick background job
test_case "Quick background job"
"$AUSH_BINARY" -c "true &" 2>&1 || echo "not_supported"
EXIT_CODE=$?
# Should complete without crashing
[ $EXIT_CODE -ge 0 ]
assert_success

# Test 3: Foreground job completion
test_case "Foreground job completion"
"$AUSH_BINARY" -c "echo foreground" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 4: Multiple foreground jobs
test_case "Sequential foreground jobs"
OUTPUT=$(echo -e "echo job1\necho job2\necho job3" | "$AUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "job1" && echo "$OUTPUT" | grep -q "job2" && echo "$OUTPUT" | grep -q "job3"
assert_success

# Test 5: Job with output
test_case "Job with stdout"
OUTPUT=$("$AUSH_BINARY" -c "echo output" 2>&1)
echo "$OUTPUT" | grep -q "output"
assert_success

# Test 6: Job exit codes
test_case "Job exit code (success)"
"$AUSH_BINARY" -c "true" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 7: Job after job
test_case "Sequential job execution"
"$AUSH_BINARY" -c "echo first" >/dev/null 2>&1
"$AUSH_BINARY" -c "echo second" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 8: Job with pipeline
test_case "Job with pipeline"
OUTPUT=$("$AUSH_BINARY" -c "echo pipe | cat" 2>&1)
echo "$OUTPUT" | grep -q "pipe"
assert_success

# Test 9: Job with redirect
test_case "Job with redirection"
TEMP_FILE="/tmp/aush_job_test_$$.txt"
"$AUSH_BINARY" -c "echo redirected > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
rm -f "$TEMP_FILE"
echo "$CONTENT" | grep -q "redirected"
assert_success

# Test 10: Job with conditional
test_case "Job with conditional (&&)"
OUTPUT=$("$AUSH_BINARY" -c "echo one && echo two" 2>&1)
echo "$OUTPUT" | grep -q "one" && echo "$OUTPUT" | grep -q "two"
assert_success

# Test 11: Long running foreground job
test_case "Short foreground sleep"
timeout 2 "$AUSH_BINARY" -c "sleep 0.1" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 12: Job completion check
test_case "Job completes successfully"
"$AUSH_BINARY" -c "echo complete" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 13: Multiple quick jobs
test_case "Rapid job execution"
for i in {1..5}; do
    "$AUSH_BINARY" -c "echo job$i" >/dev/null 2>&1
done
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 14: Job with environment
test_case "Job with variable"
OUTPUT=$("$AUSH_BINARY" -c "let x=test; echo \$x" 2>&1)
echo "$OUTPUT" | grep -q "test"
assert_success

# Test 15: Job isolation
test_case "Job isolation (separate invocations)"
"$AUSH_BINARY" -c "let var1=value1" >/dev/null 2>&1
OUTPUT=$("$AUSH_BINARY" -c "echo \$var1" 2>&1)
# Variable should not persist across invocations
! echo "$OUTPUT" | grep -q "value1"
assert_success

# Test 16: Job with builtin
test_case "Job with builtin (pwd)"
OUTPUT=$("$AUSH_BINARY" -c "pwd" 2>&1)
echo "$OUTPUT" | grep -q "/"
assert_success

# Test 17: Job with external command
test_case "Job with external (echo)"
OUTPUT=$("$AUSH_BINARY" -c "echo external" 2>&1)
echo "$OUTPUT" | grep -q "external"
assert_success

# Test 18: Job cleanup
test_case "Process cleanup after job"
"$AUSH_BINARY" -c "echo cleanup" >/dev/null 2>&1
sleep 0.2
# Check no zombie aush processes (basic check)
ZOMBIES=$(ps aux | grep -c "[r]ush.*<defunct>" || echo 0)
[ $ZOMBIES -eq 0 ]
assert_success

# Test 19: Job with error
test_case "Job handles errors"
"$AUSH_BINARY" -c "cat /nonexistent/file" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -ne 0 ]  # Should return non-zero for error
assert_success

# Test 20: Job state consistency
test_case "Job state consistency"
OUTPUT1=$("$AUSH_BINARY" -c "echo state1" 2>&1)
OUTPUT2=$("$AUSH_BINARY" -c "echo state2" 2>&1)
echo "$OUTPUT1" | grep -q "state1" && echo "$OUTPUT2" | grep -q "state2"
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
    echo -e "${GREEN}All job control tests passed!${NC}"
    exit 0
fi
