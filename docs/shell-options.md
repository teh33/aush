# Shell Options (set command)

AUSH supports standard shell options that control the behavior of the shell during script execution. These options are managed using the `set` builtin command.

## Overview

Shell options can be set (enabled) using `-` and unset (disabled) using `+`. Options affect the current shell and any subshells spawned from it.

## Supported Options

### `-e` (errexit)

Exit immediately if any command exits with a non-zero status.

```bash
# Enable errexit
set -e

# This will exit the shell after false
false
echo "This will not print"
```

**Use Case**: Useful in scripts to catch errors early and prevent cascading failures.

**Disable**:
```bash
set +e
```

### `-u` (nounset)

Treat unset variables as an error when performing parameter expansion.

```bash
# Enable nounset
set -u

# This will cause an error
echo $UNDEFINED_VARIABLE

# This works fine
DEFINED="hello"
echo $DEFINED
```

**Use Case**: Catch typos in variable names and ensure all required variables are set.

**Disable**:
```bash
set +u
```

### `-x` (xtrace)

Print commands and their arguments as they are executed, preceded by `+`.

```bash
# Enable xtrace
set -x

# This will print: + echo hello world
echo hello world

# This will print: + pwd
pwd
```

**Use Case**: Debugging scripts by seeing exactly what commands are being executed.

**Disable**:
```bash
set +x
```

### `-o pipefail`

The return value of a pipeline is the status of the last command to exit with a non-zero status, or zero if no command exited with a non-zero status.

```bash
# Without pipefail (default)
false | echo "hello"  # Exit code: 0 (from echo)

# With pipefail
set -o pipefail
false | echo "hello"  # Exit code: non-zero (from false)
```

**Use Case**: Catch errors in pipelines where an intermediate command fails but the last command succeeds.

**Disable**:
```bash
set +o pipefail
```

## Usage Examples

### Setting Single Options

```bash
# Enable errexit
set -e

# Enable nounset
set -u

# Enable xtrace
set -x
```

### Setting Multiple Options

You can combine multiple short options:

```bash
# Enable errexit, nounset, and xtrace
set -eux
```

### Viewing Current Options

Run `set` without arguments to view the current state of all options:

```bash
set
```

Output example:
```
set -e
set +u
set +x
set +o pipefail
```

### Long Form Options

Some options have long names that must be used with `-o` or `+o`:

```bash
# Set pipefail using long form
set -o pipefail

# Unset pipefail
set +o pipefail
```

## Option Combinations

### Strict Mode

For robust scripts, combine these options:

```bash
set -euo pipefail
```

This combination:
- `-e`: Exits on any error
- `-u`: Errors on undefined variables
- `-o pipefail`: Catches errors in pipelines

### Debug Mode

For debugging:

```bash
set -x
```

Or combine with other options:

```bash
set -eux
```

## Behavior Details

### errexit (`-e`)

- Applies to commands, pipelines, and subshells
- Does NOT apply to commands in conditional statements (`if`, `while`, `until`)
- Does NOT apply to commands whose return value is tested with `||` or `&&`

Example:
```bash
set -e

# This will NOT exit the shell, even if grep fails
if grep pattern file; then
    echo "Found"
fi

# This will NOT exit the shell if grep fails
grep pattern file || echo "Not found"
```

### nounset (`-u`)

- Applies to variable expansions like `$VAR` and `${VAR}`
- Special variables like `$?`, `$#`, `$@` are always defined
- Empty strings are different from unset variables

Example:
```bash
set -u

VAR=""        # This is OK (empty string)
echo $VAR     # This is OK (prints empty line)
echo $UNDEF   # This causes an error
```

### xtrace (`-x`)

- Prints to stderr (not stdout)
- Shows expanded values of variables
- Useful for debugging complex scripts

Example:
```bash
set -x
NAME="world"
echo "Hello $NAME"
# Prints to stderr: + echo 'Hello world'
# Prints to stdout: Hello world
```

### pipefail (`-o pipefail`)

- Only affects pipelines
- Returns the exit code of the first failing command in the pipeline
- If all commands succeed, returns 0

Example:
```bash
set -o pipefail

# Exit code will be from 'false', not 'echo'
false | echo "hello" | cat
echo $?  # Non-zero
```

## Subshells and Option Inheritance

Options are inherited by subshells:

```bash
set -e

# Subshell inherits errexit
(
    false
    echo "Won't print"
)
```

However, changes in subshells don't affect the parent:

```bash
set +e

# This sets errexit only in subshell
(set -e; false; echo "Won't print")

# Parent still has errexit disabled
false
echo "Will print"
```

## Common Patterns

### Safe Script Template

```bash
#!/usr/bin/env aush
set -euo pipefail

# Your script here
```

### Temporary Option Changes

```bash
# Save current state
set +e
# Do something that might fail
risky_command || echo "Failed but continuing"
# Restore errexit
set -e
```

### Debugging Specific Sections

```bash
# Enable tracing for specific section
set -x
complex_operation
set +x

# Rest of script runs without tracing
```

## Implementation Notes

- Options are stored in the `ShellOptions` struct in the Runtime
- Options can be queried with `runtime.get_option()`
- Options can be set with `runtime.set_option()`
- The `set` builtin command provides the user interface

## See Also

- Bash manual: `man bash` (search for "set")
- POSIX specification for `set`
- AUSH executor implementation: `src/executor/mod.rs`
- AUSH pipeline implementation: `src/executor/pipeline.rs`
