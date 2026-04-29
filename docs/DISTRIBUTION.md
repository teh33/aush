# AUSH Distribution and Release Process

This document describes how AUSH is built, packaged, and distributed to users.

## Overview

AUSH follows a modern CI/CD approach with:
- **Automated builds** via GitHub Actions
- **Cross-platform compilation** for macOS (Intel + ARM) and Linux
- **Binary distribution** via GitHub Releases
- **Homebrew support** for easy macOS installation
- **Checksum verification** for security
- **Automated testing** of installed binaries

## Release Workflow

### Triggering a Release

A release is triggered by:

1. **Creating a version tag** (automatic release):
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```

2. **Manual workflow dispatch** (on-demand release):
   - Go to GitHub Actions
   - Select "Release" workflow
   - Click "Run workflow"
   - Enter version (e.g., `v0.2.0`)

### What Happens Automatically

When a release is triggered:

1. **Create Release** (create-release job):
   - Generates GitHub Release
   - Creates release notes automatically
   - Adds installation instructions

2. **Build Binaries** (build-release job, parallel):
   - Compiles for 4 target platforms:
     - Linux x86_64 (glibc)
     - Linux x86_64 (musl - static)
     - macOS Intel (x86_64)
     - macOS ARM (aarch64)
   - Uses official Rust toolchains
   - Strips binaries for smaller size
   - Creates tar.gz archives
   - Generates SHA256 checksums

3. **Aggregate Checksums** (aggregate-checksums job):
   - Collects all SHA256SUMS files
   - Creates unified SHA256SUMS.txt
   - Uploads to release

4. **Test Release** (test-release job, parallel):
   - Downloads and verifies checksums
   - Tests binary functionality on:
     - Ubuntu Linux
     - macOS Intel
     - macOS ARM
   - Validates POSIX features

## Binary Distribution

### Artifact Format

Each release includes:

```
aush-{platform}.tar.gz          # Primary `aush` binary
aush-{platform}-daemon.tar.gz   # Daemon helper (`aushd`)
aush-{platform}-full.tar.gz     # `aush` and `aushd`
aush-{platform}-SHA256SUMS.txt  # Checksums
SHA256SUMS.txt                  # All checksums combined
```

Platforms:
- `linux-x86_64` - Linux glibc (requires glibc 2.29+)
- `linux-x86_64-musl` - Linux static musl (portable, no dependencies)
- `macos-x86_64` - macOS Intel
- `macos-aarch64` - macOS ARM (Apple Silicon)

### File Sizes

Typical sizes (uncompressed / compressed):
- **macOS**: ~4.7MB / ~1.2MB
- **Linux**: ~5.1MB / ~1.3MB

## Compatibility During Rebrand

Release archives use `aush-*` names and install `aush` as the primary executable. Homebrew installs `aushd` when it is present in the selected archive.

## Homebrew Distribution

### Formula Location

The Homebrew formula is stored in `/homebrew/Formula/aush.rb`:

```ruby
class Aush < Formula
  desc "High-performance, POSIX-compliant shell written in Rust"
  homepage "https://github.com/kfcafe/aush"
  version "0.1.0"

  # Detects platform and downloads correct binary
  # Installs to $(brew --prefix)/bin/aush
end
```

### How Users Install via Homebrew

```bash
brew tap kfcafe/aush https://github.com/kfcafe/aush
brew install aush
```

This:
1. Clones the tap repository
2. Reads the formula from `/homebrew/Formula/aush.rb`
3. Downloads the appropriate binary for their platform
4. Verifies SHA256
5. Installs to Homebrew's bin directory
6. Makes it available in PATH

### Updating the Homebrew Formula

The formula currently has placeholder SHA256 values that need to be updated for each release. This is handled by:

1. **Manual update** (if needed):
   ```bash
   # Get actual checksums from GitHub release
   curl https://github.com/kfcafe/aush/releases/latest/download/SHA256SUMS.txt

   # Update homebrew/Formula/aush.rb with actual values
   ```

2. **Automated update** (ideal future state):
   - Add GitHub Actions job that updates formula
   - Calculate SHA256s from built binaries
   - Commit to main branch

## Direct Binary Installation

Users can download binaries directly from GitHub:

```bash
# Latest release, automatic platform detection
curl -s https://api.github.com/repos/kfcafe/aush/releases/latest | \
  jq -r '.assets[] | select(.name | contains("linux-x86_64")) | .browser_download_url'
