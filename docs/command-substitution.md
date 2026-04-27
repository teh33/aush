# Command Substitution

Command substitution allows you to capture the output of a command and use it as an argument to another command.

## Syntax

AUSH supports two syntaxes for command substitution:

### `$()` syntax (recommended)
```bash
echo $(pwd)
echo "Current directory: $(pwd)"
result=$(cat file.txt)
```

### Backtick syntax (legacy)
```bash
echo `pwd`
result=`cat file.txt`
```

The `$()` syntax is recommended because it:
- Nests more cleanly
- Is easier to read
- Follows modern shell conventions

## Features

### Basic Usage
```bash
# Capture command output
current_dir=$(pwd)
echo $current_dir

# Use as command argument
echo "You are in: $(pwd)"
```

### Nested Substitution
Command substitutions can be nested:
```bash
# Single nesting
echo $(echo $(pwd))

# Double nesting
echo $(echo $(echo "hello"))
```

### In Assignments
```bash
# Assign command output to variable
let dir = $(pwd)
let files = $(ls)
```

### In Pipelines
```bash
# Command substitution in pipes
echo $(pwd) | cat
cat $(find . -name "*.txt" | head -1)
```

### With Redirects
```bash
# Redirect output of substituted command
echo $(ls) > files.txt
```

## Behavior

### Whitespace Handling
- Trailing newlines are automatically trimmed (bash-compatible behavior)
- Internal newlines are preserved

```bash
# Output: "line1 line2" (trailing newline removed)
echo $(printf "line1\nline2\n")
```

### Exit Codes
- The exit code of the command substitution does not affect the outer command
- The outer command's exit code is what gets propagated

```bash
# Even if inner command fails, echo succeeds
echo $(false)  # exit code: 0 (from echo)
```

### Multiple Substitutions
You can use multiple command substitutions in a single command:
```bash
echo $(echo first) middle $(echo last)
# Output: first middle last
```

## Examples

### Get current directory
```bash
current=$(pwd)
echo "Working in: $current"
```

### List files with count
```bash
file_count=$(ls | wc -l)
echo "Found $file_count files"
```

### Nested git commands
```bash
# Get current branch name
branch=$(git rev-parse --abbrev-ref HEAD)
echo "On branch: $branch"
```

### Process file contents
```bash
# Read first line of file
first_line=$(head -1 config.txt)
echo "Config: $first_line"
```

### Complex nesting
```bash
# Find and read a specific file
content=$(cat $(find . -name "README.md"))
```

## Implementation Details

### Lexer
The lexer uses a custom parser function to handle nested `$()` syntax:
- Tracks depth of parentheses
- Correctly handles `$(echo $(pwd))` type patterns
- Returns complete substitution including delimiters

### Parser
The parser recognizes `CommandSubstitution` and `BacktickSubstitution` tokens and converts them to `Argument::CommandSubstitution` in the AST.

### Executor
The executor:
1. Extracts the command from the delimiters (`$()` or backticks)
2. Parses the command
3. Creates a sub-executor with the same runtime
4. Executes the command and captures stdout
5. Trims trailing newlines
6. Substitutes the result into the parent command

This happens recursively, so nested substitutions work correctly.

## Limitations

### String Interpolation
Currently, command substitutions inside double-quoted strings are not automatically expanded:

```bash
# This doesn't interpolate yet
echo "path: $(pwd)"  # Outputs: path: $(pwd)

# Workaround: use without quotes or concatenate
echo path: $(pwd)    # Works: path: /current/directory
```

## Comparison with Other Shells

| Feature | AUSH | Bash | Zsh |
|---------|------|------|-----|
| `$()` syntax | ✓ | ✓ | ✓ |
| Backticks | ✓ | ✓ | ✓ |
| Nested `$()` | ✓ | ✓ | ✓ |
| Trim trailing newlines | ✓ | ✓ | ✓ |
| In pipelines | ✓ | ✓ | ✓ |
| In redirects | ✓ | ✓ | ✓ |
| String interpolation | ✗ | ✓ | ✓ |

## Best Practices

1. **Use `$()` instead of backticks** for better readability
2. **Quote substitutions** when you want to preserve whitespace:
   ```bash
   files="$(ls)"  # Preserves newlines
   ```
3. **Check for errors** explicitly if needed:
   ```bash
   result=$(command_that_might_fail)
   # Check $? if you need to handle errors
   ```
4. **Avoid deep nesting** for maintainability
   ```bash
   # Instead of: $(cmd1 $(cmd2 $(cmd3)))
   # Do:
   let step1 = $(cmd3)
   let step2 = $(cmd2 $step1)
   let result = $(cmd1 $step2)
   ```

## Performance Considerations

- Each command substitution creates a sub-executor
- Nested substitutions execute sequentially from innermost to outermost
- Output is captured in memory (be careful with large outputs)
- No progress indicators are shown for substituted commands

## Future Enhancements

Planned improvements:
- [ ] String interpolation support (`"path: $(pwd)"` auto-expands)
- [ ] Process substitution (`<(command)` and `>(command)`)
- [ ] Array expansion from multi-line output
- [ ] Configurable whitespace handling
