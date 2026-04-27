# Installing AUSH

AUSH is a high-performance, POSIX-compliant shell written in Rust. This document covers all installation methods.

## Quick Start

### macOS (Homebrew) - Recommended

The easiest way to install AUSH on macOS:

```bash
brew tap opus-workshop/aush https://github.com/opus-workshop/aush
brew install aush
```

### Linux and macOS (Binary Download)

Pre-built binaries are available for:
- **macOS Intel** (x86_64)
- **macOS ARM** (Apple Silicon / aarch64)
- **Linux** (x86_64 glibc)
- **Linux** (x86_64 musl - static, portable)

#### Download and Install

```bash
# Determine your platform
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# macOS ARM (Apple Silicon)
curl -LO https://github.com/opus-workshop/aush/releases/latest/download/aush-macos-aarch64.tar.gz
tar xzf aush-macos-aarch64.tar.gz
sudo mv aush /usr/local/bin/

# macOS Intel
curl -LO https://github.com/opus-workshop/aush/releases/latest/download/aush-macos-x86_64.tar.gz
tar xzf aush-macos-x86_64.tar.gz
sudo mv aush /usr/local/bin/

# Linux x86_64
curl -LO https://github.com/opus-workshop/aush/releases/latest/download/aush-linux-x86_64.tar.gz
tar xzf aush-linux-x86_64.tar.gz
sudo mv aush /usr/local/bin/

# Linux x86_64 (static binary - more portable)
curl -LO https://github.com/opus-workshop/aush/releases/latest/download/aush-linux-x86_64-musl.tar.gz
tar xzf aush-linux-x86_64-musl.tar.gz
sudo mv aush /usr/local/bin/
```

#### Verify the Download

Each release includes SHA256 checksums for verification:

```bash
# Download the checksum file
curl -LO https://github.com/opus-workshop/aush/releases/latest/download/SHA256SUMS.txt

# Verify on Linux
sha256sum -c SHA256SUMS.txt --ignore-missing

# Verify on macOS
shasum -a 256 -c SHA256SUMS.txt --ignore-missing
```

### Cargo Install

If you have Rust installed:

```bash
cargo install aush
```

The crates.io package is `aush`. During the migration it installs `aush` as the primary command plus a legacy `rush` executable for existing scripts.

To install from git instead:

```bash
cargo install --git https://github.com/opus-workshop/aush
```

To install a specific git tag:

```bash
cargo install --git https://github.com/opus-workshop/aush --tag v0.1.0
```

### Build from Source

Clone the repository and build:

```bash
git clone https://github.com/opus-workshop/aush.git
cd aush
cargo build --release
sudo cp target/release/aush /usr/local/bin/
```

**Requirements:**
- Rust 1.70 or later
- Cargo

## Setting as Default Shell

After installing, you can make AUSH your default shell:

### macOS (Homebrew)

```bash
# Add AUSH to allowed shells
echo "$(brew --prefix)/bin/aush" | sudo tee -a /etc/shells

# Change your shell
chsh -s "$(brew --prefix)/bin/aush"
```

### Linux / macOS (Binary Install)

```bash
# Add AUSH to allowed shells
echo "/usr/local/bin/aush" | sudo tee -a /etc/shells

# Change your shell
chsh -s /usr/local/bin/aush
```

## Migrating from Rush

AUSH releases keep a legacy `rush` executable during the migration. New scripts and login-shell configuration should use `aush`; existing scripts that call `rush` can continue to work while you update them.

If you previously used Rush as your login shell, add the new AUSH path to `/etc/shells` and run `chsh` again:

```bash
echo "/usr/local/bin/aush" | sudo tee -a /etc/shells
chsh -s /usr/local/bin/aush
```

AUSH reads new `AUSH_*` environment variables and `~/.aushrc` first, with legacy `RUSH_*` and `~/.rushrc` fallback for compatibility. Release archives also include a legacy `rush` executable during this migration phase. The daemon helper binary is still named `rushd` and is installed when present in the selected package.

## Daemon Mode (Optional)

For ultra-fast startup times, use AUSH's daemon mode:

```bash
# Start the daemon
rushd start

# Commands now connect to the daemon (much faster)
aush -c "ls"      # ~0.4ms instead of ~4.9ms

# Stop the daemon
rushd stop
```

This is ideal for:
- CI/CD pipelines with many shell invocations
- Build systems (Make, scripts)
- Test suites that fork many processes
- AI agents making rapid shell calls

## Updating

### Homebrew

```bash
brew update
brew upgrade aush
```

### Binary Downloads

Download and install the latest release using the instructions above.

### Cargo

```bash
cargo install --git https://github.com/opus-workshop/aush --force
```

## Uninstalling

### Homebrew

```bash
brew uninstall aush
brew untap opus-workshop/aush
```

### Binary Install

```bash
sudo rm /usr/local/bin/aush
sudo rm /usr/local/bin/rush  # optional legacy compatibility executable, if installed
```

### From Source

```bash
sudo rm /usr/local/bin/aush
sudo rm /usr/local/bin/rush  # optional legacy compatibility executable, if installed
```

## Troubleshooting

### Binary not found after installation

Make sure `/usr/local/bin` is in your PATH:

```bash
echo $PATH
```

If not, add it:

```bash
export PATH="/usr/local/bin:$PATH"
```

### Permission denied when running aush

Make sure the binary is executable:

```bash
chmod +x /usr/local/bin/aush
```

### macOS: "aush cannot be opened because it is from an unidentified developer"

This is a macOS security feature. You can bypass it with:

```bash
xattr -d com.apple.quarantine /usr/local/bin/aush
```

Or via System Preferences:
1. Go to System Preferences > Security & Privacy
2. Click "Open Anyway" next to AUSH

### Linux: "cannot execute binary file: Exec format error"

This usually means the binary doesn't match your architecture. Make sure you downloaded the correct version:

```bash
# Check your architecture
uname -m    # Should be x86_64
uname -s    # Should be Linux

# Download the appropriate binary
# For x86_64: aush-linux-x86_64.tar.gz
# For ARM: aush-linux-aarch64.tar.gz (if available)
```

## Performance

AUSH binary sizes:

| Platform | Size (uncompressed) | Size (compressed) |
|----------|-------------------|-------------------|
| macOS ARM | ~4.7MB | ~1.2MB |
| macOS Intel | ~4.7MB | ~1.2MB |
| Linux x86_64 | ~5.1MB | ~1.3MB |
| Linux x86_64 musl | ~5.2MB | ~1.3MB |

Startup times:

| Mode | Time |
|------|------|
| Cold start | ~4.9ms |
| Daemon mode | ~0.4ms |

## Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| macOS | ARM (Apple Silicon) | Fully supported |
| macOS | Intel (x86_64) | Fully supported |
| Linux | x86_64 (glibc) | Fully supported |
| Linux | x86_64 (musl) | Fully supported |
| Linux | ARM/aarch64 | Available (community builds) |
| Windows | WSL2 | Tested and working |
| BSD | FreeBSD | Community support |

## Security

All binaries are:
- Built from source in GitHub Actions
- Signed checksums provided for verification
- Hosted on official GitHub releases
- No telemetry or tracking

## Getting Help

- **Documentation**: https://github.com/opus-workshop/aush
- **Issues**: https://github.com/opus-workshop/aush/issues
- **Discussions**: https://github.com/opus-workshop/aush/discussions

## License

AUSH is dual-licensed under MIT or Apache-2.0 (your choice).
