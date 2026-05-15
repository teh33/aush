# Installing AUSH

AUSH is a Rust shell for Unix-style command execution, native builtins, structured output, and automation-heavy workflows. This document covers installation and safe alpha trial paths. AUSH 0.1.0 is public alpha / beta-candidate software, not a guaranteed drop-in login-shell replacement.

## Quick Start

### macOS (Homebrew) - Recommended

The easiest way to install AUSH on macOS:

```bash
brew tap kfcafe/aush https://github.com/kfcafe/aush
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
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-aarch64.tar.gz
tar xzf aush-macos-aarch64.tar.gz
sudo mv aush /usr/local/bin/

# macOS Intel
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-x86_64.tar.gz
tar xzf aush-macos-x86_64.tar.gz
sudo mv aush /usr/local/bin/

# Linux x86_64
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64.tar.gz
tar xzf aush-linux-x86_64.tar.gz
sudo mv aush /usr/local/bin/

# Linux x86_64 (static binary - more portable)
curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64-musl.tar.gz
tar xzf aush-linux-x86_64-musl.tar.gz
sudo mv aush /usr/local/bin/
```

#### Verify the Download

Each release includes SHA256 checksums for verification:

```bash
# Download the checksum file
curl -LO https://github.com/kfcafe/aush/releases/latest/download/SHA256SUMS.txt

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

The crates.io package is `aush`. It installs `aush` as the primary command.

To install from git instead:

```bash
cargo install --git https://github.com/kfcafe/aush
```

To install a specific git tag:

```bash
cargo install --git https://github.com/kfcafe/aush --tag v0.1.0
```

### Build from Source

Clone the repository and build:

```bash
git clone https://github.com/kfcafe/aush.git
cd aush
cargo build --release
sudo cp target/release/aush /usr/local/bin/
```

**Requirements:**
- Rust 1.70 or later
- Cargo

## Trying AUSH as a Login Shell

For the public alpha, prefer a terminal-specific trial before changing your
system login shell. Keep a known-good shell such as `zsh` or `bash` available for
rollback.

### Ghostty trial first

Configure Ghostty to launch AUSH without changing your account shell:

```text
command = /absolute/path/to/aush
command-arg = --login
```

AUSH-specific startup files are:

- `~/.aush_profile` for interactive login shells;
- `~/.aushrc` for interactive shells.

AUSH does not automatically source `/etc/profile`, `~/.profile`, `.zprofile`,
`.zshrc`, or `.bashrc`. Put required PATH/tool setup in `~/.aush_profile`, or
source a shared file explicitly.

Smoke check a fresh terminal before doing real work:

```bash
command -v aush
aush --version
echo "$PATH"
command -v git
command -v cargo
command -v imp  # if you use imp
type -a imp     # optional: inspect duplicates/precedence
```

Rollback is immediate: set Ghostty back to `/bin/zsh` or your previous shell and
open a new window.

### System-wide `chsh` after a successful trial

Only after a successful terminal-specific trial, add AUSH to `/etc/shells` and
change the account shell:

```bash
AUSH_PATH="$(command -v aush)"
grep -qxF "$AUSH_PATH" /etc/shells || echo "$AUSH_PATH" | sudo tee -a /etc/shells
chsh -s "$AUSH_PATH"
```

Rollback:

```bash
chsh -s /bin/zsh
# or, if your normal zsh is Homebrew-installed:
# chsh -s /opt/homebrew/bin/zsh
```

Release archives include the `aush` shell and install `aushd` when daemon support
is packaged. `aushd` is experimental and should only be run where local clients
with access to its socket are trusted.

## Daemon Mode (Optional)

For ultra-fast startup times, use AUSH's daemon mode:

```bash
# Start the daemon
aushd start

# Commands now connect to the daemon (much faster)
aush -c "ls"      # ~0.4ms instead of ~4.9ms

# Stop the daemon
aushd stop
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
cargo install --git https://github.com/kfcafe/aush --force
```

## Uninstalling

### Homebrew

```bash
brew uninstall aush
brew untap kfcafe/aush
```

### Binary Install

```bash
sudo rm /usr/local/bin/aush
```

### From Source

```bash
sudo rm /usr/local/bin/aush
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

- **Documentation**: https://github.com/kfcafe/aush
- **Issues**: https://github.com/kfcafe/aush/issues
- **Discussions**: https://github.com/kfcafe/aush/discussions

## License

AUSH is dual-licensed under MIT or Apache-2.0 (your choice).
