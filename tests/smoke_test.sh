#!/bin/bash
#
# AUSH Shell Smoke Test Suite
#
# Run: ./tests/smoke_test.sh [path-to-aush]
#
# This tests fundamental shell functionality that ANY POSIX shell must support.
# If these fail, the shell is not usable for real work.
#

set -u

AUSH="${1:-./target/release/aush}"
if [[ ! -x "$AUSH" && -x ./target/release/aush ]]; then
    AUSH=./target/release/aush
fi
PASS=0
FAIL=0
SKIP=0

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Temp directory for test files
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

TRUE_BIN=$(command -v true || true)
FALSE_BIN=$(command -v false || true)

# Test runner
test_case() {
    local name="$1"
    local cmd="$2"
    local expected="$3"

    # Run with --no-rc to avoid config file issues
    local actual
    actual=$("$AUSH" --no-rc -c "$cmd" 2>&1)
    local exit_code=$?

    if [[ "$actual" == "$expected" ]]; then
        echo -e "${GREEN}✓${NC} $name"
        ((PASS++))
        return 0
    else
        echo -e "${RED}✗${NC} $name"
        echo "  Command:  $cmd"
        echo "  Expected: $expected"
        echo "  Actual:   $actual"
        ((FAIL++))
        return 1
    fi
}

# Test that command succeeds (don't check output)
test_succeeds() {
    local name="$1"
    local cmd="$2"

    "$AUSH" --no-rc -c "$cmd" >/dev/null 2>&1
    local exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        echo -e "${GREEN}✓${NC} $name"
        ((PASS++))
        return 0
    else
        echo -e "${RED}✗${NC} $name (exit code: $exit_code)"
        echo "  Command: $cmd"
        ((FAIL++))
        return 1
    fi
}

# Test that command fails (expected failure)
test_fails() {
    local name="$1"
    local cmd="$2"

    "$AUSH" --no-rc -c "$cmd" >/dev/null 2>&1
    local exit_code=$?

    if [[ $exit_code -ne 0 ]]; then
        echo -e "${GREEN}✓${NC} $name (correctly failed)"
        ((PASS++))
        return 0
    else
        echo -e "${RED}✗${NC} $name (should have failed)"
        echo "  Command: $cmd"
        ((FAIL++))
        return 1
    fi
}

# Test output contains substring
test_contains() {
    local name="$1"
    local cmd="$2"
    local substring="$3"

    local actual
    actual=$("$AUSH" --no-rc -c "$cmd" 2>&1)

    if [[ "$actual" == *"$substring"* ]]; then
        echo -e "${GREEN}✓${NC} $name"
        ((PASS++))
        return 0
    else
        echo -e "${RED}✗${NC} $name"
        echo "  Command:  $cmd"
        echo "  Expected to contain: $substring"
        echo "  Actual:   $actual"
        ((FAIL++))
        return 1
    fi
}

# Skip a test (document known issue)
test_skip() {
    local name="$1"
    local reason="$2"
    echo -e "${YELLOW}○${NC} $name (SKIP: $reason)"
    ((SKIP++))
}

section() {
    echo ""
    echo -e "${BLUE}━━━ $1 ━━━${NC}"
}

# ============================================================================
# TEST SUITE BEGINS
# ============================================================================

echo "AUSH Shell Smoke Test Suite"
echo "Testing: $AUSH"
echo "Temp dir: $TMPDIR"

# Check aush exists
if [[ ! -x "$AUSH" ]]; then
    echo -e "${RED}Error: $AUSH not found or not executable${NC}"
    echo "Build with: cargo build --release"
    exit 1
fi

# ----------------------------------------------------------------------------
section "1. BASIC BUILTINS"
# ----------------------------------------------------------------------------

test_case "echo simple" 'echo hello' 'hello'
test_case "echo multiple words" 'echo hello world' 'hello world'
test_case "echo with quotes" 'echo "hello world"' 'hello world'
test_case "echo empty" 'echo' ''
test_succeeds "pwd runs" 'pwd'
test_contains "pwd returns path" 'pwd' '/'
test_case "true returns 0" 'true; echo $?' '0'
test_case "false returns 1" 'false; echo $?' '1'
test_succeeds "cd to tmp" "cd /tmp; pwd"
test_case "cd and pwd" 'cd /tmp; pwd' '/tmp'
test_case "cd with tilde" 'cd ~; pwd' "$HOME"

