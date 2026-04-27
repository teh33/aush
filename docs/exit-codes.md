# Exit Code Propagation in AUSH

This document describes how exit codes work in the AUSH shell, including the `$?` special variable and conditional operators.

## Overview

AUSH implements POSIX-compliant exit code handling, allowing scripts to make decisions based on command success or failure. Every command execution sets the `$?` variable to reflect the exit status.

## The `$?` Special Variable

The `$?` variable contains the exit code of the last executed command:

- `0` indicates success
- Non-zero values indicate failure (typically `1-255`)

### Example

```bash
# Check exit code of last command
echo "hello"
echo $?  # Outputs: 0

# Try to read non-existent file
cat /nonexistent/file
echo $?  # Outputs: non-zero (typically 1)
```

### Implementation Details

- `$?` is automatically updated after every command, pipeline, or statement execution
- Initial value is `0` when the shell starts
- The variable is stored in the runtime's variable map with the key `"?"`
- Access via `runtime.get_last_exit_code()` and `runtime.set_last_exit_code(code)`

## Conditional Operators

AUSH supports two conditional operators for controlling execution flow based on exit codes.

### The `&&` Operator (Conditional AND)

Executes the right-hand command only if the left-hand command succeeds (exit code `0`).

```bash
# Both commands execute if first succeeds
mkdir /tmp/mydir && cd /tmp/mydir

# Second command skipped if first fails
cat /nonexistent/file && echo "This won't run"
```

#### Behavior

1. Execute left-hand statement
2. Update `$?` with left-hand exit code
3. If exit code is `0`, execute right-hand statement
4. If exit code is non-zero, skip right-hand statement
5. Return the exit code of the last executed statement

### The `||` Operator (Conditional OR)

Executes the right-hand command only if the left-hand command fails (exit code non-zero).

```bash
# Fallback execution
cat config.json || echo "Config not found"

# Skip second command if first succeeds
mkdir /tmp/dir || echo "Directory already exists"
```

#### Behavior

1. Execute left-hand statement
2. Update `$?` with left-hand exit code
3. If exit code is non-zero, execute right-hand statement
4. If exit code is `0`, skip right-hand statement
5. Return the exit code of the last executed statement

## Chaining Conditionals

You can chain multiple conditional operators together:

```bash
# All three execute if each succeeds
command1 && command2 && command3

# Execute first fallback if command1 fails, otherwise execute success
command1 || fallback1 && success1

# Complex chains
mkdir /tmp/test && cd /tmp/test && echo "Ready" || echo "Setup failed"
```

### Evaluation Order

Conditional operators are left-associative and evaluated left-to-right:

```bash
# Parsed as: (cmd1 && cmd2) && cmd3
cmd1 && cmd2 && cmd3

# Parsed as: (cmd1 || cmd2) || cmd3
cmd1 || cmd2 || cmd3

# Mixed: (cmd1 && cmd2) || cmd3
cmd1 && cmd2 || cmd3
```

## Pipeline Exit Codes

In pipelines, the exit code is determined by the last command in the pipeline:

```bash
# Exit code is from 'grep'
cat file.txt | grep pattern

# Exit code is from 'wc'
echo "test" | cat | wc -l
```

### Example

```bash
# Pipeline succeeds if last command succeeds
ls | grep foo | wc -l
echo $?  # 0 if wc succeeds, non-zero otherwise
```

### Note on `set -o pipefail`

The current implementation uses the default pipeline behavior (exit code of last command). Future versions may support `set -o pipefail` to return the exit code of the first failing command in the pipeline.

## Exit Codes with Builtins

Built-in commands follow the same exit code conventions:

- `cd /valid/path` → exit code `0`
- `cd /invalid/path` → exit code `1`
- `cat valid_file.txt` → exit code `0`
- `cat /nonexistent` → exit code `1`

All builtins properly set `$?` after execution.

## Exit Codes with External Commands

External commands return their process exit code:

```bash
# Most Unix commands use these conventions:
ls /existing/dir     # exit 0
ls /nonexistent/dir  # exit 1 or 2
grep pattern file    # exit 0 if found, 1 if not found, 2 if error
```

## Exit Codes in Functions

User-defined functions return the exit code of their last statement:

```bash
fn deploy(env: String) {
    echo "Deploying to $env"
    cd /app
    ./deploy.sh
}

# $? contains exit code of deploy.sh
deploy production
echo $?
```

## Exit Codes in Control Structures

### If Statements

Conditions evaluate to true/false, but don't affect `$?` in the current implementation:

```bash
if $condition {
    echo "true branch"
}
# $? is not affected by the if statement itself
```

### For Loops

The `$?` variable reflects the last command executed in the loop body:

```bash
for file in *.txt {
    cat $file
}
# $? contains exit code of last cat command
```

## Best Practices

1. **Check exit codes for critical operations:**
   ```bash
   mkdir /important/dir || exit 1
   ```

2. **Use conditionals for error handling:**
   ```bash
   deploy_app && notify_success || notify_failure
   ```

3. **Chain commands for dependent operations:**
   ```bash
   validate_config && build_app && run_tests && deploy
   ```

4. **Store exit codes when needed:**
   ```bash
   run_test
   let test_result = $?
   # Use test_result later
   ```

## Implementation Architecture

### Runtime (`src/runtime/mod.rs`)

- `set_last_exit_code(code: i32)` - Sets `$?` variable
- `get_last_exit_code() -> i32` - Retrieves current exit code

### AST (`src/parser/ast.rs`)

- `ConditionalAnd` - Represents `&&` operator
- `ConditionalOr` - Represents `||` operator

### Parser (`src/parser/mod.rs`)

- `parse_conditional_statement()` - Parses `&&` and `||` operators
- Creates proper AST nodes for conditional chains

### Executor (`src/executor/mod.rs`)

- `execute_conditional_and()` - Implements `&&` logic
- `execute_conditional_or()` - Implements `||` logic
- Updates `$?` after every statement execution

### Pipeline Executor (`src/executor/pipeline.rs`)

- Returns exit code of last command in pipeline
- Properly propagates exit codes through pipeline stages

## Testing

Comprehensive tests are available in `tests/exit_code_tests.rs`:

- `$?` variable after success and failure
- `&&` operator with success and failure cases
- `||` operator with success and failure cases
- Pipeline exit code propagation
- Exit codes with builtins and external commands
- Chained conditional operators
- Variable expansion of `$?`

Run tests with:

```bash
cargo test exit_code
```

## Future Enhancements

Potential improvements for future versions:

1. **`set -o pipefail`** - Return first failure in pipeline
2. **`set -e`** - Exit shell on first error
3. **`PIPESTATUS` array** - Store exit codes of all pipeline commands
4. **Error handlers** - `trap ERR` functionality
5. **Exit code ranges** - Custom exit codes for different error types
