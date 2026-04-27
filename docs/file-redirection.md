# File Redirection in AUSH

## Overview

AUSH supports comprehensive file redirection, allowing you to control where command input comes from and where output goes. This is a fundamental feature for shell scripting and command-line operations.

## Supported Redirect Operators

### Standard Output Redirect (`>`)

Redirects stdout to a file, overwriting if the file exists.

```bash
echo "Hello World" > output.txt
ls -l > directory_listing.txt
```

**Behavior:**
- Creates the file if it doesn't exist
- Overwrites the file if it exists
- Only redirects stdout (not stderr)

### Append Redirect (`>>`)

Appends stdout to a file instead of overwriting.

```bash
echo "First line" > log.txt
echo "Second line" >> log.txt
echo "Third line" >> log.txt
```

**Behavior:**
- Creates the file if it doesn't exist
- Appends to the end of the file if it exists
- Only redirects stdout (not stderr)

### Standard Input Redirect (`<`)

Redirects a file's contents to stdin of a command.

```bash
cat < input.txt
wc -l < data.txt
grep "pattern" < file.txt
```

**Behavior:**
- Reads from the specified file
- Fails if the file doesn't exist
- Passes file contents as stdin to the command

### Standard Error Redirect (`2>`)

Redirects stderr to a file.

```bash
ls nonexistent 2> errors.log
command_that_fails 2> error_output.txt
```

**Behavior:**
- Creates the file if it doesn't exist
- Overwrites the file if it exists
- Only redirects stderr (not stdout)
- Useful for separating error messages from normal output

### Redirect Both (`&>`)

Redirects both stdout and stderr to the same file.

```bash
command &> all_output.log
./script.sh &> complete_log.txt
```

**Behavior:**
- Creates the file if it doesn't exist
- Overwrites the file if it exists
- Captures both standard output and error output
- Equivalent to `> file 2>&1` in bash

### Stderr to Stdout (`2>&1`)

Redirects stderr to wherever stdout is currently going.

```bash
command 2>&1 | grep error
command > output.txt 2>&1
```

**Behavior:**
- Merges stderr into stdout stream
- Useful with pipes to process all output together
- Order matters: `> file 2>&1` redirects both to file, but `2>&1 > file` only redirects stdout to file

## Usage Examples

### Basic File Output

```bash
# Write command output to file
echo "Configuration data" > config.txt

# Append to existing file
date >> activity.log
```

### Error Handling

```bash
# Capture errors separately
make build 2> build_errors.log

# Combine stdout and stderr
make build &> build_output.log

# Send errors to stdout for processing
./script.sh 2>&1 | grep -i error
```

### Using Redirects with Pipes

```bash
# Pipeline with output redirect
cat data.txt | grep pattern | sort > results.txt

# Pipeline with error redirect
command1 | command2 2> errors.log | command3

# Capture everything from pipeline
pipeline_command1 | pipeline_command2 2>&1 | tee full_log.txt
```

### Input Redirection

```bash
# Read from file
sort < unsorted.txt

# Combine input and output redirection
sort < input.txt > sorted.txt

# Multiple redirects
command < input.txt > output.txt 2> errors.txt
```

### Advanced Patterns

```bash
# Separate stdout and stderr
command > output.log 2> error.log

# Discard stderr (redirect to /dev/null)
noisy_command 2> /dev/null

# Append both streams
command >> combined.log 2>&1

# Chain multiple operations
echo "data" | process1 > intermediate.txt
process2 < intermediate.txt > final.txt
```

## Implementation Details

### AST Representation

Redirects are represented in the AST as:

```rust
pub struct Redirect {
    pub kind: RedirectKind,
    pub target: Option<String>, // None for special cases like 2>&1
}

pub enum RedirectKind {
    Stdout,          // >
    StdoutAppend,    // >>
    Stdin,           // <
    Stderr,          // 2>
    StderrToStdout,  // 2>&1
    Both,            // &>
}
```

### Execution

During command execution:

1. **Lexer** tokenizes redirect operators
2. **Parser** attaches redirects to command nodes
3. **Executor** sets up file descriptors before spawning the process:
   - Opens files with appropriate modes (read/write/append)
   - Configures process stdin/stdout/stderr using `std::process::Stdio`
   - Handles special cases like `2>&1` by merging streams

### File Descriptor Setup

```rust
// Example: stdout redirect
RedirectKind::Stdout => {
    let file = File::create(target)?;
    cmd.stdout(Stdio::from(file));
}

// Example: append
RedirectKind::StdoutAppend => {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(target)?;
    cmd.stdout(Stdio::from(file));
}

// Example: stdin
RedirectKind::Stdin => {
    let file = File::open(target)?;
    cmd.stdin(Stdio::from(file));
}
```

## Error Handling

AUSH provides helpful error messages for redirection failures:

- **File not found** (for stdin): Clear error indicating which file couldn't be opened
- **Permission denied**: Indicates lack of read/write permissions
- **Directory doesn't exist**: Fails gracefully when redirecting to non-existent directory
- **Invalid file path**: Reports malformed paths

## Compatibility

AUSH's redirection behavior is designed to match POSIX shell standards:

- `>` overwrites files (like bash)
- `>>` appends to files
- `2>&1` redirects stderr to current stdout location
- `&>` is a bash-style shortcut for `>file 2>&1`
- Order of redirects matters for proper behavior

## Limitations and Future Work

Current implementation:

- ✅ All basic redirect operators
- ✅ Works with pipes
- ✅ Multiple redirects per command
- ✅ File creation/overwrite/append
- ✅ Error handling

Potential future enhancements:

- File descriptor numbers (e.g., `3>`, `4<`)
- Here documents (`<<EOF`)
- Here strings (`<<<`)
- Process substitution (`<(command)`, `>(command)`)
- Append stderr (`2>>`)

## Testing

Comprehensive tests cover:

- Basic stdout/stderr/stdin redirection
- Append vs overwrite behavior
- Multiple redirects on single command
- Redirects combined with pipes
- Error cases (missing files, permissions)
- Creating files vs overwriting existing files

See `tests/redirect_tests.rs` for the full test suite.
