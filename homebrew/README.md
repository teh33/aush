# Homebrew Tap for AUSH

This directory contains the Homebrew formula for [AUSH](https://github.com/opus-workshop/aush), a high-performance POSIX-compliant shell written in Rust.

## Installation

```bash
# Add the tap
brew tap opus-workshop/aush https://github.com/opus-workshop/aush

# Install AUSH
brew install aush
```

## Usage

The formula installs `aush` as the primary command and installs `aushd` when the selected archive includes daemon support.


After installation, you can run AUSH:

```bash
aush                      # Start interactive shell
aush -c "echo hello"      # Run a command
aush script.sh            # Run a script
```

## Setting as Default Shell

To use AUSH as your default shell:

```bash
# Add to allowed shells
echo "$(brew --prefix)/bin/aush" | sudo tee -a /etc/shells

# Change your shell
chsh -s "$(brew --prefix)/bin/aush"
```

## Daemon Mode

For ultra-fast startup (~0.4ms), use daemon mode:

```bash
aushd start               # Start the daemon
aush -c "ls"              # Commands use the daemon
aushd stop                # Stop the daemon
```

## Updating

```bash
brew update
brew upgrade aush
```

## Uninstalling

```bash
brew uninstall aush
brew untap opus-workshop/aush
```
