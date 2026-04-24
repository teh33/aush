# AUSH - Quick Start Guide

Get up and running with AUSH (Actually Usable Shell) in under 5 minutes!

## Building AUSH

### 1. Build in Release Mode
```bash
cd /Users/asher/knowledge/rush
cargo build --release
```

This will create the optimized binary at `target/release/rush`.

### 2. Run AUSH
```bash
./target/release/rush
```

You should see the AUSH prompt:
```
aush>
```

## Try It Out!

### Basic Commands
```bash
# Navigate directories
pwd
ls
ls -la
cd /tmp
cd ~

# View files
cat Cargo.toml
cat src/main.rs

# Search
grep "fn main" src/**/*.rs
find . -name "*.rs"

# Variables
export MY_VAR="hello"
echo $MY_VAR
```

### Fast Builtins (AUSH's Superpower!)

AUSH reimplements common commands with high performance:

```bash
# Memory-mapped cat (blazing fast for large files)
cat target/debug/rush  # Large binary, still instant!

# Parallel ls (fast even with many files)
ls -la target/release

# WalkBuilder find (parallel directory traversal)
find src -name "*.rs"

# Ripgrep integration (super fast search)
grep "TODO" src/**/*.rs
```

### Command History

AUSH has a powerful history system:

```bash
# Add some commands to history
echo "test 1"
ls -la
cargo build
git status

# View recent history
history

# Show last 5 commands
history 5

# Fuzzy search for commands containing "cargo"
history search cargo

# Clear history
history clear
```

### User-Defined Functions

AUSH supports functions with parameters:

```bash
# Define a greeting function
function greet(name) {
    echo "Hello, $name!"
    echo "Welcome to AUSH"
}

# Call it
greet "World"
# Output:
# Hello, World!
# Welcome to AUSH

# Functions with multiple statements
function deploy(env) {
    echo "Deploying to $env..."
    ls -la
    echo "Deployment complete!"
}

deploy "production"
```

### Tab Completion

AUSH has smart, context-aware tab completion:

```bash
# Command completion
ca<TAB>          # Completes to: cat, cargo, etc.

# Path completion
cat src/ma<TAB>  # Completes to: src/main.rs

# Git-aware completion (in a git repo)
git sta<TAB>     # Completes to: git status
git checkout <TAB>  # Shows branches

# Cargo-aware completion (in Rust project)
cargo bu<TAB>    # Completes to: cargo build
cargo test <TAB>  # Shows test names

# Flag completion
ls -<TAB>        # Shows: -l, -a, -la, -h, etc.
```

### Undo File Operations

AUSH can undo file operations:

```bash
# Track operations (enabled by default)
undo list

# If you make a mistake:
rm important-file.txt
undo  # Restores it!

# View what can be undone
undo list

# Disable tracking temporarily
undo disable

# Re-enable
undo enable

# Clear undo history
undo clear
```

## Test AUSH's Performance

### Quick Performance Test

Run this in both zsh and AUSH to feel the difference:

```bash
# In zsh (traditional shell)
time for i in {1..100}; do echo "test" > /dev/null; done

# In AUSH (optimized builtins)
time for i in {1..100}; do echo "test" > /dev/null; done
```

### Comprehensive Benchmark

```bash
# Install timing tools
brew install coreutils

# Run the benchmark suite
cd benchmarks
./compare.sh
```

This will:
1. Generate test data (large files, deep directories)
2. Run 10 different performance tests
3. Compare AUSH vs zsh
4. Show speedup metrics

Expected results on M2 Macbook Air:
- **cat**: 2-5x faster (memory-mapped I/O)
- **ls**: 1.5-3x faster (parallel reads)
- **find**: 2-4x faster (WalkBuilder)
- **grep**: 3-10x faster (ripgrep)

## Key Features

### 1. Fast Builtins
- `ls` - Optimized directory listing
- `cat` - Memory-mapped file reading
- `find` - Parallel directory traversal
- `grep` - Ripgrep integration
- `cd`, `pwd`, `echo`, `export` - Zero overhead

### 2. Command History
- Persistent storage at `~/.aush_history`, with legacy `~/.rush_history` migration support planned
- Fuzzy search with ranking
- Deduplication (no repeated commands)
- Timestamp tracking
- Privacy mode (space-prefix to ignore)

