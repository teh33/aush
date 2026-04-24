# Rush Shell - Build Targets
#
# Usage:
#   make build       - Standard release build
#   make pgo         - Profile-guided optimization build (10-20% faster)
#   make bench-start - Benchmark startup time (requires hyperfine)
#   make clean       - Clean build artifacts and PGO data
#   make install     - Install rush to ~/.cargo/bin

CARGO := cargo
PGO_DIR := /tmp/rush-pgo-data

# Find llvm-profdata from the Rust toolchain
LLVM_PROFDATA := $(shell find $$(rustc --print sysroot) -name llvm-profdata 2>/dev/null | head -1)

.PHONY: build pgo pgo-check pgo-instrument pgo-collect pgo-merge pgo-build bench-start bench-usable bench-aush bench-aush-fast clean install

# --- Standard Targets ---

build:
	$(CARGO) build --release

install: build
	cp target/release/rush ~/.cargo/bin/rush

clean:
	$(CARGO) clean
	rm -rf $(PGO_DIR)

# --- Benchmarking ---

bench-start:
	@command -v hyperfine >/dev/null 2>&1 || { echo "Error: hyperfine not found. Install with: cargo install hyperfine"; exit 1; }
	hyperfine --warmup 5 --runs 30 './target/release/rush -c exit'

bench-usable: build
	@echo "=== Actually Usable session benchmarks ==="
	bash ./benches/interactive_benchmark.sh
	@echo ""
	bash ./benches/session_benchmark.sh

bench-aush-fast: build
	@echo "=== AUSH fast smoke benchmark ==="
	bash ./benches/aush_smoke_fast.sh ./target/release/rush

bench-aush: build
	@echo "=== AUSH full benchmark suite ==="
	bash ./benches/aush_suite.sh ./target/release/rush

# --- PGO Build Pipeline ---
#
# Profile-Guided Optimization uses runtime profiling data to optimize
# branch prediction, code layout, and inlining decisions.
#
# Prerequisites:
#   rustup component add llvm-tools-preview
#
# The full PGO pipeline:
#   1. Build with instrumentation (generates profile hooks)
#   2. Run representative workloads (collects .profraw files)
#   3. Merge profiles into a single .profdata file
#   4. Rebuild using the merged profile data

pgo: pgo-check pgo-instrument pgo-collect pgo-merge pgo-build
	@echo ""
	@echo "============================================"
	@echo "  PGO build complete!"
	@echo "  Binary: target/release/rush"
	@echo "============================================"
	@echo ""
	@echo "Benchmark with:"
	@echo "  hyperfine --warmup 5 --runs 30 './target/release/rush -c exit'"

pgo-check:
ifeq ($(LLVM_PROFDATA),)
	@echo "Error: llvm-profdata not found in Rust toolchain."
	@echo ""
	@echo "Install it with:"
	@echo "  rustup component add llvm-tools-preview"
	@echo ""
	@echo "Then retry: make pgo"
	@exit 1
else
	@echo "Found llvm-profdata: $(LLVM_PROFDATA)"
endif

pgo-instrument:
	@echo ""
	@echo "=== Step 1/4: Instrumented build ==="
	@rm -rf $(PGO_DIR)
	@mkdir -p $(PGO_DIR)
	RUSTFLAGS="-Cprofile-generate=$(PGO_DIR)" $(CARGO) build --release

pgo-collect:
	@echo ""
	@echo "=== Step 2/4: Collecting profile data ==="
	@echo "Running representative workloads..."
	@# Basic startup/exit
	./target/release/rush -c "exit"
	./target/release/rush -c "true"
	./target/release/rush -c "false" || true
	@# Echo and output
	./target/release/rush -c "echo hello"
	./target/release/rush -c "echo hello world"
	./target/release/rush -c "echo one two three four five"
	@# Builtins
	./target/release/rush -c "pwd"
	./target/release/rush -c "echo $$HOME"
	./target/release/rush -c "type echo"
	@# Pipelines
	./target/release/rush -c "echo hello | cat"
	./target/release/rush -c "echo test | cat | cat"
	@# Variable expansion
	./target/release/rush -c 'X=hello; echo $$X'
	@# Iteration for statistical significance (startup-heavy workloads)
	@echo "Running 100 iterations of startup workloads..."
	@for i in $$(seq 1 100); do ./target/release/rush -c "echo test"; done
	@for i in $$(seq 1 100); do ./target/release/rush -c "exit"; done
	@for i in $$(seq 1 50); do ./target/release/rush -c "echo hello | cat"; done
	@echo "Profile data collected."

pgo-merge:
	@echo ""
	@echo "=== Step 3/4: Merging profile data ==="
	$(LLVM_PROFDATA) merge -o $(PGO_DIR)/merged.profdata $(PGO_DIR)/
	@echo "Merged profile: $(PGO_DIR)/merged.profdata"

pgo-build:
	@echo ""
	@echo "=== Step 4/4: PGO-optimized build ==="
	RUSTFLAGS="-Cprofile-use=$(PGO_DIR)/merged.profdata -Cllvm-args=-pgo-warn-missing-function" $(CARGO) build --release
	@echo "PGO-optimized binary built."

# --- Combined Benchmark (before + after PGO) ---

bench-pgo: build
	@echo "=== Baseline (standard release build) ==="
	@command -v hyperfine >/dev/null 2>&1 || { echo "Error: hyperfine not found. Install with: cargo install hyperfine"; exit 1; }
	@cp target/release/rush /tmp/rush-baseline
	@$(MAKE) pgo
	@echo ""
	@echo "=== Comparing baseline vs PGO ==="
	hyperfine --warmup 5 --runs 30 \
		'/tmp/rush-baseline -c exit' \
		'./target/release/rush -c exit'
	@rm -f /tmp/rush-baseline
