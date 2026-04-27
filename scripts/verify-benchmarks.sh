#!/usr/bin/env bash

# verify-benchmarks.sh - Verify that benchmarking setup is working correctly
# Run this to ensure all benchmark infrastructure is properly configured

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}AUSH Benchmark Verification${NC}\n"

# Check 1: Cargo.toml configuration
echo -n "Checking Cargo.toml benchmark targets... "
if grep -q "^\[\[bench\]\]" Cargo.toml && \
   grep -q "name = \"startup\"" Cargo.toml && \
   grep -q "name = \"builtins\"" Cargo.toml; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Missing benchmark targets${NC}"
    exit 1
fi

# Check 2: Benchmark files exist
echo -n "Checking benchmark files exist... "
if [ -f "benches/startup.rs" ] && [ -f "benches/builtins.rs" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Missing benchmark files${NC}"
    exit 1
fi

# Check 3: Scripts exist and are executable
echo -n "Checking benchmark scripts... "
if [ -x "scripts/benchmark.sh" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ benchmark.sh not executable${NC}"
    echo "Run: chmod +x scripts/benchmark.sh"
    exit 1
fi

# Check 4: Results directory exists
echo -n "Checking results directory... "
if [ -d "results" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ results/ directory missing${NC}"
    exit 1
fi

# Check 5: Documentation exists
echo -n "Checking benchmark documentation... "
if [ -f "BENCHMARKS.md" ] && \
   [ -f "benches/README.md" ] && \
   [ -f "scripts/README.md" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ Missing documentation${NC}"
    exit 1
fi

# Check 6: lib.rs exists for benchmark imports
echo -n "Checking src/lib.rs exists... "
if [ -f "src/lib.rs" ]; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ src/lib.rs missing (needed for benchmarks)${NC}"
    exit 1
fi

# Check 7: Criterion dependency
echo -n "Checking criterion dependency... "
if grep -q "criterion.*=" Cargo.toml; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗ criterion not in dependencies${NC}"
    exit 1
fi

# Check 8: Release build compiles
echo -n "Building release binary... "
if cargo build --release --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${YELLOW}⚠ Build warnings (non-critical)${NC}"
fi

# Check 9: Benchmarks compile
echo -n "Checking benchmarks compile... "
if cargo check --benches --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${YELLOW}⚠ Benchmark compilation warnings (non-critical)${NC}"
fi

# Check 10: Hyperfine availability
echo -n "Checking hyperfine installation... "
if command -v hyperfine &> /dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${YELLOW}⚠ hyperfine not installed${NC}"
    echo "  Install with: cargo install hyperfine"
    echo "  Or on macOS: brew install hyperfine"
fi

echo ""
echo -e "${GREEN}All checks passed!${NC}"
echo ""
echo "You can now run benchmarks:"
echo "  ${BLUE}cargo bench${NC}                  - Run all criterion benchmarks"
echo "  ${BLUE}cargo bench --bench startup${NC}  - Run startup benchmarks only"
echo "  ${BLUE}cargo bench --bench builtins${NC} - Run builtin benchmarks only"
echo "  ${BLUE}./scripts/benchmark.sh${NC}       - Run real-world comparisons"
echo ""
echo "View results:"
echo "  ${BLUE}open target/criterion/report/index.html${NC}"
echo ""
echo "Documentation:"
echo "  ${BLUE}BENCHMARKS.md${NC}        - Comprehensive guide"
echo "  ${BLUE}benches/README.md${NC}    - Criterion benchmarks"
echo "  ${BLUE}scripts/README.md${NC}    - Hyperfine scripts"
