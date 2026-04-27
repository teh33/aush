# Subshell Support in AUSH

Subshells provide a way to execute commands in an isolated environment where changes to variables and the current directory do not affect the parent shell.

## Syntax

Subshells are created by enclosing commands in parentheses:

```bash
(command1 && command2)
```

## Features

### Variable Isolation

Variables set or modified within a subshell do not affect the parent shell's environment:

```bash
let x = parent
(let x = child)
echo $x  # Outputs: parent
```

The subshell inherits the parent's variables but modifications are isolated:

```bash
let greeting = hello
(echo $greeting)  # Outputs: hello
(let greeting = goodbye)
echo $greeting    # Outputs: hello (unchanged)
```

### Current Directory Isolation

Changes to the current directory within a subshell do not affect the parent shell:

```bash
pwd          # /home/user
(cd /tmp && pwd)  # /tmp
pwd          # /home/user (unchanged)
```

This is particularly useful when you need to temporarily work in a different directory:

```bash
(cd /var/log && grep ERROR syslog)
# Still in original directory after the command completes
```

### Exit Code Propagation

The exit code of a subshell is the exit code of the last command executed within it:

```bash
(echo hello && false)  # Exit code: 1 (from false)
(true && echo world)   # Exit code: 0 (from echo)
```

This allows proper error handling with `&&` and `||` operators:

```bash
(cd /tmp && make build) && echo "Build succeeded"
```

### Nested Subshells

Subshells can be nested to create multiple levels of isolation:

```bash
let x = level0
(let x = level1 && (let x = level2))
echo $x  # Outputs: level0
```

Each level maintains its own isolated environment:

```bash
((echo nested))  # Works fine
```

### Multiple Statements

Subshells can contain multiple statements separated by `&&`, `||`, or semicolons:

```bash
(
    cd /tmp
    echo "Working in: $(pwd)"
    ls -la
)
```

Or on a single line:

```bash
(cd /tmp && ls -la && echo done)
```

## Use Cases

### Temporary Environment Changes

Run commands with temporary directory or variable changes:

```bash
(export DEBUG=1 && run-tests)
# DEBUG is not set in parent shell
```

### Grouping Commands

Group related commands that should execute in isolation:

```bash
(
    cd build
    cmake ..
    make
) && echo "Build completed"
```

### Clean Script Organization

Keep your script organized by isolating side effects:

```bash
# Save report in a different directory without changing parent's cwd
(cd /var/reports && generate-report > monthly.txt)

# Continue working in original directory
process-data
```

### Testing and Experimentation

Safely test commands without affecting your environment:

```bash
(rm -rf build && make clean && make test)
# build directory only removed in subshell
```

## Implementation Details

### How It Works

1. When a subshell is encountered, the parser creates a `Statement::Subshell` variant containing the statements to execute
2. The executor clones the current runtime environment (variables, current directory, etc.)
3. A new executor is created with the cloned runtime
4. All statements in the subshell execute in this isolated environment
5. Only the execution result (stdout, stderr, exit code) is returned to the parent
6. All runtime changes in the subshell are discarded

### Performance Considerations

- Subshells clone the runtime environment, which has a small overhead
- For most use cases, this overhead is negligible
- Nested subshells multiply the cloning overhead
- Consider using regular blocks if isolation is not needed

## Differences from Other Shells

### Bash Compatibility

AUSH subshells work similarly to Bash subshells:

- Variable isolation works the same way
- Exit code propagation is identical
- Directory isolation behaves the same

### Current Limitations

1. **Pipelines with Subshells**: While basic subshells work, using subshells within complex pipelines may have limitations
2. **Background Jobs**: Subshells do not currently support background execution (`&`)
3. **Process Substitution**: `<()` syntax is not yet supported

## Examples

### Example 1: Safe Directory Navigation

```bash
# Process files in multiple directories without changing cwd
(cd /var/log && process-logs)
(cd /etc && backup-configs)
echo "Still in: $(pwd)"  # Original directory
```

### Example 2: Temporary Variables

```bash
let API_URL = production-url

# Use different URL for testing
(let API_URL = test-url && run-integration-tests)

# API_URL still points to production
deploy-to $API_URL
```

### Example 3: Complex Build Process

```bash
# Build in a clean environment
(
    cd build
    rm -rf *
    cmake ..
    make -j4
) && echo "Build successful" || echo "Build failed"
```

### Example 4: Nested Isolation

```bash
let env = production

(
    let env = staging
    echo "Outer subshell: $env"  # staging

    (
        let env = development
        echo "Inner subshell: $env"  # development
    )

    echo "Back to outer: $env"  # staging
)

echo "Parent shell: $env"  # production
```

## See Also

- [Functions](./functions.md) - User-defined functions with scope
- [Variables](./variables.md) - Variable assignment and expansion
- [Command History](./command-history.md) - History tracking
