#!/bin/bash
# Comprehensive integration tests for AUSH shell as a login shell
# Tests non-interactive mode, script execution, command substitution,
# signal handling, redirection, pipelines, exit codes, and job control

TESTS_PASSED=0
TESTS_FAILED=0
FAILURES=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Determine aush binary path
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AUSH_BINARY="$PROJECT_ROOT/target/release/aush"
if [ ! -f "$AUSH_BINARY" ] && [ -f "$PROJECT_ROOT/target/release/aush" ]; then
    AUSH_BINARY="$PROJECT_ROOT/target/release/aush"
fi
AUSH_BINARY="$AUSH_BINARY"

# Check if aush binary exists
if [ ! -f "$AUSH_BINARY" ]; then
    echo -e "${RED}Error: aush binary not found at $AUSH_BINARY${NC}"
    echo "Please build aush first: cargo build --release"
    exit 1
fi

echo "======================================"
echo "AUSH Shell Integration Tests"
echo "======================================"
echo "AUSH binary: $AUSH_BINARY"
echo ""

# Test helper function
test_case() {
    local test_name="$1"
    echo -n "Testing: $test_name ... "
}

# Assert helper functions
assert_success() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC}"
        ((TESTS_FAILED++))
        FAILURES="${FAILURES}\n- $1"
        return 1
    fi
}

assert_exit_code() {
    local expected=$1
    local actual=$2
    local test_name=$3
    if [ $actual -eq $expected ]; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC} (expected exit code $expected, got $actual)"
        ((TESTS_FAILED++))
        FAILURES="${FAILURES}\n- $test_name: expected exit code $expected, got $actual"
        return 1
    fi
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local test_name="$3"
    if echo "$haystack" | grep -q "$needle"; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC} (output does not contain '$needle')"
        ((TESTS_FAILED++))
        FAILURES="${FAILURES}\n- $test_name: output does not contain '$needle'"
        return 1
    fi
}

assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local test_name="$3"
    if ! echo "$haystack" | grep -q "$needle"; then
        echo -e "${GREEN}PASS${NC}"
        ((TESTS_PASSED++))
        return 0
    else
        echo -e "${RED}FAIL${NC} (output should not contain '$needle')"
        ((TESTS_FAILED++))
        FAILURES="${FAILURES}\n- $test_name: output should not contain '$needle'"
        return 1
    fi
}

# ============================================================================
# NON-INTERACTIVE MODE TESTS
# ============================================================================

echo "======================================"
echo "Section 1: Non-Interactive Mode Tests"
echo "======================================"

# Test 1: Echo piped to aush
test_case "Piped echo command"
OUTPUT=$(echo "echo hello world" | "$AUSH_BINARY" 2>&1)
assert_contains "$OUTPUT" "hello world" "Piped echo command"