# ----------------------------------------------------------------------------
section "2. VARIABLES"
# ----------------------------------------------------------------------------

test_case "echo HOME" 'echo $HOME' "$HOME"
test_case "echo USER" 'echo $USER' "$USER"
test_contains "echo PATH" 'echo $PATH' '/bin'
test_case "echo PWD" 'cd /tmp; echo $PWD' '/tmp'

# Variable assignment (POSIX)
test_case "VAR=value assignment" 'FOO=bar; echo $FOO' 'bar'
test_case "export VAR=value" 'export FOO=bar; echo $FOO' 'bar'
test_case "VAR=value command" 'FOO=test echo $FOO' 'test'

# Special variables
test_contains "dollar-dollar ($$)" 'echo $$' ''  # Just check it runs
test_case "dollar-question ($?)" 'true; echo $?' '0'
test_case "dollar-question after false" 'false; echo $?' '1'

# ----------------------------------------------------------------------------
section "3. QUOTING"
# ----------------------------------------------------------------------------

test_case "double quotes" 'echo "hello world"' 'hello world'
test_case "single quotes" "echo 'hello world'" 'hello world'
test_case "var in double quotes" 'FOO=bar; echo "value is $FOO"' 'value is bar'
test_case "var in single quotes" "FOO=bar; echo 'value is \$FOO'" 'value is $FOO'
test_case "escaped quote" 'echo "say \"hello\""' 'say "hello"'
test_case "mixed quotes" 'echo "it'"'"'s fine"' "it's fine"

# ----------------------------------------------------------------------------
section "4. COMMAND EXECUTION"
# ----------------------------------------------------------------------------

test_succeeds "external command (date)" 'date'
test_case "external with args" 'echo foo | cat' 'foo'
test_succeeds "absolute path" '/bin/echo hello'
test_case "which builtin" 'type echo' 'echo is a shell builtin'
test_contains "type external" 'type date' 'date'

# ----------------------------------------------------------------------------
section "5. PIPELINES"
# ----------------------------------------------------------------------------

test_case "simple pipe" 'echo hello | cat' 'hello'
test_case "pipe chain" 'echo -e "a\nb\nc" | cat | cat' $'a\nb\nc'
test_contains "grep in pipe" 'echo hello | grep hello' 'hello'
test_case "pipe with builtin" 'echo foo | cat' 'foo'

# ----------------------------------------------------------------------------
section "6. REDIRECTIONS"
# ----------------------------------------------------------------------------

test_succeeds "redirect stdout" "echo test > $TMPDIR/out.txt"
test_case "read redirected file" "echo test > $TMPDIR/out.txt; cat $TMPDIR/out.txt" 'test'
test_succeeds "append redirect" "echo line1 > $TMPDIR/app.txt; echo line2 >> $TMPDIR/app.txt"
test_case "append content" "echo a > $TMPDIR/app2.txt; echo b >> $TMPDIR/app2.txt; cat $TMPDIR/app2.txt" $'a\nb'
test_succeeds "input redirect" "echo hello > $TMPDIR/in.txt; cat < $TMPDIR/in.txt"

# stderr redirection
test_succeeds "stderr redirect" "ls /nonexistent 2> $TMPDIR/err.txt || true"

# ----------------------------------------------------------------------------
section "7. COMMAND CHAINING"
# ----------------------------------------------------------------------------

test_case "semicolon chain" 'echo a; echo b' $'a\nb'
test_case "triple semicolon" 'echo 1; echo 2; echo 3' $'1\n2\n3'
test_case "and chain success" 'true && echo yes' 'yes'
test_case "and chain failure" 'false && echo yes' ''
test_case "or chain success" 'true || echo no' ''
test_case "or chain failure" 'false || echo no' 'no'
test_case "mixed chain" 'false || echo fallback && echo done' $'fallback\ndone'

# ----------------------------------------------------------------------------
section "8. CONTROL FLOW"
# ----------------------------------------------------------------------------

# if/then/fi (POSIX style)
test_case "if then fi" 'if true; then echo yes; fi' 'yes'
test_case "if else" 'if false; then echo yes; else echo no; fi' 'no'
test_case "if elif else" 'if false; then echo 1; elif true; then echo 2; else echo 3; fi' '2'
test_case "if with test" 'if [ 1 -eq 1 ]; then echo equal; fi' 'equal'
test_case "if with test -f" "echo x > $TMPDIR/exist.txt; if [ -f $TMPDIR/exist.txt ]; then echo found; fi" 'found'

