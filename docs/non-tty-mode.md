# Non-TTY Mode Support

## Overview

AUSH shell now supports both TTY (interactive) and non-TTY (piped input) modes, making it suitable as a login shell and for scripting scenarios.

## Implementation

### TTY Detection

The shell uses the `atty` crate to detect whether stdin is a TTY:

```rust
fn run_interactive() -> Result<()> {
    if atty::is(atty::Stream::Stdin) {
        run_interactive_with_reedline()
    } else {
        run_non_interactive()
    }
}
```

### Interactive Mode (TTY)

When running in a terminal (TTY mode), AUSH uses reedline for:
- Line editing
- Tab completion
- History
- Syntax highlighting
- Multi-line input

### Non-Interactive Mode (Non-TTY)

When stdin is piped (non-TTY mode), AUSH:
- Reads from stdin line by line
- Executes each line immediately
- Prints output as it's generated
- Skips empty lines and comments (lines starting with `#`)
- Continues execution on errors (doesn't exit)
- Exits cleanly when stdin closes (EOF)

## Usage Examples

### Piped Input
```bash
echo "echo hello" | aush
```

### Multiple Commands
```bash
printf "echo line1\necho line2\n" | aush
```

### Script Execution
```bash
cat script.txt | aush
```

### Login Shell
```bash
# In /etc/shells, add aush path
# Then use chsh to change shell
chsh -s /path/to/aush
```

### Process Substitution
```bash
aush < <(echo "echo test")
```

## Error Handling

In non-interactive mode, errors are reported to stderr but execution continues:
```bash
echo "echo ok" > test.txt
echo "invalid_command" >> test.txt
echo "echo still works" >> test.txt
cat test.txt | aush
# Outputs:
# ok
# Error: invalid_command
# still works
```

## Implementation Details

### Dependencies
- `atty = "0.2"` - TTY detection

### Modified Files
- `Cargo.toml` - Added atty dependency
- `src/main.rs` - Split run_interactive into two paths:
  - `run_interactive_with_reedline()` - Original TTY mode
  - `run_non_interactive()` - New non-TTY mode

### Code Structure
```rust
fn run_non_interactive() -> Result<()> {
    let mut executor = Executor::new();
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match execute_line(line, &mut executor) {
            Ok(result) => {
                if !result.stdout.is_empty() {
                    print!("{}", result.stdout);
                }
                if !result.stderr.is_empty() {
                    eprint!("{}", result.stderr);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                // Continue on error in non-interactive mode
            }
        }
    }

    Ok(())
}
```

## Testing

The implementation has been tested with:
- Single command piped input
- Multiple commands
- Comments and empty lines
- Error handling (continues on error)
- EOF handling

All tests pass successfully.