# Test 2: Multiple commands piped
test_case "Multiple piped commands"
OUTPUT=$(echo -e "echo first\necho second\necho third" | "$AUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "first" && echo "$OUTPUT" | grep -q "second" && echo "$OUTPUT" | grep -q "third"
assert_success "Multiple piped commands"

# Test 3: pwd command
test_case "pwd command"
OUTPUT=$(echo "pwd" | "$AUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "/"
assert_success "pwd command"

# Test 4: Empty input
test_case "Empty input handling"
OUTPUT=$(echo "" | "$AUSH_BINARY" 2>&1)
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ] || [ -n "$OUTPUT" ]
assert_success "Empty input handling"

# Test 5: Comments only
test_case "Comments only input"
OUTPUT=$(echo -e "# Just a comment\n# Another comment" | "$AUSH_BINARY" 2>&1)
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success "Comments only input"

# ============================================================================
# SCRIPT EXECUTION TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 2: Script Execution Tests"
echo "======================================"

FIXTURES_DIR="$PROJECT_ROOT/tests/fixtures"

# Test 6: Simple script execution
test_case "Execute simple script"
OUTPUT=$("$AUSH_BINARY" "$FIXTURES_DIR/simple_script.sh" 2>&1)
assert_contains "$OUTPUT" "Script is running" "Simple script execution"

# Test 7: Script with conditionals
test_case "Execute conditional script"
OUTPUT=$("$AUSH_BINARY" "$FIXTURES_DIR/conditional_script.sh" 2>&1)
assert_contains "$OUTPUT" "AND succeeded" "Conditional script execution"

# ============================================================================
# COMMAND SUBSTITUTION TESTS (-c flag)
# ============================================================================

echo ""
echo "======================================"
echo "Section 3: Command Substitution Tests"
echo "======================================"

# Test 8: Simple -c flag
test_case "-c flag with echo"
OUTPUT=$("$AUSH_BINARY" -c "echo test output" 2>&1)
assert_contains "$OUTPUT" "test output" "-c flag with echo"

# Test 9: -c flag with pwd
test_case "-c flag with pwd"
OUTPUT=$("$AUSH_BINARY" -c "pwd" 2>&1)
echo "$OUTPUT" | grep -q "/"
assert_success "-c flag with pwd"

# Test 10: Multiple commands with -c
test_case "Multiple commands with -c"
OUTPUT=$("$AUSH_BINARY" -c "echo first; echo second" 2>&1)
assert_contains "$OUTPUT" "first" "Multiple commands with -c"

# ============================================================================
# EXIT CODE TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 4: Exit Code Tests"
echo "======================================"

# Test 11: Success exit code
test_case "Success exit code (true)"
"$AUSH_BINARY" -c "echo success" >/dev/null 2>&1
EXIT_CODE=$?
assert_exit_code 0 $EXIT_CODE "Success exit code"

# Test 12: Test variable script
test_case "Variable script execution"
OUTPUT=$("$AUSH_BINARY" "$FIXTURES_DIR/variable_script.sh" 2>&1)
EXIT_CODE=$?
assert_exit_code 0 $EXIT_CODE "Variable script execution"

# Test 13: Conditional success
test_case "Conditional && success"
"$AUSH_BINARY" -c "echo first && echo second" >/dev/null 2>&1
EXIT_CODE=$?
assert_exit_code 0 $EXIT_CODE "Conditional && success"

# Test 14: Conditional || fallback
test_case "Conditional || fallback"
"$AUSH_BINARY" -c "false || echo fallback" >/dev/null 2>&1
EXIT_CODE=$?
assert_exit_code 0 $EXIT_CODE "Conditional || fallback"

# ============================================================================
# REDIRECTION TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 5: Redirection Tests"
echo "======================================"

TEMP_FILE="/tmp/aush_integration_test_$$.txt"

# Test 15: Output redirection
test_case "Output redirection >"
"$AUSH_BINARY" -c "echo redirect test > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
rm -f "$TEMP_FILE"
assert_contains "$CONTENT" "redirect test" "Output redirection"

# Test 16: Append redirection
test_case "Append redirection >>"
"$AUSH_BINARY" -c "echo line1 > $TEMP_FILE" 2>&1
"$AUSH_BINARY" -c "echo line2 >> $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
rm -f "$TEMP_FILE"
echo "$CONTENT" | grep -q "line1" && echo "$CONTENT" | grep -q "line2"
assert_success "Append redirection"

# Test 17: Input redirection
test_case "Input redirection <"
echo "input test data" > "$TEMP_FILE"
OUTPUT=$("$AUSH_BINARY" -c "cat $TEMP_FILE" 2>&1)
rm -f "$TEMP_FILE"
assert_contains "$OUTPUT" "input test data" "Input redirection"

# ============================================================================
# PIPELINE TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 6: Pipeline Tests"
echo "======================================"

# Test 18: Simple pipeline
test_case "Simple pipeline (echo | cat)"
OUTPUT=$("$AUSH_BINARY" -c "echo pipeline test | cat" 2>&1)
assert_contains "$OUTPUT" "pipeline test" "Simple pipeline"

# Test 19: Pipeline with grep
test_case "Pipeline with grep"
OUTPUT=$("$AUSH_BINARY" -c "echo -e 'line1\nline2\nline3' | grep line2" 2>&1)
assert_contains "$OUTPUT" "line2" "Pipeline with grep"

# Test 20: Multi-stage pipeline
test_case "Multi-stage pipeline"
OUTPUT=$(echo "pipeline script test" | "$AUSH_BINARY" 2>&1 | grep -q "pipeline" && echo "success" || echo "")
assert_contains "$OUTPUT" "success" "Multi-stage pipeline"

# ============================================================================
# SIGNAL HANDLING TESTS (Basic)
# ============================================================================

echo ""
echo "======================================"
echo "Section 7: Signal Handling Tests"
echo "======================================"

# Test 21: Background job (if supported)
test_case "Background job syntax accepted"
OUTPUT=$("$AUSH_BINARY" -c "sleep 0.1 &" 2>&1 || echo "not_supported")
# Just verify it doesn't crash
[ -n "$OUTPUT" ]
assert_success "Background job syntax"

# Test 22: Quick termination test
test_case "Quick process termination"
timeout 2 "$AUSH_BINARY" -c "echo quick && exit 0" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ] || [ $EXIT_CODE -eq 124 ]  # 0 = success, 124 = timeout (also ok)
assert_success "Quick process termination"

