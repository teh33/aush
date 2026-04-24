# Error Recovery in AUSH Shell

## Overview

AUSH shell implements comprehensive error recovery to ensure stability and reliability. The shell is designed to handle errors gracefully and continue operating even in the face of parse errors, execution errors, or unexpected panics.

## Design Principles

1. **Never crash the shell** - Errors should be reported but should not terminate the interactive session
2. **Preserve shell state** - Variables, functions, and history should remain intact after errors
3. **Clear error messages** - Users should understand what went wrong
4. **Graceful degradation** - Even if a component fails, the shell should recover
5. **Defensive programming** - Use Result types throughout, minimize unwrap() calls

## Error Recovery Layers

### 1. Parse Error Recovery

The parser returns `Result<Vec<Statement>, Error>` and handles syntax errors gracefully:

- Invalid token sequences
- Incomplete statements
- Mismatched braces/parentheses
- Invalid variable expansions

**Behavior:**
- Parse errors are reported to the user
- The shell returns to the prompt
- Shell state is unchanged

**Example:**
```bash
aush> if x {
Error: Expected LeftBrace, found None
aush> echo still_works
still_works
aush>
```

### 2. Execution Error Recovery

The executor returns `Result<ExecutionResult, Error>` and handles runtime errors:

- Command not found
- File I/O errors
- Permission errors
- Invalid redirections
- Function call errors
- Variable expansion errors

**Behavior:**
- Execution errors are reported with context
- The shell returns to the prompt
- Shell state may be partially updated (e.g., if error occurs mid-pipeline)
- Exit code is set appropriately

**Example:**
```bash
aush> nonexistent_command
Error: Command not found: 'nonexistent_command'

Did you mean?
  existing_command (85%, builtin)
aush> echo $?
1
aush>
```

### 3. Panic Recovery

As a last resort, the interactive loop includes panic recovery using `panic::catch_unwind`:

- Catches unexpected panics from any part of the codebase
- Logs the panic message
- Returns to the prompt
- Attempts to preserve history

**Implementation:**
```rust
let result = panic::catch_unwind(AssertUnwindSafe(|| {
    execute_line(line, &mut executor)
}));

match result {
    Ok(Ok(exec_result)) => {
        // Success - print output
    }
    Ok(Err(e)) => {
        // Expected error - report it
        eprintln!("Error: {}", e);
    }
    Err(panic_info) => {
        // Unexpected panic - recover
        eprintln!("Fatal error: {}", panic_msg);
        eprintln!("Shell recovered and is ready for next command.");
    }
}
```

**Behavior:**
- Panic is caught and logged
- Shell state may be corrupted (but isolated to that execution)
- User is informed of recovery
- Shell continues accepting commands

**Example:**
```bash
aush> trigger_panic_somehow
Fatal error: attempted to divide by zero
Shell recovered and is ready for next command.
aush> echo still_alive
still_alive
aush>
```

## State Management

### Variable State

Variables are managed by the Runtime and preserved across errors:

- Successful assignments are committed immediately
- Failed assignments don't modify state
- Variable state survives parse and execution errors

### Function State

Functions are stored in the Runtime's function table:

- Successful function definitions are stored immediately
- Invalid function definitions are rejected without partial updates
- Function state survives errors in other commands

### History State

Command history is managed independently:

- Commands are added to history before execution
- History survives all types of errors
- History is periodically saved to disk
- On panic, best-effort attempt to preserve history

### Job State

Background jobs are tracked by the job manager:

- Jobs continue running even if shell encounters errors
- Job state is updated on each prompt
- Errors in job management don't crash the shell

## Error Categories

### Expected Errors (Recoverable)

These are normal operational errors that are always recoverable:

1. **Parse Errors**
   - Syntax errors
   - Incomplete statements
   - Invalid tokens

2. **Command Errors**
   - Command not found
   - Permission denied
   - File not found

3. **Runtime Errors**
   - Variable not set
   - Invalid function call
   - I/O errors

### Unexpected Errors (Panic Recovery)

These should be rare but are caught by panic recovery:

1. **Logic Errors**
   - Assertion failures
   - Array out of bounds
   - Division by zero

2. **Library Panics**
   - Panics from third-party libraries
   - Unexpected state transitions

## Testing

