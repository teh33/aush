# Signal Handling in AUSH

## Overview

AUSH implements comprehensive signal handling to provide a robust shell experience. The shell properly handles SIGINT (Ctrl-C), SIGTERM, and SIGHUP signals, ensuring clean termination and no orphaned processes.

## Supported Signals

### SIGINT (Signal 2) - Ctrl-C
- **Interactive Mode**: Returns to prompt without exiting the shell
- **Script Mode**: Terminates the script with exit code 130
- **Command Mode (-c flag)**: Terminates with exit code 130
- **During Command Execution**: Kills the running command and any child processes

### SIGTERM (Signal 15) - Termination Request
- Cleanly terminates the shell with exit code 143
- Kills all child processes before exiting
- Ensures no orphaned processes remain

### SIGHUP (Signal 1) - Hangup
- Terminates the shell with exit code 129
- Typical when terminal is closed or SSH connection drops
- Ensures clean shutdown of all processes

## Implementation Details

### Signal Handler Module (`src/signal.rs`)

The signal handler uses the `signal_hook` crate to manage Unix signals in a thread-safe manner:

```rust
use signal_hook::consts::{SIGINT, SIGTERM, SIGHUP};
use signal_hook::iterator::Signals;

pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
}
```

Key features:
- **Thread-safe**: Uses atomic operations for signal state
- **Non-blocking**: Signal checking doesn't block execution
- **Resettable**: Interactive mode can reset signal state after Ctrl-C

### Integration Points

#### Main Entry Point (`src/main.rs`)

Signal handlers are set up early in the `main()` function:

```rust
fn main() -> Result<()> {
    let signal_handler = SignalHandler::new();
    signal_handler.setup()?;

    // ... rest of initialization
}
```

The handler is then passed to all execution modes:
- `run_interactive(signal_handler)`
- `run_command(command, signal_handler)`
- `run_script(path, args, signal_handler)`

#### Executor (`src/executor/mod.rs`)

The executor checks for signals:
1. **Before each statement**: Prevents starting new work after signal
2. **During command execution**: Actively monitors long-running commands
3. **Child process cleanup**: Kills child processes when signal received

```rust
// Check for signals during command execution
loop {
    if let Some(handler) = &self.signal_handler {
        if handler.should_shutdown() {
            // Kill the child process
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow!("Command interrupted by signal"));
        }
    }

    // Check if command finished...
}
```

## Exit Codes

AUSH follows the standard Unix convention for signal exit codes:

| Signal  | Exit Code | Calculation |
|---------|-----------|-------------|
| SIGINT  | 130       | 128 + 2     |
| SIGTERM | 143       | 128 + 15    |
| SIGHUP  | 129       | 128 + 1     |

These exit codes are compatible with other Unix shells (bash, zsh, etc.).

## Behavior in Different Modes

### Interactive Mode

In interactive mode, SIGINT (Ctrl-C) has special handling:

1. If a command is running, it's interrupted
2. Control returns to the prompt
3. The shell continues running
4. Signal state is reset for the next command

Example:
```
/home/user> sleep 100
^C
/home/user> echo "Shell still running"
Shell still running
/home/user>
```

### Script Mode

When running a script file, signals cause immediate termination:

```bash
#!/usr/bin/env aush
echo "Starting..."
sleep 10
echo "This won't run if interrupted"
```

If SIGINT is received during `sleep 10`, the script exits with code 130 and the final echo never executes.

### Command Mode (-c flag)

One-off commands terminate on signal:

```bash
$ aush -c "sleep 10"
^C
$ echo $?
130
```

### Non-Interactive Input (Piped)

When reading from stdin (non-TTY), signals terminate execution:

```bash
$ echo "sleep 10" | aush
^C
Interrupted by signal
```

## Child Process Management

AUSH ensures no orphaned processes by:

