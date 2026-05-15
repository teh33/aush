# AUSH Release Testing Guide

This guide provides step-by-step instructions for testing AUSH installations on clean machines before release publication.

## Pre-Release Checklist

Before releasing a new version, complete these tests:

- [ ] GitHub Actions release workflow runs successfully
- [ ] All binaries compile for all target platforms
- [ ] All checksums are generated correctly
- [ ] Checksums aggregate into single file
- [ ] Automated CI tests pass on all platforms
- [ ] Manual testing on clean macOS (Intel) machine
- [ ] Manual testing on clean macOS (ARM) machine
- [ ] Manual testing on clean Linux machine
- [ ] Homebrew formula works with new release
- [ ] README installation instructions verified
- [ ] Documentation is up-to-date
- [ ] Explicit approval received before publishing, tagging, pushing, or updating external registries

## Crates.io Release Gate

Use this gate before publishing `aush` to crates.io:

```bash
cargo test --quiet --lib --bins
cargo publish --dry-run
```

Run `cargo publish --dry-run --allow-dirty` only while validating uncommitted release-documentation changes locally. The final pre-publish check must run without `--allow-dirty` from a clean worktree.

This is the minimum required local gate for a crates.io package publish. It verifies the library and binary unit tests compile and pass, then proves Cargo can package and verify the crate exactly as crates.io will receive it.

For the first `0.1.x` crates.io releases, do **not** require the entire integration suite to be green before publish unless each suite has been made hermetic. The current `cargo test --quiet --tests` set includes suites that are valuable for development but are not yet stable release gates because they depend on host shell environment, external services, shellspec fixture layout, terminal/process-group behavior, or known compatibility-roadmap work.

Before publishing, record the current status of excluded suites in mana and make sure every exclusion has a follow-up task or a clear reason. Exclusions must be explicit; do not silently ignore failing tests.

### Optional integration confidence pass

When time allows, run targeted integration suites that are stable on the current platform:

```bash
cargo test --quiet --test error_recovery_tests --test exit_code_tests
```

Add more suites to this optional pass only after they are deterministic on clean machines.

### Currently non-gating integration categories

These categories are tracked separately until they are hermetic enough to become release gates:

- host-environment-sensitive tests, such as login shell and shell variable initialization;
- external-service tests, such as AI/pipe-ask flows that can make network requests;
- shellspec/POSIX fixture tests that require a specific project layout or runner setup;
- long-running or platform-sensitive terminal, signal, and process-group tests;
- compatibility-roadmap tests for shell features that are known incomplete in `0.1.x`.

## Platform-Specific Tests

### macOS Intel (x86_64) Testing

#### 1. Binary Download and Verification

```bash
# Create test directory
mkdir -p ~/aush-test-intel
cd ~/aush-test-intel

# Download the binary
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-x86_64.tar.gz

# Download checksums
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-x86_64-SHA256SUMS.txt

# Verify checksum
shasum -a 256 -c aush-macos-x86_64-SHA256SUMS.txt
```

Expected output:
```
aush-macos-x86_64.tar.gz: OK
```

#### 2. Extract and Install

```bash
# Extract
tar xzf aush-macos-x86_64.tar.gz
chmod +x aush

# Test basic execution
./aush -c 'echo "AUSH on macOS Intel"'
```

Expected output:
```
AUSH on macOS Intel
```

#### 3. Install to System PATH

```bash
# Install to /usr/local/bin (may need sudo)
sudo cp aush /usr/local/bin/aush

# Test from PATH
aush -c 'pwd'
```

#### 4. Functionality Tests

```bash
# Variables
aush -c 'export TEST=hello && echo $TEST'
# Expected: hello

# Arithmetic
aush -c 'x=5; echo $((x + 3))'
# Expected: 8

# Conditionals
aush -c 'if [ 1 -eq 1 ]; then echo "true"; fi'
# Expected: true

# Loops
aush -c 'for i in 1 2 3; do echo $i; done'
# Expected: 1\n2\n3

# Pipes
aush -c 'echo -e "apple\nbanana" | grep apple'
# Expected: apple
```

#### 5. Daemon Mode Test

```bash
# Test the daemon archive if downloaded separately
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-x86_64-daemon.tar.gz
tar xzf aush-macos-x86_64-daemon.tar.gz
chmod +x aushd

# Start daemon
aushd start

# Test command execution with daemon
aush -c 'echo "Using daemon"'

# Stop daemon
aushd stop
```

#### 6. Setting as Default Shell (Optional)

```bash
# Add to allowed shells
echo "/usr/local/bin/aush" | sudo tee -a /etc/shells

# Change to aush (temporary test only!)
# Don't do this on production machines
```

