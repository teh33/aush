#!/bin/bash
# Pipeline integration tests for rush shell
# Tests single pipes, multi-stage pipelines, and pipeline with redirects

TESTS_PASSED=0
TESTS_FAILED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

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
echo "Rush Pipeline Tests"
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

TEMP_DIR="/tmp/rush_pipeline_test_$$"
mkdir -p "$TEMP_DIR"

# Test 1: Simple pipeline (echo | cat)
test_case "Simple pipeline: echo | cat"
OUTPUT=$("$RUSH_BINARY" -c "echo simple | cat" 2>&1)
echo "$OUTPUT" | grep -q "simple"
assert_success

# Test 2: Pipeline with grep
test_case "Pipeline: echo | grep"
OUTPUT=$("$RUSH_BINARY" -c "echo hello world | grep world" 2>&1)
echo "$OUTPUT" | grep -q "world"
assert_success

# Test 3: Pipeline with builtin cat
test_case "Pipeline with builtin cat"
OUTPUT=$("$RUSH_BINARY" -c "echo test | cat" 2>&1)
echo "$OUTPUT" | grep -q "test"
assert_success

# Test 4: Multi-line through pipeline
test_case "Multi-line through pipeline"
TEMP_FILE="$TEMP_DIR/multiline.txt"
echo -e "line1\nline2\nline3" > "$TEMP_FILE"
OUTPUT=$("$RUSH_BINARY" -c "cat $TEMP_FILE | grep line2" 2>&1)
echo "$OUTPUT" | grep -q "line2"
assert_success

# Test 5: Three-stage pipeline
test_case "Three-stage pipeline"
OUTPUT=$("$RUSH_BINARY" -c "echo abc | cat | cat" 2>&1)
echo "$OUTPUT" | grep -q "abc"
assert_success

# Test 6: Pipeline with wc
test_case "Pipeline: echo | wc"
OUTPUT=$("$RUSH_BINARY" -c "echo one two three | wc -w" 2>&1)
echo "$OUTPUT" | grep -q "3"
assert_success

# Test 7: Pipeline preserves data
test_case "Pipeline data integrity"
OUTPUT=$("$RUSH_BINARY" -c "echo 'preserve this data' | cat" 2>&1)
echo "$OUTPUT" | grep -q "preserve this data"
assert_success

# Test 8: Empty pipeline
test_case "Empty input through pipeline"
OUTPUT=$("$RUSH_BINARY" -c "echo | cat" 2>&1)
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 9: Pipeline with head
test_case "Pipeline: cat | head"
TEMP_FILE="$TEMP_DIR/head.txt"
echo -e "1\n2\n3\n4\n5" > "$TEMP_FILE"
OUTPUT=$("$RUSH_BINARY" -c "cat $TEMP_FILE | head -n 2" 2>&1)
LINES=$(echo "$OUTPUT" | wc -l)
[ $LINES -eq 2 ]
assert_success

# Test 10: Pipeline with tail
test_case "Pipeline: cat | tail"
TEMP_FILE="$TEMP_DIR/tail.txt"
echo -e "1\n2\n3\n4\n5" > "$TEMP_FILE"
OUTPUT=$("$RUSH_BINARY" -c "cat $TEMP_FILE | tail -n 2" 2>&1)
echo "$OUTPUT" | grep -q "5"
assert_success

# Test 11: Pipeline with sort
test_case "Pipeline: echo | sort"
OUTPUT=$("$RUSH_BINARY" -c "echo -e 'c\nb\na' | sort" 2>&1)
FIRST_LINE=$(echo "$OUTPUT" | head -n 1)
echo "$FIRST_LINE" | grep -q "a"
assert_success

# Test 12: Pipeline with uniq
test_case "Pipeline: echo | uniq"
OUTPUT=$("$RUSH_BINARY" -c "echo -e 'a\na\nb' | uniq" 2>&1)
LINE_COUNT=$(echo "$OUTPUT" | wc -l)
[ $LINE_COUNT -eq 2 ]
assert_success

# Test 13: Pipeline exit code
test_case "Pipeline exit code"
"$RUSH_BINARY" -c "echo test | cat" >/dev/null 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 14: Pipeline with redirection
test_case "Pipeline with output redirect"
TEMP_FILE="$TEMP_DIR/pipe_redirect.txt"
"$RUSH_BINARY" -c "echo redirect | cat > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE")
echo "$CONTENT" | grep -q "redirect"
assert_success

# Test 15: Pipeline from file
test_case "Pipeline from file input"
TEMP_FILE="$TEMP_DIR/pipe_input.txt"
echo "file content" > "$TEMP_FILE"
OUTPUT=$("$RUSH_BINARY" -c "cat $TEMP_FILE | cat" 2>&1)
echo "$OUTPUT" | grep -q "file content"
assert_success

# Test 16: Complex pipeline chain
test_case "Complex pipeline chain"
OUTPUT=$("$RUSH_BINARY" -c "echo 'a b c d' | cat | cat | cat" 2>&1)
echo "$OUTPUT" | grep -q "a b c d"
assert_success

# Test 17: Pipeline with tr
test_case "Pipeline: echo | tr"
OUTPUT=$("$RUSH_BINARY" -c "echo UPPER | tr '[:upper:]' '[:lower:]'" 2>&1)
echo "$OUTPUT" | grep -q "upper"
assert_success

# Test 18: Pipeline with cut
test_case "Pipeline: echo | cut"
OUTPUT=$("$RUSH_BINARY" -c "echo 'one:two:three' | cut -d: -f2" 2>&1)
echo "$OUTPUT" | grep -q "two"
assert_success

# Test 19: Pipeline preserves newlines
test_case "Pipeline preserves newlines"
OUTPUT=$("$RUSH_BINARY" -c "echo -e 'a\nb' | cat" 2>&1)
LINES=$(echo "$OUTPUT" | wc -l)
[ $LINES -ge 2 ]
assert_success

# Test 20: Pipeline with conditional
test_case "Pipeline with &&"
OUTPUT=$("$RUSH_BINARY" -c "echo first | cat && echo second" 2>&1)
echo "$OUTPUT" | grep -q "first" && echo "$OUTPUT" | grep -q "second"
assert_success

# Cleanup
rm -rf "$TEMP_DIR"

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
    echo -e "${GREEN}All pipeline tests passed!${NC}"
    exit 0
fi
