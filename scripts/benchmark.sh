#!/usr/bin/env bash

# benchmark.sh - Real-world performance comparison using hyperfine
# This script compares AUSH with other shells (bash, zsh) and system utilities

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if hyperfine is installed
if ! command -v hyperfine &> /dev/null; then
    echo -e "${RED}Error: hyperfine is not installed${NC}"
    echo "Install with: cargo install hyperfine"
    echo "Or on macOS: brew install hyperfine"
    exit 1
fi

# Build AUSH in release mode
echo -e "${BLUE}Building AUSH in release mode...${NC}"
cargo build --release

AUSH_BIN="${AUSH_BIN:-./target/release/aush}"
if [ ! -f "$AUSH_BIN" ] && [ -f ./target/release/rush ]; then
    AUSH_BIN=./target/release/rush
fi

if [ ! -f "$AUSH_BIN" ]; then
    echo -e "${RED}Error: AUSH binary not found at $AUSH_BIN${NC}"
    exit 1
fi

echo -e "${GREEN}AUSH binary ready!${NC}\n"

# Create a temporary directory for test files
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Create test files
echo "Creating test files in $TEMP_DIR..."
for i in {1..100}; do
    echo "Line $i: Some test content here" > "$TEMP_DIR/file_$i.txt"
done

echo -e "\n${YELLOW}=== Starting Benchmarks ===${NC}\n"

# 1. Shell Startup Time
echo -e "${BLUE}[1/7] Benchmarking Shell Startup Time${NC}"
echo "Target: < 10ms"
hyperfine \
    --warmup 5 \
    --min-runs 50 \
    --export-markdown results/startup_comparison.md \
    "$AUSH_BIN -c exit" \
    "bash -c exit" \
    "zsh -c exit" \
    2>/dev/null || true

echo ""

# 2. Simple Echo Command
echo -e "${BLUE}[2/7] Benchmarking Echo Command${NC}"
hyperfine \
    --warmup 3 \
    --min-runs 30 \
    "$AUSH_BIN -c 'echo hello world'" \
    "bash -c 'echo hello world'" \
    "echo hello world"

echo ""

# 3. PWD Command
echo -e "${BLUE}[3/7] Benchmarking PWD Command${NC}"
hyperfine \
    --warmup 3 \
    --min-runs 30 \
    "$AUSH_BIN -c 'pwd'" \
    "bash -c 'pwd'" \
    "pwd"

echo ""

# 4. CD Command
echo -e "${BLUE}[4/7] Benchmarking CD Command${NC}"
hyperfine \
    --warmup 3 \
    --min-runs 30 \
    "$AUSH_BIN -c 'cd /tmp && pwd'" \
    "bash -c 'cd /tmp && pwd'"

echo ""

# 5. Multiple Commands
echo -e "${BLUE}[5/7] Benchmarking Multiple Sequential Commands${NC}"
hyperfine \
    --warmup 3 \
    --min-runs 20 \
    "$AUSH_BIN -c 'pwd && echo test && pwd'" \
    "bash -c 'pwd && echo test && pwd'"

echo ""

# 6. Variable Export
echo -e "${BLUE}[6/7] Benchmarking Environment Variable Export${NC}"
hyperfine \
    --warmup 3 \
    --min-runs 30 \
    "$AUSH_BIN -c 'export TEST=value'" \
    "bash -c 'export TEST=value'"

echo ""

# 7. Complex Pipeline (if supported)
echo -e "${BLUE}[7/7] Benchmarking Complex Operations${NC}"
cd "$TEMP_DIR"
hyperfine \
    --warmup 2 \
    --min-runs 10 \
    "bash -c 'for i in {1..10}; do echo \$i; done'" \
    "for i in {1..10}; do echo \$i; done"
cd - > /dev/null

echo ""

# Memory Usage Comparison
echo -e "${YELLOW}=== Memory Usage Comparison ===${NC}"
echo -e "${BLUE}Measuring peak memory usage...${NC}"

echo -n "AUSH: "
/usr/bin/time -l $AUSH_BIN -c "pwd && echo test" 2>&1 | grep "maximum resident set size" | awk '{print $1 / 1024 / 1024 " MB"}' || echo "N/A"

echo -n "Bash: "
/usr/bin/time -l bash -c "pwd && echo test" 2>&1 | grep "maximum resident set size" | awk '{print $1 / 1024 / 1024 " MB"}' || echo "N/A"

echo ""

# Generate summary
echo -e "${YELLOW}=== Benchmark Summary ===${NC}"
echo -e "${GREEN}Benchmarks completed!${NC}"
echo ""
echo "Performance Targets:"
echo "  - Startup time: < 10ms"
echo "  - Memory usage: < 10MB"
echo "  - Builtins: Competitive with GNU utils"
echo ""
echo "Results have been saved to results/ directory (if --export-markdown was used)"
echo ""
echo -e "${BLUE}To run criterion benchmarks:${NC}"
echo "  cargo bench --bench startup"
echo "  cargo bench --bench builtins"
echo ""
echo -e "${BLUE}To profile in detail:${NC}"
echo "  cargo flamegraph --bench startup"
echo "  cargo instruments -t time --bench builtins"
