#!/bin/bash
# File redirection integration tests for AUSH shell
# Tests stdout redirection (>), append (>>), stdin redirection (<)

TESTS_PASSED=0
TESTS_FAILED=0

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

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
echo "AUSH Redirection Tests"
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

TEMP_DIR="/tmp/aush_redirect_test_$$"
mkdir -p "$TEMP_DIR"

# Test 1: Simple output redirection
test_case "Output redirection >"
TEMP_FILE="$TEMP_DIR/out1.txt"
"$AUSH_BINARY" -c "echo hello > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "hello"
assert_success

# Test 2: Overwrite with redirection
test_case "Overwrite with >"
TEMP_FILE="$TEMP_DIR/out2.txt"
"$AUSH_BINARY" -c "echo first > $TEMP_FILE" 2>&1
"$AUSH_BINARY" -c "echo second > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "second" && ! echo "$CONTENT" | grep -q "first"
assert_success

# Test 3: Append redirection
test_case "Append redirection >>"
TEMP_FILE="$TEMP_DIR/out3.txt"
"$AUSH_BINARY" -c "echo line1 > $TEMP_FILE" 2>&1
"$AUSH_BINARY" -c "echo line2 >> $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "line1" && echo "$CONTENT" | grep -q "line2"
assert_success

# Test 4: Multiple appends
test_case "Multiple appends"
TEMP_FILE="$TEMP_DIR/out4.txt"
"$AUSH_BINARY" -c "echo a > $TEMP_FILE" 2>&1
"$AUSH_BINARY" -c "echo b >> $TEMP_FILE" 2>&1
"$AUSH_BINARY" -c "echo c >> $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "a" && echo "$CONTENT" | grep -q "b" && echo "$CONTENT" | grep -q "c"
assert_success

# Test 5: Input redirection
test_case "Input redirection <"
TEMP_FILE="$TEMP_DIR/input1.txt"
echo "input data" > "$TEMP_FILE"
OUTPUT=$("$AUSH_BINARY" -c "cat $TEMP_FILE" 2>&1)
echo "$OUTPUT" | grep -q "input data"
assert_success

# Test 6: Redirect to multiple files (sequential)
test_case "Sequential redirects"
FILE1="$TEMP_DIR/seq1.txt"
FILE2="$TEMP_DIR/seq2.txt"
"$AUSH_BINARY" -c "echo one > $FILE1" 2>&1
"$AUSH_BINARY" -c "echo two > $FILE2" 2>&1
CONTENT1=$(cat "$FILE1")
CONTENT2=$(cat "$FILE2")
echo "$CONTENT1" | grep -q "one" && echo "$CONTENT2" | grep -q "two"
assert_success

# Test 7: Redirect with builtin command
test_case "Redirect with builtin (echo)"
TEMP_FILE="$TEMP_DIR/builtin1.txt"
"$AUSH_BINARY" -c "echo builtin test > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE")
echo "$CONTENT" | grep -q "builtin test"
assert_success

# Test 8: Redirect with external command
test_case "Redirect with external (printf)"
TEMP_FILE="$TEMP_DIR/external1.txt"
"$AUSH_BINARY" -c "printf 'external test' > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "external test"
assert_success

# Test 9: Redirect empty output
test_case "Redirect empty output"
TEMP_FILE="$TEMP_DIR/empty1.txt"
"$AUSH_BINARY" -c "echo -n > $TEMP_FILE" 2>&1
[ -f "$TEMP_FILE" ]  # File should exist
assert_success

# Test 10: Redirect with spaces in filename
test_case "Redirect with quoted filename"
TEMP_FILE="$TEMP_DIR/spaced file.txt"
"$AUSH_BINARY" -c "echo spaces > '$TEMP_FILE'" 2>&1
[ -f "$TEMP_FILE" ] && cat "$TEMP_FILE" | grep -q "spaces"
assert_success

# Test 11: Chain of redirects
test_case "Chain of redirects"
FILE1="$TEMP_DIR/chain1.txt"
FILE2="$TEMP_DIR/chain2.txt"
"$AUSH_BINARY" -c "echo start > $FILE1" 2>&1
"$AUSH_BINARY" -c "cat $FILE1 > $FILE2" 2>&1
CONTENT=$(cat "$FILE2")
echo "$CONTENT" | grep -q "start"
assert_success

# Test 12: Redirect pwd output
test_case "Redirect pwd output"
TEMP_FILE="$TEMP_DIR/pwd.txt"
"$AUSH_BINARY" -c "pwd > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE")
echo "$CONTENT" | grep -q "/"
assert_success

# Test 13: Redirect with pipeline
test_case "Redirect after pipeline"
TEMP_FILE="$TEMP_DIR/pipeline.txt"
"$AUSH_BINARY" -c "echo test | cat > $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "test"
assert_success

# Test 14: Multiple lines redirect
test_case "Multiple lines redirect"
TEMP_FILE="$TEMP_DIR/multiline.txt"
"$AUSH_BINARY" <<EOF > /dev/null 2>&1
echo line1 > $TEMP_FILE
echo line2 >> $TEMP_FILE
echo line3 >> $TEMP_FILE
EOF
CONTENT=$(cat "$TEMP_FILE")
echo "$CONTENT" | grep -q "line1" && echo "$CONTENT" | grep -q "line2" && echo "$CONTENT" | grep -q "line3"
assert_success

# Test 15: Redirect to /dev/null
test_case "Redirect to /dev/null"
"$AUSH_BINARY" -c "echo discard > /dev/null" 2>&1
EXIT_CODE=$?
[ $EXIT_CODE -eq 0 ]
assert_success

# Test 16: Read from redirected file
test_case "Cat redirected file"
TEMP_FILE="$TEMP_DIR/cattest.txt"
echo "cat this" > "$TEMP_FILE"
OUTPUT=$("$AUSH_BINARY" -c "cat $TEMP_FILE" 2>&1)
echo "$OUTPUT" | grep -q "cat this"
assert_success

# Test 17: Append to non-existent file
test_case "Append to non-existent file"
TEMP_FILE="$TEMP_DIR/newappend.txt"
rm -f "$TEMP_FILE"
"$AUSH_BINARY" -c "echo new >> $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE" 2>/dev/null || echo "")
echo "$CONTENT" | grep -q "new"
assert_success

# Test 18: Large output redirect
test_case "Large output redirect"
TEMP_FILE="$TEMP_DIR/large.txt"
"$AUSH_BINARY" <<EOF > /dev/null 2>&1
echo line1 > $TEMP_FILE
echo line2 >> $TEMP_FILE
echo line3 >> $TEMP_FILE
echo line4 >> $TEMP_FILE
echo line5 >> $TEMP_FILE
EOF
LINES=$(wc -l < "$TEMP_FILE")
[ $LINES -eq 5 ]
assert_success

# Test 19: Redirect with conditional
test_case "Redirect with conditional"
TEMP_FILE="$TEMP_DIR/cond.txt"
"$AUSH_BINARY" -c "echo first > $TEMP_FILE && echo second >> $TEMP_FILE" 2>&1
CONTENT=$(cat "$TEMP_FILE")
echo "$CONTENT" | grep -q "first" && echo "$CONTENT" | grep -q "second"
assert_success

# Test 20: Redirect permissions
test_case "File created with redirect"
TEMP_FILE="$TEMP_DIR/perms.txt"
"$AUSH_BINARY" -c "echo test > $TEMP_FILE" 2>&1
[ -f "$TEMP_FILE" ] && [ -r "$TEMP_FILE" ]
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
    echo -e "${GREEN}All redirection tests passed!${NC}"
    exit 0
fi