### 3. Tab Completion
- Context-aware (git, cargo, npm)
- Path completion (respects .gitignore)
- Command and flag completion
- Smart caching (5-min TTL)

### 4. User Functions
- Define reusable command sequences
- Parameter passing
- Local variable scoping
- Recursion support (100 depth limit)

### 5. Undo Capability
- Track file create/delete/modify/move
- Automatic backups
- Undo stack (last 100 operations)
- Safe file operations

### 6. Project Context
- Auto-detects project type (Rust, Node, Python, etc.)
- Smart command routing (test → cargo test)
- Git integration
- Fast caching

## Tips & Tricks

### 1. Use Built-in Commands
```bash
# Prefer AUSH builtins (always faster)
ls           # ✓ Fast builtin
cat file     # ✓ Fast builtin
find . -name # ✓ Fast builtin

# Avoid external commands when builtin exists
/bin/ls      # ✗ Slower (spawns process)
```

### 2. Leverage History Search
```bash
# Instead of retyping:
history search "long complicated command"
# Then copy/paste the result
```

### 3. Use Tab Completion
```bash
# Save typing:
cargo b<TAB>    # → cargo build
git st<TAB>     # → git status
cat src/m<TAB>  # → cat src/main.rs
```

### 4. Create Functions for Common Tasks
```bash
function gs() {
    git status
}

function build() {
    cargo build --release
}

function test() {
    cargo test
}
```

### 5. Protect Important Operations
```bash
# Before dangerous operations, undo is enabled:
undo list  # See current state
rm file
undo       # Oops! Restore it
```

## Configuration

### History Settings

Edit `~/.rush_history` file or configure in code:
- Max size: 10,000 entries (default)
- Deduplication: Consecutive only (default) or all
- Timestamps: Off (default) or on
- Ignore patterns: Add sensitive commands

### Undo Settings

Configure in `~/.rush_undo`:
- Max operations: 100 (default)
- Backup location: `~/.rush_undo/`
- Enable/disable: `undo enable` / `undo disable`

## Known Limitations

Current phase (Phase 3):

1. **No script execution** - Can't run `.sh` files yet
   - Coming in Phase 4
   - Use interactive mode for now

2. **Limited pipe support** - Basic pipes work, complex ones don't
   - `echo "test" | cat` ✓
   - `cmd1 | cmd2 | cmd3` ✓ (basic)
   - Complex pipelines may fail

3. **No job control** - Can't background processes yet
   - No `&`, `fg`, `bg`, `jobs`
   - Coming in future phase

4. **Limited env var support** - Basic export/get works
   - More advanced features coming

## What's Next?

### Phase 4 (Planned)
- Shell script execution
- Advanced piping and redirection
- Job control (background jobs)
- More builtins (sed, awk alternatives)
- Configuration file support

### Phase 5+ (Future)
- Plugin system
- Remote execution
- AI-powered suggestions
- Cross-session history sync
- Custom themes and prompts

## Troubleshooting

### AUSH won't build
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Commands not found
```bash
# Make sure you're using builtins:
ls        # ✓ Builtin
/bin/ls   # ✗ External

# Check if command exists:
which ls  # Should show "builtin"
```

### History not saving
```bash
# Check permissions:
ls -la ~/.rush_history

# Manually load:
# (in code, happens automatically)
```

### Undo not working
```bash
# Check if enabled:
undo list

# Enable if disabled:
undo enable

# Check backup directory:
ls -la ~/.rush_undo
```

## Getting Help

### Documentation
- `/docs/` - Comprehensive feature documentation
- `/benchmarks/README.md` - Performance benchmarking guide
- `/docs/phase-3-completion-summary.md` - Complete feature list

### Command Help
```bash
# Most commands support --help:
history --help
undo --help
```

### Testing
```bash
# Run all tests:
cargo test

# Run specific module:
cargo test history
cargo test completion
```

## Enjoy AUSH!

AUSH is designed to be fast, safe, and powerful. Key advantages:

✅ **2-10x faster** than traditional shells for file operations
✅ **Safe by default** with undo capability
✅ **Smart completion** that understands your context
✅ **Rich history** with fuzzy search
✅ **Modern Rust** - safe, fast, concurrent

Try it for your daily work and feel the difference!

---

**Feedback?** Create an issue in the repo or contribute improvements!