Error recovery is tested at multiple levels:

### Unit Tests

Individual components test error conditions:

```rust
#[test]
fn test_parse_error_recovery() {
    let tokens = Lexer::tokenize("if x { echo test").unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_err());
}
```

### Integration Tests

Full shell behavior is tested in `tests/error_recovery_tests.rs`:

- Parse error recovery
- Execution error recovery
- State preservation after errors
- Multiple consecutive errors
- Nested error recovery (subshells, pipelines)

### Manual Testing

Interactive testing scenarios:

1. Invalid syntax followed by valid command
2. Non-existent command followed by valid command
3. Invalid redirect followed by valid command
4. Pipeline with failing command
5. Conditional execution with errors

## Best Practices

### For Contributors

1. **Use Result types** - Don't panic in production code
2. **Provide context** - Use `context()` to add error context
3. **Test error paths** - Write tests for error conditions
4. **Validate inputs** - Check preconditions before operations
5. **Avoid unwrap()** - Use `?` or match instead

### For Users

1. **Check exit codes** - Use `$?` to verify command success
2. **Use conditionals** - `&&` and `||` for error handling
3. **Redirect stderr** - Capture error messages when needed
4. **Read error messages** - AUSH provides helpful error context

## Implementation Details

### Result Type Usage

Throughout the codebase:

```rust
// Lexer
pub fn tokenize(input: &str) -> Result<Vec<Token>>

// Parser
pub fn parse(&mut self) -> Result<Vec<Statement>>

// Executor
pub fn execute(&mut self, statements: Vec<Statement>) -> Result<ExecutionResult>
pub fn execute_statement(&mut self, statement: Statement) -> Result<ExecutionResult>
```

### Error Types

AUSH uses `anyhow::Result` for flexible error handling:

- Allows error context chaining
- Preserves error sources
- Enables detailed error messages

### AssertUnwindSafe

The panic recovery uses `AssertUnwindSafe` to wrap the executor:

- Allows catching panics across FFI boundaries
- Indicates that unwinding through this code is safe
- Used only in the interactive loop as a safety net

## Limitations

### What's NOT Recovered

1. **Process crashes** - Signal 9 (SIGKILL) cannot be caught
2. **Stack overflow** - May not be recoverable on all platforms
3. **Out of memory** - System may terminate the process
4. **Corrupted state** - After panic, executor state may be inconsistent

### Trade-offs

1. **Performance** - Panic recovery has minimal overhead
2. **State consistency** - After panic, state may be inconsistent (but isolated)
3. **Error messages** - Panic messages may be less informative than expected errors

## Future Enhancements

### Potential Improvements

1. **Checkpoint/Restore** - Save shell state before risky operations
2. **Isolated execution** - Run commands in separate processes
3. **Automatic recovery** - Retry operations on transient failures
4. **Error statistics** - Track common error patterns
5. **Interactive debugging** - Step through failed commands

### History Preservation

Future work could include:

- Flush history to disk on every command
- Separate history process/thread
- Signal handler to save history on crash
- Atomic history writes

## Examples

### Example 1: Parse Error

```bash
aush> let x =
Error: Expected expression
aush> let x = 42
aush> echo $x
42
```

### Example 2: Command Not Found

```bash
aush> xyz
Error: Command not found: 'xyz'
aush> echo $?
127
```

### Example 3: Pipeline Error

```bash
aush> false | echo "runs anyway"
runs anyway
aush> echo $?
0
```

### Example 4: Subshell Error

```bash
aush> echo before
before
aush> (false)
aush> echo $?
1
aush> echo after
after
```

### Example 5: Conditional Error Handling

```bash
aush> false && echo "won't print"
aush> true || echo "won't print"
aush> false || echo "will print"
will print
```

## Conclusion

AUSH shell's error recovery system ensures that the shell remains stable and usable even when errors occur. By using Result types throughout, catching panics in the interactive loop, and preserving shell state, AUSH provides a robust and reliable shell experience.

The multi-layered approach to error handling means that:
- Common errors are handled gracefully with helpful messages
- Uncommon errors are caught and logged
- Even unexpected panics don't crash the shell
- Users can continue their work uninterrupted

This makes AUSH suitable for both interactive use and scripting, where reliability is paramount.
