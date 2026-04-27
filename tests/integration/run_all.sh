#!/bin/bash
# Master test runner for all AUSH integration tests

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOTAL_PASSED=0
TOTAL_FAILED=0
FAILED_SUITES=""

echo "========================================"
echo "AUSH Integration Test Suite"
echo "========================================"
echo ""

run_test_suite() {
    local test_script="$1"
    local test_name=$(basename "$test_script" .sh)

    echo -e "${BLUE}Running: $test_name${NC}"
    echo "----------------------------------------"

    if timeout 60 "$test_script"; then
        echo -e "${GREEN}✓ $test_name PASSED${NC}"
        echo ""
        return 0
    else
        echo -e "${RED}✗ $test_name FAILED${NC}"
        FAILED_SUITES="${FAILED_SUITES}\n  - $test_name"
        echo ""
        return 1
    fi
}

# Run all test suites
for test_script in "$SCRIPT_DIR"/*_test.sh; do
    if [ -f "$test_script" ] && [ -x "$test_script" ]; then
        if run_test_suite "$test_script"; then
            ((TOTAL_PASSED++))
        else
            ((TOTAL_FAILED++))
        fi
    fi
done

# Summary
echo "========================================"
echo "Test Suite Summary"
echo "========================================"
echo -e "Suites Passed: ${GREEN}$TOTAL_PASSED${NC}"
echo -e "Suites Failed: ${RED}$TOTAL_FAILED${NC}"
echo "Total Suites:  $((TOTAL_PASSED + TOTAL_FAILED))"

if [ $TOTAL_FAILED -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed test suites:${NC}"
    echo -e "$FAILED_SUITES"
    echo ""
    exit 1
else
    echo ""
    echo -e "${GREEN}All integration test suites passed!${NC}"
    echo ""
    exit 0
fi