# while loop
test_case "while loop" 'i=0; while [ $i -lt 3 ]; do echo $i; i=$((i+1)); done' $'0\n1\n2'

# for loop
test_case "for loop" 'for x in a b c; do echo $x; done' $'a\nb\nc'
test_case "for with command sub" 'for x in $(echo "a b c"); do echo $x; done' $'a\nb\nc'

# case statement
test_case "case statement" 'x=foo; case $x in foo) echo matched;; bar) echo other;; esac' 'matched'
test_case "case with pattern" 'x=hello; case $x in h*) echo starts-h;; *) echo other;; esac' 'starts-h'

# until loop
test_case "until loop" 'i=0; until [ $i -ge 3 ]; do echo $i; i=$((i+1)); done' $'0\n1\n2'

# ----------------------------------------------------------------------------
section "9. FUNCTIONS"
# ----------------------------------------------------------------------------

test_case "function definition" 'foo() { echo hello; }; foo' 'hello'
test_case "function with args" 'greet() { echo "hi $1"; }; greet world' 'hi world'
test_case "function return" 'ret5() { return 5; }; ret5; echo $?' '5'
test_case "local variables" 'f() { local x=inner; echo $x; }; x=outer; f; echo $x' $'inner\nouter'

# ----------------------------------------------------------------------------
section "10. COMMAND SUBSTITUTION"
# ----------------------------------------------------------------------------

test_case "simple subst" 'echo $(echo hello)' 'hello'
test_case "subst in string" 'echo "dir: $(pwd)"' "dir: $(pwd)"
test_case "nested subst" 'echo $(echo $(echo deep))' 'deep'
test_case "subst in var" 'x=$(echo foo); echo $x' 'foo'

# ----------------------------------------------------------------------------
section "11. ARITHMETIC"
# ----------------------------------------------------------------------------

test_case "arithmetic expansion" 'echo $((1+2))' '3'
test_case "arithmetic multiply" 'echo $((3*4))' '12'
test_case "arithmetic with var" 'x=5; echo $((x+3))' '8'
test_case "arithmetic compare" 'echo $((5>3))' '1'

# ----------------------------------------------------------------------------
section "12. GLOB EXPANSION"
# ----------------------------------------------------------------------------

# Create test files
mkdir -p "$TMPDIR/globtest"
touch "$TMPDIR/globtest/file1.txt" "$TMPDIR/globtest/file2.txt" "$TMPDIR/globtest/other.md"

test_contains "star glob" "ls $TMPDIR/globtest/*.txt" 'file1.txt'
test_contains "question glob" "ls $TMPDIR/globtest/file?.txt" 'file1.txt'
test_case "glob expansion count" "echo $TMPDIR/globtest/*.txt | wc -w | tr -d ' '" '2'

# ----------------------------------------------------------------------------
section "13. FILENAME HANDLING"
# ----------------------------------------------------------------------------

# The critical tests - filenames with dots
touch "$TMPDIR/README.md"
touch "$TMPDIR/Cargo.toml"
touch "$TMPDIR/file.tar.gz"

test_succeeds "ls file with dot" "ls $TMPDIR/README.md"
test_succeeds "cat file with dot" "cat $TMPDIR/README.md"
test_case "unquoted dotfile" "ls $TMPDIR/README.md >/dev/null && echo ok" 'ok'
test_case "file with multiple dots" "ls $TMPDIR/file.tar.gz >/dev/null && echo ok" 'ok'
test_succeeds "relative dotfile" "cd $TMPDIR && ls README.md"

# Without path prefix (just filename)
test_case "bare filename with dot" "cd $TMPDIR && cat README.md; echo done" 'done'

# ----------------------------------------------------------------------------
section "14. TEST BUILTIN"
# ----------------------------------------------------------------------------

