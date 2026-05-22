# Homebrew Tap for AUSH

This directory contains the Homebrew formula for [AUSH](https://github.com/kfcafe/aush), a public-alpha Unix-style shell written in Rust.

## Installation

```bash
brew tap kfcafe/aush https://github.com/kfcafe/aush
brew install aush
```

## Usage

The formula installs `aush` and, when present in the selected release archive, `aushd`.

```bash
aush --no-rc -c 'echo hello'
aush
```

## Setting as Default Shell

AUSH is alpha software. Prefer a terminal-specific trial before changing your system login shell. If you do change it, keep another terminal open and know your rollback shell.

```bash
AUSH_PATH="$(brew --prefix)/bin/aush"
grep -qxF "$AUSH_PATH" /etc/shells || echo "$AUSH_PATH" | sudo tee -a /etc/shells
chsh -s "$AUSH_PATH"

# Roll back if needed
chsh -s /bin/zsh
```

## Daemon Mode

`aushd` is experimental daemon infrastructure for trusted local clients.

```bash
aushd --help
```

## Updating

```bash
brew update
brew upgrade aush
```

## Uninstalling

```bash
brew uninstall aush
brew untap kfcafe/aush
```
