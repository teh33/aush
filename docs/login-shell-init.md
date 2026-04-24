# Login Shell Initialization

This document describes AUSH's login shell initialization behavior and configuration file system.

## Overview

AUSH supports POSIX-style shell initialization with profile files for login shells and RC files for interactive shells. This allows users to customize their environment and set up aliases, functions, and environment variables.

## Shell Types

### Login Shell

A login shell is the first shell you get when you log into a system. AUSH detects login shells in two ways:

1. **Automatic Detection**: When the first character of `argv[0]` is `-` (e.g., `-aush`)
2. **Explicit Flag**: Using the `--login` or `-l` flag

```bash
aush --login          # Explicitly start as login shell
-aush                 # Started as login shell by system
```

### Interactive Shell

An interactive shell is a shell where you can type commands interactively. AUSH automatically detects if it's running in interactive mode by checking if stdin is a TTY.

## Configuration Files

### ~/.aush_profile

Sourced when AUSH starts as a **login shell**. This file is typically used for:

- Setting environment variables (`export PATH=$PATH:/custom/bin`)
- Setting up the terminal (`export TERM=xterm-256color`)
- Loading system-wide settings
- One-time initialization tasks

Example `~/.aush_profile`:

```bash
# Set up PATH
export PATH=$HOME/bin:/usr/local/bin:$PATH

# Set environment variables
export EDITOR=vim
export VISUAL=vim

# Set up language and locale
export LANG=en_US.UTF-8

# Custom greeting
echo "Welcome to AUSH Shell!"
```

### ~/.aushrc

Sourced for **all interactive shells** (including login shells). This file is typically used for:

- Defining aliases
- Setting shell options
- Defining functions
- Setting up prompt customization

Example `~/.aushrc`:

```bash
# Aliases
export LS_ALIAS=ls -la
export GREP_ALIAS=grep --color=auto

# Functions
fn greet(name) {
    echo "Hello, $name!"
}

# Shell history settings (when implemented)
# export HISTSIZE=10000
# export HISTFILE=$HOME/.aush_history
```

## Initialization Order

When AUSH starts, it initializes in the following order:

1. **Set Environment Variables**:
   - `$SHELL` - Path to the AUSH executable
   - `$TERM` - Terminal type (if not already set)
   - `$USER` - Username (if not already set)
   - `$HOME` - Home directory (if not already set)

2. **Login Shell** (if `--login` or argv[0] starts with `-`):
   - Source `~/.aush_profile` (if it exists)

3. **Interactive Shell** (if stdin is a TTY):
   - Source `~/.aushrc` (if it exists)

4. **Start Shell**:
   - Enter interactive mode with REPL
   - OR execute the provided command/script

## Command-Line Flags

### --login, -l

Forces AUSH to behave as a login shell, sourcing `~/.aush_profile`.

```bash
aush --login
aush -l
```

### --no-rc, --norc

Skips sourcing all configuration files (both `~/.aush_profile` and `~/.aushrc`).

```bash
aush --no-rc           # Start without loading config files
aush --login --no-rc   # Login shell but skip config files
```

### -c command

Execute a command and exit. Does not source config files.

```bash
aush -c "echo hello"
aush -c "ls -la | grep txt"
```

## The source Builtin

AUSH provides a `source` builtin command to execute commands from a file in the current shell context. This is useful for:

- Loading configuration files manually
- Reloading configuration after changes
- Sourcing utility scripts

```bash
source ~/.aushrc               # Reload aushrc
source ~/scripts/aliases.aush  # Load custom aliases
source ~/.aush_profile         # Reload profile
```

### Features

- **Tilde Expansion**: `source ~/.aushrc` expands `~` to home directory
- **Relative Paths**: Resolved relative to current working directory
- **Error Handling**: Continues executing even if individual lines fail
- **Comments**: Lines starting with `#` are ignored
- **Empty Lines**: Blank lines are skipped

### Syntax

```bash
source <file>
source ~/config.aush
source /absolute/path/to/file.aush
source relative/path/to/file.aush
```

## Environment Variables

AUSH automatically sets the following environment variables if they are not already defined:

### $SHELL

Path to the AUSH executable. Used by other programs to determine the user's shell.

```bash
echo $SHELL  # /usr/local/bin/aush
```

### $TERM

Terminal type. Defaults to `xterm-256color` if not set.

```bash
echo $TERM  # xterm-256color
```

### $USER

Current username. Derived from `$LOGNAME` or system user information.