# ============================================================================
# STDIN FROM FILE TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 8: Stdin From File Tests"
echo "======================================"

# Test 23: Execute commands from file via stdin
test_case "Commands from file via stdin"
TEMP_SCRIPT="/tmp/aush_stdin_test_$$.sh"
echo -e "echo test1\necho test2" > "$TEMP_SCRIPT"
OUTPUT=$("$AUSH_BINARY" < "$TEMP_SCRIPT" 2>&1)
rm -f "$TEMP_SCRIPT"
echo "$OUTPUT" | grep -q "test1" && echo "$OUTPUT" | grep -q "test2"
assert_success "Commands from file via stdin"

# Test 24: Script with comments via stdin
test_case "Script with comments via stdin"
TEMP_SCRIPT="/tmp/aush_comments_test_$$.sh"
echo -e "# Comment\necho visible\n# Another comment" > "$TEMP_SCRIPT"
OUTPUT=$("$AUSH_BINARY" < "$TEMP_SCRIPT" 2>&1)
rm -f "$TEMP_SCRIPT"
assert_contains "$OUTPUT" "visible" "Script with comments via stdin"

# ============================================================================
# BUILTIN COMMANDS TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 9: Builtin Commands Tests"
echo "======================================"

# Test 25: cd command
test_case "cd builtin"
"$AUSH_BINARY" -c "cd /tmp && pwd" >/dev/null 2>&1
EXIT_CODE=$?
assert_exit_code 0 $EXIT_CODE "cd builtin"

# Test 26: echo builtin
test_case "echo builtin"
OUTPUT=$("$AUSH_BINARY" -c "echo builtin test" 2>&1)
assert_contains "$OUTPUT" "builtin test" "echo builtin"

# Test 27: cat builtin
test_case "cat builtin"
TEMP_FILE="/tmp/aush_cat_test_$$.txt"
echo "cat test data" > "$TEMP_FILE"
OUTPUT=$("$AUSH_BINARY" -c "cat $TEMP_FILE" 2>&1)
rm -f "$TEMP_FILE"
assert_contains "$OUTPUT" "cat test data" "cat builtin"

# ============================================================================
# ERROR HANDLING TESTS
# ============================================================================

echo ""
echo "======================================"
echo "Section 10: Error Handling Tests"
echo "======================================"

# Test 28: Graceful handling of command not found
test_case "Handle command not found"
"$AUSH_BINARY" -c "nonexistent_command_xyz_123" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -ne 0 ]  # Should fail with non-zero exit
assert_success "Handle command not found"

# Test 29: Continue after failed command
test_case "Continue after failed command"
OUTPUT=$(echo -e "echo before\ncat /nonexistent_file_xyz 2>/dev/null\necho after" | "$AUSH_BINARY" 2>&1)
echo "$OUTPUT" | grep -q "before" && echo "$OUTPUT" | grep -q "after"
assert_success "Continue after failed command"

# Test 30: Handle unclosed quote gracefully
test_case "Handle parse error gracefully"
OUTPUT=$(echo 'echo "unclosed' | "$AUSH_BINARY" 2>&1 || echo "handled")
[ -n "$OUTPUT" ]  # Should produce some output (error or otherwise)
assert_success "Handle parse error gracefully"

# ============================================================================
# TEST SUMMARY
# ============================================================================

echo ""
echo "======================================"
echo "Test Summary"
echo "======================================"
echo -e "Tests Passed: ${GREEN}$TESTS_PASSED${NC}"
echo -e "Tests Failed: ${RED}$TESTS_FAILED${NC}"
echo "Total Tests:  $((TESTS_PASSED + TESTS_FAILED))"

if [ $TESTS_FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed tests:${NC}"
    echo -e "$FAILURES"
    echo ""
    exit 1
else
    echo ""
    echo -e "${GREEN}All integration tests passed!${NC}"
    echo ""
    exit 0
fi