### macOS ARM (Apple Silicon) Testing

Follow the same steps as Intel, but with:

```bash
# Step 1: Download ARM binary
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-aarch64.tar.gz
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-aarch64-SHA256SUMS.txt
```

Verify that the binary is ARM-compiled:
```bash
file aush
# Expected: Mach-O 64-bit executable arm64
```

### Linux x86_64 Testing

#### 1. Binary Download and Verification

```bash
# Create test directory
mkdir -p ~/aush-test-linux
cd ~/aush-test-linux

# Download the binary
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64.tar.gz

# Download checksums
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64-SHA256SUMS.txt

# Verify checksum
sha256sum -c aush-linux-x86_64-SHA256SUMS.txt
```

Expected output:
```
aush-linux-x86_64.tar.gz: OK
```

#### 2. Extract and Install

```bash
# Extract
tar xzf aush-linux-x86_64.tar.gz
chmod +x aush

# Test basic execution
./aush -c 'echo "AUSH on Linux"'
```

Expected output:
```
AUSH on Linux
```

#### 3. Verify Architecture

```bash
# Check binary info
file aush
# Expected: ELF 64-bit LSB pie executable, x86-64, dynamically linked

# Check dependencies
ldd aush
# Should show normal glibc dependencies (or none for musl)
```

#### 4. Install to System PATH

```bash
# Install
sudo mv aush /usr/local/bin/aush

# Verify
which aush
# Expected: /usr/local/bin/aush

aush -c 'echo "System-wide installation works"'
```

#### 5. Functionality Tests

```bash
# Variables
aush -c 'export TEST=linux && echo $TEST'
# Expected: linux

# String manipulation
aush -c 'str="hello world"; echo ${str#hello}'
# Expected: ' world'

# Command substitution
aush -c 'echo "Home: $(cd ~ && pwd)"'
# Expected: Home: /home/username

# Background jobs
aush -c 'sleep 1 & echo "Background job started"'
```

#### 6. Daemon Mode Test

```bash
# Test the daemon archive if downloaded separately
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64-daemon.tar.gz
tar xzf aush-linux-x86_64-daemon.tar.gz
chmod +x aushd

# Start daemon
aushd start

# Test multiple quick commands
time aush -c 'ls -la /' > /dev/null
# Should be very fast (~0.4ms with daemon)

# Stop
aushd stop
```

### Static Linux (musl) Testing

The musl variant should work on any Linux system without dependencies:

```bash
# Download musl variant
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64-musl.tar.gz
tar xzf aush-linux-x86_64-musl.tar.gz

# Verify it's static
ldd ./aush
# Expected: "not a dynamic executable" or "statically linked"

# Test on different systems
./aush -c 'echo "Works without libc"'
```

## Homebrew Testing

### Install via Homebrew (macOS only)

```bash
# Add tap
brew tap kfcafe/aush https://github.com/kfcafe/aush

# Install
brew install aush

# Verify installation
which aush
# Expected: /usr/local/bin/aush (or similar brew path)

# Test
aush -c 'echo "Installed via Homebrew"'
```

### Update Homebrew Formula

After each release, verify the Homebrew formula works:

```bash
# The formula automatically downloads the latest release
# To test with a specific version, temporarily edit:
# /usr/local/Cellar/aush/*/Homebrew/Formula/aush.rb

# Test uninstall/reinstall
brew uninstall aush
brew install aush
```

## Checksum Aggregation Testing

Verify the combined checksum file works:

```bash
# Download combined checksums
curl -LO https://github.com/kfcafe/aush/releases/latest/download/SHA256SUMS.txt

# Verify all files
sha256sum -c SHA256SUMS.txt --ignore-missing
```

Expected output:
```
aush-linux-x86_64.tar.gz: OK
aush-linux-x86_64-musl.tar.gz: OK
aush-macos-x86_64.tar.gz: OK
aush-macos-aarch64.tar.gz: OK
```

## Signature Testing

Current releases don't include GPG signatures, but future releases might. When available:

```bash
# Download public key
curl https://github.com/kfcafe/aush.gpg | gpg --import

# Verify signature
gpg --verify aush-*.tar.gz.sig aush-*.tar.gz
```

## Regression Testing

For each release, verify these features still work:

```bash
# POSIX compatibility
aush -c '
  # Comments work
  x=10
  [ $x -gt 5 ] && echo "Arithmetic comparison"

  # Functions
  greet() { echo "Hello $1"; }
  greet "AUSH"

  # Case statement
  case $x in
    10) echo "Found 10" ;;
    *) echo "Other" ;;
  esac
'
```