```bash
echo $USER  # yourusername
```

### $HOME

Home directory path. Derived from system home directory.

```bash
echo $HOME  # /home/yourusername
```

## Best Practices

### Separate Concerns

- Put **environment variables** in `~/.aush_profile`
- Put **interactive settings** (aliases, functions) in `~/.aushrc`

### Keep It Fast

Configuration files are sourced on every shell start. Keep them fast by:

- Avoiding expensive operations
- Using conditional logic to skip unnecessary work
- Moving rarely-used functions to separate files

### Use Comments

Document your configuration files well:

```bash
# ~/.aushrc - AUSH shell interactive configuration

# Aliases for common operations
export LS_ALIAS=ls -lah
export GREP_ALIAS=grep --color=auto

# Development shortcuts
export DEV_DIR=$HOME/projects
```

### Test Configuration

Test your configuration files before using them:

```bash
# Test without loading your real config
aush --no-rc -c "source ~/test_config.aush"
```

## Compatibility Notes

### POSIX Shells (bash, zsh)

AUSH's initialization is inspired by POSIX shells but has some differences:

- **bash**: Uses `~/.bash_profile` or `~/.profile` for login, `~/.bashrc` for interactive
- **zsh**: Uses `~/.zprofile` for login, `~/.zshrc` for interactive
- **aush**: Uses `~/.aush_profile` for login, `~/.aushrc` for interactive

### Migration from Other Shells

If migrating from bash or zsh, you can:

1. Copy relevant settings from `~/.bash_profile` to `~/.aush_profile`
2. Copy relevant settings from `~/.bashrc` to `~/.aushrc`
3. Adjust for AUSH syntax differences (especially function definitions)

## Examples

### Complete Login Profile

```bash
# ~/.aush_profile - Login shell initialization

# Path configuration
export PATH=$HOME/bin:$HOME/.local/bin:/usr/local/bin:$PATH

# Language and locale
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Editor configuration
export EDITOR=vim
export VISUAL=vim

# XDG directories
export XDG_CONFIG_HOME=$HOME/.config
export XDG_DATA_HOME=$HOME/.local/share
export XDG_CACHE_HOME=$HOME/.cache

# Development environment
export RUST_BACKTRACE=1
export CARGO_HOME=$HOME/.cargo

# Less pager configuration
export LESS=-R
export LESS_TERMCAP_mb=$'\E[1;31m'
export LESS_TERMCAP_md=$'\E[1;36m'
export LESS_TERMCAP_me=$'\E[0m'

# Source .aushrc for interactive login shells
if [ -f ~/.aushrc ]; then
    source ~/.aushrc
fi
```

### Complete Interactive RC

```bash
# ~/.aushrc - Interactive shell configuration

# Aliases
export LS_ALIAS=ls -lah --color=auto
export GREP_ALIAS=grep --color=auto
export EGREP_ALIAS=egrep --color=auto

# Git aliases (when git command is available)
export G_ALIAS=git
export GS_ALIAS=git status
export GA_ALIAS=git add
export GC_ALIAS=git commit

# Directory shortcuts
export PROJ=$HOME/projects
export DOC=$HOME/Documents
export DL=$HOME/Downloads

# Functions
fn mkcd(dir) {
    mkdir -p $dir && cd $dir
}

fn extract(file) {
    if [ -f $file ]; then
        # Extraction logic here
        echo "Extracting $file..."
    fi
}

# Welcome message
echo "AUSH shell ready. Type 'exit' to quit."
```

## Troubleshooting

### Config File Not Loading

1. Check file exists: `ls -la ~/.aushrc ~/.aush_profile`
2. Check file permissions: `chmod 644 ~/.aushrc ~/.aush_profile`
3. Check for syntax errors: `aush --no-rc -c "source ~/.aushrc"`

### Variables Not Set

1. Verify export statement: `export VAR=value` not `VAR=value`
2. Check if config file is being sourced (add `echo "Loading aushrc"` at top)
3. Use `--login` flag if you need login shell behavior

### Slow Startup

1. Profile your config files by adding timing statements
2. Remove expensive operations
3. Consider lazy-loading functions and aliases

## Future Enhancements

Planned features for future versions:

- `~/.aush_logout` for logout cleanup
- `$AUSHOPTS` for shell option configuration
- `~/.config/aush/aushrc` for XDG-compliant configuration
- Per-directory `.aushrc` files (similar to `.envrc`)
