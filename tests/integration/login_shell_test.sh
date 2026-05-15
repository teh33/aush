#!/bin/bash
set -euo pipefail

TESTS_PASSED=0
TESTS_FAILED=0
FAILURES=""

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
AUSH_BINARY="$PROJECT_ROOT/target/debug/aush"
if [ ! -x "$AUSH_BINARY" ]; then
    echo -e "${RED}Error: aush binary not found at $AUSH_BINARY${NC}"
    echo "Build it first with: cargo test --test login_shell_tests"
    exit 1
fi

echo "======================================"
echo "AUSH login shell integration tests"
echo "======================================"
echo "AUSH binary: $AUSH_BINARY"
echo ""

test_case() {
    local test_name="$1"
    echo -n "Testing: $test_name ... "
}

pass() {
    echo -e "${GREEN}PASS${NC}"
    ((TESTS_PASSED+=1))
}

fail() {
    local test_name="$1"
    local detail="$2"
    echo -e "${RED}FAIL${NC} ${detail}"
    ((TESTS_FAILED+=1))
    FAILURES="${FAILURES}\n- ${test_name}: ${detail}"
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local test_name="$3"
    if grep -Fq "$needle" <<<"$haystack"; then
        pass
    else
        fail "$test_name" "expected output to contain '$needle'"
    fi
}

assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local test_name="$3"
    if grep -Fq "$needle" <<<"$haystack"; then
        fail "$test_name" "output should not contain '$needle'"
    else
        pass
    fi
}

run_case() {
    local home_dir="$1"
    shift
    HOME="$home_dir" SHELL= "$AUSH_BINARY" "$@"
}

TEST_HOME="$(mktemp -d)"
trap 'rm -rf "$TEST_HOME"' EXIT

cat >"$TEST_HOME/.aush_profile" <<'EOF'
echo profile_loaded
EOF

cat >"$TEST_HOME/.aushrc" <<'EOF'
echo rc_loaded
bad_startup_command
echo rc_after_error
EOF

# 1. -c should stay deterministic and skip startup files even with --login.
test_case "-c with --login skips startup files"
OUTPUT="$(run_case "$TEST_HOME" --login -c 'echo command_only' 2>&1)"
assert_contains "$OUTPUT" "command_only" "-c with --login skips startup files"
assert_not_contains "$OUTPUT" "profile_loaded" "-c with --login skips startup files"
assert_not_contains "$OUTPUT" "rc_loaded" "-c with --login skips startup files"

# 2. -c with --no-rc should also skip startup files.
test_case "-c with --no-rc skips startup files"
OUTPUT="$(run_case "$TEST_HOME" --no-rc -c 'echo no_rc_command' 2>&1)"
assert_contains "$OUTPUT" "no_rc_command" "-c with --no-rc skips startup files"
assert_not_contains "$OUTPUT" "profile_loaded" "-c with --no-rc skips startup files"
assert_not_contains "$OUTPUT" "rc_loaded" "-c with --no-rc skips startup files"

# 3. Document current direct stdin semantics: non-interactive input is deterministic and does not source rc files.
test_case "stdin mode skips startup files"
OUTPUT="$(printf 'echo stdin_command\n' | HOME="$TEST_HOME" SHELL= "$AUSH_BINARY" 2>&1)"
assert_contains "$OUTPUT" "stdin_command" "stdin mode skips startup files"
assert_not_contains "$OUTPUT" "profile_loaded" "stdin mode skips startup files"
assert_not_contains "$OUTPUT" "rc_loaded" "stdin mode skips startup files"

# 4. Source errors should be actionable and execution should continue.
test_case "source errors name the failing command and continue"
RC_FILE="$TEST_HOME/manual_rc.aush"
cat >"$RC_FILE" <<'EOF'
echo before_manual_error
manual_bad_command
echo after_manual_error
EOF
OUTPUT="$(run_case "$TEST_HOME" -c "source $RC_FILE" 2>&1)"
assert_contains "$OUTPUT" "before_manual_error" "source errors name the failing command and continue"
assert_contains "$OUTPUT" "after_manual_error" "source errors name the failing command and continue"
assert_contains "$OUTPUT" "manual_bad_command" "source errors name the failing command and continue"

echo ""
echo "======================================"
echo "Summary"
echo "======================================"
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"

if [ "$TESTS_FAILED" -ne 0 ]; then
    echo -e "${RED}Failures:${NC}${FAILURES}"
    exit 1
fi

echo -e "${GREEN}All login shell integration tests passed.${NC}"