```bash
# Built-in commands
aush -c '
  # File operations
  echo "test" | cat

  # Directory operations
  mkdir -p /tmp/test
  cd /tmp/test
  pwd

  # List directory
  touch file.txt
  ls -la

  # Cleanup
  cd /
  rm -rf /tmp/test
'
```

```bash
# Performance baseline
time aush -c 'echo "startup test"'
# Should be <10ms for cold start
```

## Error Handling Testing

Test that errors are handled gracefully:

```bash
# Non-existent command
aush -c 'nonexistent_command 2>&1'
# Should show appropriate error

# Syntax error
aush -c 'if [ 1; then echo test' 2>&1
# Should show parse error

# Exit code
aush -c 'exit 42'
echo $?
# Should output 42
```

## Documentation Verification

- [ ] `aushd` daemon archive or full archive verified where daemon mode is tested
- [ ] README.md installation section is accurate
- [ ] docs/INSTALLATION.md is up-to-date
- [ ] Release notes are clear and helpful
- [ ] Examples in documentation still work
- [ ] Links to resources are correct

## Performance Baselines

Record these metrics for each release:

```bash
# Startup time (cold)
time aush -c 'echo startup' > /dev/null

# Startup time (daemon)
aushd start
time aush -c 'echo daemon' > /dev/null
aushd stop

# Memory usage
aush -c 'ps aux | grep aush'

# Binary size
ls -lh /usr/local/bin/aush
```

## Test Summary Template

Use this template to document test results:

```
Release: v0.x.x
Date: YYYY-MM-DD
Tester: Name
System: macOS/Linux version

macOS Intel (x86_64):
  - [ ] Binary downloads: PASS/FAIL
  - [ ] Checksum verification: PASS/FAIL
  - [ ] Installation: PASS/FAIL
  - [ ] Basic tests: PASS/FAIL
  - [ ] Functionality tests: PASS/FAIL
  - [ ] Daemon mode: PASS/FAIL

macOS ARM (aarch64):
  - [ ] Binary downloads: PASS/FAIL
  - [ ] Checksum verification: PASS/FAIL
  - [ ] Installation: PASS/FAIL
  - [ ] Basic tests: PASS/FAIL
  - [ ] Functionality tests: PASS/FAIL

Linux x86_64:
  - [ ] Binary downloads: PASS/FAIL
  - [ ] Checksum verification: PASS/FAIL
  - [ ] Installation: PASS/FAIL
  - [ ] Basic tests: PASS/FAIL
  - [ ] Functionality tests: PASS/FAIL

Homebrew:
  - [ ] Install via brew: PASS/FAIL
  - [ ] Functionality: PASS/FAIL

Issues found:
- None

Overall: READY FOR RELEASE
```

## Continuous Testing

The GitHub Actions workflow automatically tests releases, but manual testing on fresh machines provides additional confidence:

1. **Automated tests** (GitHub Actions):
   - Run automatically for every release
   - Test on clean Ubuntu, macOS Intel, macOS ARM
   - Verify checksums and basic functionality

2. **Manual tests** (this guide):
   - Performed on actual machines by human testers
   - Verify real-world installation experience
   - Catch issues that CI might miss
   - Test user documentation accuracy

## Troubleshooting

### Binary won't execute on macOS

```bash
# Check quarantine attribute
xattr -l aush

# Remove if quarantined
xattr -d com.apple.quarantine aush

# Try running again
./aush -c 'echo test'
```

### Checksum mismatch

```bash
# Re-download files (might be incomplete)
rm *.tar.gz *.txt
curl -LO https://github.com/kfcafe/aush/releases/latest/download/...

# Try different hash tool
md5 aush-*.tar.gz
```

### Daemon port in use

```bash
# Find process using daemon port
lsof -i :9090  # Default daemon port

# Kill existing daemon
pkill -f aushd
```

## Sign-Off

After completing all tests, document results:

```bash
# Create test report
cat > RELEASE_TEST_RESULTS.md << 'EOF'
# Test Results for v0.x.x

Date: YYYY-MM-DD
Tester: Your Name

## Summary
All tests passed successfully.

## Platform Results
- macOS Intel: PASS
- macOS ARM: PASS
- Linux: PASS
- Homebrew: PASS

## Ready for Release: YES
EOF
```

## Next Steps

After successful testing and explicit approval:
1. Merge any final fixes to main branch
2. Create release tag: `git tag v0.x.x && git push origin v0.x.x`
3. Wait for GitHub Actions to build and publish
4. Verify release is available on GitHub
5. Test installation from release (user perspective)
6. Close related issues
7. Announce release