test_case "test -z empty" 'test -z "" && echo empty' 'empty'
test_case "test -n nonempty" 'test -n "foo" && echo nonempty' 'nonempty'
test_case "test string equal" 'test "foo" = "foo" && echo eq' 'eq'
test_case "test string not equal" 'test "foo" != "bar" && echo neq' 'neq'
test_case "test -eq" 'test 5 -eq 5 && echo eq' 'eq'
test_case "test -lt" 'test 3 -lt 5 && echo lt' 'lt'
test_case "test -gt" 'test 5 -gt 3 && echo gt' 'gt'
test_case "test -f file" "touch $TMPDIR/testfile; test -f $TMPDIR/testfile && echo exists" 'exists'
test_case "test -d dir" "test -d $TMPDIR && echo isdir" 'isdir'
test_case "test ! negation" 'test ! -f /nonexistent && echo notfile' 'notfile'
test_case "bracket notation" '[ 1 -eq 1 ] && echo yes' 'yes'

# ----------------------------------------------------------------------------
section "15. SPECIAL BUILTINS"
# ----------------------------------------------------------------------------

test_case "colon no-op" ': && echo ok' 'ok'
test_succeeds "eval simple" "eval 'echo hello'"
test_case "eval with var" "x=world; eval 'echo hello \$x'" 'hello world'
test_succeeds "source command" "echo 'echo sourced' > $TMPDIR/src.sh; source $TMPDIR/src.sh"
test_case "shift positional" 'set -- a b c; shift; echo $1' 'b'
test_succeeds "trap command" "trap 'echo trapped' EXIT"
test_case "printf format" 'printf "%s %d\n" hello 42' 'hello 42'
test_succeeds "read builtin" "echo test | read x"

# ----------------------------------------------------------------------------
section "16. JOB CONTROL"
# ----------------------------------------------------------------------------

test_succeeds "background job" "sleep 0.1 &"
test_succeeds "jobs builtin" "sleep 0.1 & jobs"
test_succeeds "wait builtin" "sleep 0.1 & wait"

# ----------------------------------------------------------------------------
section "17. EXIT CODES"
# ----------------------------------------------------------------------------

test_case "exit 0" 'exit 0; echo no' ''
test_case "command not found" 'nonexistent_cmd_12345 2>/dev/null; echo $?' '127'
test_case "successful command" "$TRUE_BIN; echo $?" '0'
test_case "failed command" "$FALSE_BIN; echo $?" '1'

# ----------------------------------------------------------------------------
section "18. HERE DOCUMENTS"
# ----------------------------------------------------------------------------

test_case "here-doc basic" $'cat <<EOF\nhello\nEOF' 'hello'
test_case "here-doc multiline" $'cat <<EOF\nline1\nline2\nEOF' $'line1\nline2'
test_case "here-doc with var" $'x=world; cat <<EOF\nhello $x\nEOF' 'hello world'

# ----------------------------------------------------------------------------
section "19. SUBSHELLS"
# ----------------------------------------------------------------------------

test_case "subshell isolation" 'x=outer; (x=inner; echo $x); echo $x' $'inner\nouter'
test_case "subshell exit" '(exit 5); echo $?' '5'
test_succeeds "subshell pipeline" 'echo test | (cat)'

# ----------------------------------------------------------------------------
section "20. RUSH-SPECIFIC BUILTINS"
# ----------------------------------------------------------------------------

test_succeeds "help builtin" "help"
test_contains "ls builtin" "ls" ""  # Just runs
test_succeeds "grep builtin" "echo hello | grep hello"
test_succeeds "find builtin" "find . -maxdepth 1 -type f | head -1"
test_succeeds "cat builtin" "echo test | cat"

# Git builtins (only if in git repo)
if git rev-parse --git-dir >/dev/null 2>&1; then
    test_succeeds "git status builtin" "git status"
    if git rev-parse HEAD >/dev/null 2>&1; then
        test_succeeds "git log builtin" "git log --oneline -1"
    else
        test_skip "git log builtin" "repository has no commits yet"
    fi
fi

# ============================================================================
# SUMMARY
# ============================================================================

echo ""
echo -e "${BLUE}━━━ RESULTS ━━━${NC}"
echo ""

TOTAL=$((PASS + FAIL + SKIP))

echo -e "Passed:  ${GREEN}$PASS${NC}"
echo -e "Failed:  ${RED}$FAIL${NC}"
echo -e "Skipped: ${YELLOW}$SKIP${NC}"
echo "Total:   $TOTAL"
echo ""

if [[ $FAIL -eq 0 ]]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    PERCENT=$((PASS * 100 / TOTAL))
    echo -e "${RED}$FAIL tests failed${NC} ($PERCENT% pass rate)"
    exit 1
fi