```

## Cross-Compilation Details

### macOS Compilation

- **macOS 13** runner for Intel targets (x86_64-apple-darwin)
- **macOS 14** runner for ARM targets (aarch64-apple-darwin)
- Uses official dtolnay/rust-toolchain action
- Strips binaries post-build
- Requires ~5 minutes per build

### Linux Compilation

- **Ubuntu latest** runner
- Two variants:
  - **glibc**: Standard Linux, depends on system libc (faster, smaller)
  - **musl**: Statically linked, completely portable (larger, more compatible)
- musl-tools installed for musl builds
- Strips binaries post-build
- Requires ~5 minutes per build

## Checksum Verification

### Generation

During build:
```bash
# For each platform
sha256sum aush-*.tar.gz > aush-{platform}-SHA256SUMS.txt
```

### Aggregation

All checksums combined into single `SHA256SUMS.txt`:
```
abc123...  aush-linux-x86_64.tar.gz
def456...  aush-linux-x86_64-musl.tar.gz
ghi789...  aush-macos-x86_64.tar.gz
jkl012...  aush-macos-aarch64.tar.gz
```

### User Verification

**Linux:**
```bash
sha256sum -c SHA256SUMS.txt --ignore-missing
```

**macOS:**
```bash
shasum -a 256 -c SHA256SUMS.txt --ignore-missing
```

## Testing Strategy

### Automated Testing

Each release is tested by `test-release` job:

1. **Download binary**
2. **Verify checksum**
3. **Test version** (--version or -c 'echo test')
4. **Test basic commands**:
   - `echo` - basic output
   - `pwd` - directory navigation
   - Variable expansion
5. **Test POSIX features**:
   - Conditionals (if/then)
   - Loops (for/do)
   - Arithmetic expansion

Tested on:
- Ubuntu Linux (latest)
- macOS Intel (13)
- macOS ARM (14)

### Manual Testing (for acceptance)

Before marking as complete, test on real machines:

1. **macOS ARM (Apple Silicon)**:
   ```bash
   curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-aarch64.tar.gz
   tar xzf aush-macos-aarch64.tar.gz
   ./aush -c 'echo "Hello from AUSH"'
   ```

2. **macOS Intel**:
   ```bash
   curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-macos-x86_64.tar.gz
   tar xzf aush-macos-x86_64.tar.gz
   ./aush -c 'echo "Hello from AUSH"'
   ```

3. **Linux x86_64**:
   ```bash
   curl -LO https://github.com/kfcafe/aush/releases/latest/download/aush-linux-x86_64.tar.gz
   tar xzf aush-linux-x86_64.tar.gz
   ./aush -c 'echo "Hello from AUSH"'
   ```

## Troubleshooting Release Issues

### Build Failures

Check GitHub Actions logs:
1. Go to https://github.com/kfcafe/aush/actions
2. Select "Release" workflow
3. Click failed job
4. Review logs

Common issues:
- **Rust version**: Update `dtolnay/rust-toolchain` action
- **Dependencies**: Check dependencies in Cargo.toml
- **Target unavailable**: Some toolchain versions lack certain targets

### Checksum Mismatches

If users report checksum mismatches:
1. Re-download SHA256SUMS.txt from latest release
2. Verify binary was downloaded completely
3. Check network issues during download

### Binary Doesn't Run

If downloaded binary won't execute:
1. Verify file is executable: `chmod +x aush`
2. Check architecture matches: `uname -m`
3. For macOS: Verify not quarantined: `xattr -d com.apple.quarantine ./aush`
4. For Linux: Check glibc version: `./aush -c 'echo test'` (if musl, should work always)

## Performance Metrics

### Build Times

- **Linux glibc build**: ~5 minutes
- **Linux musl build**: ~5 minutes
- **macOS Intel build**: ~5 minutes
- **macOS ARM build**: ~5 minutes
- **Total workflow**: ~15-20 minutes (parallel builds)

### Binary Sizes

| Platform | Uncompressed | Compressed |
|----------|-------------|-----------|
| macOS ARM | 4.7 MB | 1.2 MB |
| macOS Intel | 4.7 MB | 1.2 MB |
| Linux glibc | 5.1 MB | 1.3 MB |
| Linux musl | 5.2 MB | 1.3 MB |

### Startup Performance

- **Cold start**: 4.9ms
- **Daemon mode**: 0.4ms
- **Binary load time**: <0.5ms

## Future Improvements

1. **Auto-update Homebrew formula** via GitHub Actions
2. **Additional platforms**: Windows (WSL2), BSD, additional Linux architectures
3. **Signed releases**: GPG signature verification
4. **Docker images**: Official AUSH Docker image
5. **Distribution mirrors**: CDN distribution for faster downloads
6. **Version upgrade checks**: In-app update notifications

## Security Considerations

### Current Security

- GitHub Actions build from public source
- Checksums provided for all binaries
- No code signing (open source, public builds)
- No telemetry or tracking in binaries
- Binaries are deterministic (rebuilding same version produces same hash)

### Recommended Best Practices

1. Always verify checksums before installation
2. Download only from official GitHub releases
3. Keep AUSH updated for security patches
4. Report security issues privately

## Maintenance

### Version Numbering

AUSH uses semantic versioning (MAJOR.MINOR.PATCH):
- `v0.1.0` - First release
- `v0.2.0` - Feature release
- `v0.2.1` - Bug fix

### Release Frequency

- **Regular releases**: Every 2-4 weeks
- **Patch releases**: As needed for critical bugs
- **Beta releases**: Marked with `-beta` suffix (e.g., `v0.2.0-beta.1`)

### Supported Versions

Currently, only the latest version is actively supported. Users on older versions are encouraged to upgrade.

## Contact and Support

- **Issues**: https://github.com/kfcafe/aush/issues
- **Discussions**: https://github.com/kfcafe/aush/discussions
- **Email**: See GitHub profile

## License

This distribution process and all tooling is part of AUSH, which is dual-licensed under MIT or Apache-2.0.