1. **Tracking child processes**: All spawned commands are tracked
2. **Signal propagation**: Signals are caught and children are killed
3. **Cleanup on exit**: All children terminated before shell exits

### Process Groups

While AUSH doesn't currently use process groups, child processes are managed individually:

```rust
// When signal received during command execution
let _ = child.kill();  // Send SIGKILL to child
let _ = child.wait();  // Reap the zombie process
```

## State Management

Signal state is managed through atomic operations:

- `SIGNAL_RECEIVED`: Global flag indicating a signal was caught
- `SIGNAL_NUMBER`: The specific signal that was received
- `shutdown_flag`: Per-handler flag for shutdown requests

### Resetting State

In interactive mode, signal state is reset after Ctrl-C:

```rust
match sig {
    Ok(Signal::CtrlC) => {
        signal_handler.reset();
        continue;
    }
    // ...
}
```

## Testing

Comprehensive tests verify signal handling:

### Unit Tests (`src/signal.rs`)
- Handler creation and setup
- Exit code calculation
- State management and reset

### Integration Tests (`tests/signal_handling_tests.rs`)
- SIGINT during script execution
- SIGTERM cleanup
- SIGHUP handling
- No orphaned processes
- Command mode signal handling
- Interactive mode behavior

Run tests:
```bash
cargo test signal
```

## Error Handling

Signal-related errors are handled gracefully:

```rust
if let Err(e) = signal_handler.setup() {
    eprintln!("Warning: Failed to setup signal handlers: {}", e);
    // Continue execution - signal handling is non-critical
}
```

The shell continues to function even if signal setup fails, though signal handling won't work.

## Platform Support

Signal handling is primarily implemented for Unix-like systems (Linux, macOS, BSD):

- Uses `signal_hook` crate for portable signal handling
- Conditional compilation for Unix-specific features
- Windows support could be added using different mechanisms

## Future Enhancements

Potential improvements:

1. **Process Groups**: Use `setpgid()` for better job control
2. **SIGTSTP/SIGCONT**: Implement job suspension and resumption
3. **Signal Masks**: More sophisticated signal blocking during critical sections
4. **Custom Signal Handlers**: Allow users to define signal behavior
5. **Background Job Management**: Track and signal background jobs

## Best Practices

When writing AUSH scripts:

1. **Trap Cleanup**: Consider adding cleanup handlers (future feature)
2. **Atomic Operations**: Make critical operations atomic where possible
3. **Graceful Degradation**: Handle interrupted state appropriately
4. **Resource Cleanup**: Always close file handles and cleanup resources

## References

- [signal_hook crate documentation](https://docs.rs/signal-hook/)
- [Unix Signal Handling](https://man7.org/linux/man-pages/man7/signal.7.html)
- [POSIX Signals](https://pubs.opengroup.org/onlinepubs/9699919799/functions/V2_chap02.html#tag_15_04)
- [Shell Signal Exit Codes](https://tldp.org/LDP/abs/html/exitcodes.html)

## Troubleshooting

### Signals Not Working

If signals aren't being handled properly:

1. Check that `signal_handler.setup()` succeeded
2. Verify the process is running in a terminal (for interactive mode)
3. Check system signal delivery with `kill -l`

### Orphaned Processes

If processes are being orphaned:

1. Verify signal handler is being passed to executor
2. Check that `child.kill()` is being called
3. Use `ps` to inspect process tree

### Exit Codes

If exit codes are incorrect:

1. Verify signal number is being captured correctly
2. Check calculation: exit_code = 128 + signal_number
3. Ensure signal state isn't being reset prematurely

## Summary

AUSH's signal handling provides:
- ✅ Proper SIGINT, SIGTERM, and SIGHUP handling
- ✅ No orphaned child processes
- ✅ Correct signal exit codes (130 for SIGINT, etc.)
- ✅ Clean state management and resource cleanup
- ✅ Different behavior for interactive vs script mode
- ✅ Comprehensive test coverage
