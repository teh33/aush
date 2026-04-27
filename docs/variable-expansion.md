# Variable Expansion in AUSH

AUSH supports comprehensive variable expansion using the `${...}` syntax, compatible with POSIX shell standards.

## Overview

Variable expansion allows you to manipulate variables at the time they are used, providing default values, error handling, and string transformations.

## Syntax Reference

### Simple Expansion

```bash
${VAR}
```

Expands to the value of `VAR`. If `VAR` is unset, expands to an empty string.

**Example:**
```bash
let NAME = "World"
echo ${NAME}  # Outputs: World
```

### Use Default Value

```bash
${VAR:-default}
```

If `VAR` is unset, use `default` instead. Does not modify `VAR`.

**Examples:**
```bash
# VAR is unset
echo ${VAR:-hello}  # Outputs: hello
# VAR remains unset

# VAR is set
let VAR = "world"
echo ${VAR:-hello}  # Outputs: world
```

### Assign Default Value

```bash
${VAR:=default}
```

If `VAR` is unset, assign `default` to it and return `default`.

**Examples:**
```bash
# VAR is unset
echo ${VAR:=hello}  # Outputs: hello
# VAR is now set to "hello"

# VAR is already set
let VAR = "world"
echo ${VAR:=hello}  # Outputs: world
# VAR remains "world"
```

### Error If Unset

```bash
${VAR:?error_message}
```

If `VAR` is unset, display `error_message` and exit with an error.

**Examples:**
```bash
# VAR is set
let VAR = "value"
echo ${VAR:?not set}  # Outputs: value

# VAR is unset
echo ${MISSING:?variable is required}
# Error: MISSING: variable is required
```

## Pattern Matching

### Remove Shortest Prefix Match

```bash
${VAR#pattern}
```

Remove the shortest match of `pattern` from the beginning of `VAR`.

**Examples:**
```bash
let PATH = "/usr/local/bin"
echo ${PATH#/usr/}  # Outputs: local/bin

let FILE = "prefix_middle_suffix"
echo ${FILE#prefix_}  # Outputs: middle_suffix

# With glob patterns
let VAR = "hello_world_test"
echo ${VAR#hello_*}  # Outputs: world_test (removes up to first match)
```

### Remove Longest Prefix Match

```bash
${VAR##pattern}
```

Remove the longest match of `pattern` from the beginning of `VAR`.

**Examples:**
```bash
let PATH = "/usr/local/bin"
echo ${PATH##/*/}  # Outputs: bin

let VAR = "foo/bar/foo/baz"
echo ${VAR##foo/}  # Outputs: baz (removes up to last "foo/")

# With glob patterns
let VAR = "prefix_one_prefix_two"
echo ${VAR##prefix_*}  # Outputs: two (removes up to last match)
```

### Remove Shortest Suffix Match

```bash
${VAR%pattern}
```

Remove the shortest match of `pattern` from the end of `VAR`.

**Examples:**
```bash
let FILE = "document.tar.gz"
echo ${FILE%.gz}  # Outputs: document.tar

let FILE = "test.backup.txt"
echo ${FILE%.txt}  # Outputs: test.backup

# With glob patterns
let VAR = "test_hello_world"
echo ${VAR%_*}  # Outputs: test_hello (removes from last "_" to end)
```

### Remove Longest Suffix Match

```bash
${VAR%%pattern}
```

Remove the longest match of `pattern` from the end of `VAR`.

**Examples:**
```bash
let FILE = "document.tar.gz"
echo ${FILE%%.tar*}  # Outputs: document

let FILE = "backup.2023.12.31.tar.gz"
echo ${FILE%%.*}  # Outputs: backup (removes from first "." to end)

# With glob patterns
let VAR = "test_hello_world"
echo ${VAR%%_*}  # Outputs: test (removes from first "_" to end)
```

## Pattern Matching with Wildcards

The pattern matching operators (`#`, `##`, `%`, `%%`) support simple glob patterns:

- `*` - Matches any sequence of characters

**Examples:**
```bash
let PATH = "/home/user/documents/file.txt"

# Remove everything up to last slash
echo ${PATH##*/}  # Outputs: file.txt

# Remove extension
echo ${PATH%.*}  # Outputs: /home/user/documents/file

# Get directory path
let FULL = "/home/user/file.txt"
echo ${FULL%/*}  # Outputs: /home/user
```

## Common Use Cases

### Working with File Paths

```bash
let FILE = "/home/user/documents/report.pdf"

# Get filename
echo ${FILE##*/}  # Outputs: report.pdf

# Get directory
echo ${FILE%/*}  # Outputs: /home/user/documents

# Get basename (remove extension)
let BASENAME = ${FILE##*/}
echo ${BASENAME%.*}  # Outputs: report

# Get extension
echo ${FILE##*.}  # Outputs: pdf
```

### Providing Defaults

```bash
# Use environment variable or default
let PORT = ${PORT:-8080}
echo "Server running on port ${PORT}"

# Require configuration variable
let CONFIG = ${CONFIG_PATH:?Configuration path must be set}
```

### String Manipulation

```bash
let URL = "https://example.com/path/to/resource"

# Remove protocol
echo ${URL#https://}  # Outputs: example.com/path/to/resource

# Get domain
let DOMAIN_PATH = ${URL#https://}
echo ${DOMAIN_PATH%%/*}  # Outputs: example.com
```

### Multiple Transformations

```bash
let FILE = "/usr/local/bin/program.sh"

# Chain operations by assigning to intermediate variables
let BASENAME = ${FILE##*/}      # program.sh
let NAME = ${BASENAME%.sh}      # program

echo "Program name: ${NAME}"
```

## Differences from Regular Variables

Regular `$VAR` syntax:
- Simple variable substitution only
- No operators or transformations
- Shorter syntax for simple cases

Braced `${VAR}` syntax:
- Supports all expansion operators
- Required for transformations
- Clearer in complex expressions
- Allows adjacent text: `${VAR}suffix`

Both syntaxes work for simple variable expansion:
```bash
let NAME = "World"
echo $NAME       # Outputs: World
echo ${NAME}     # Outputs: World
```

## Implementation Notes

### Pattern Matching Algorithm

- Prefix matching (`#`, `##`): Searches from the start of the string
  - `#`: Finds the first occurrence and removes up to that point
  - `##`: Finds the last occurrence and removes up to that point

- Suffix matching (`%`, `%%`): Searches from the end of the string
  - `%`: Finds the last occurrence and removes from that point to the end
  - `%%`: Finds the first occurrence and removes from that point to the end

### Nested Expansions

Currently, AUSH does not support nested variable expansions like `${VAR:-${DEFAULT}}`. Use intermediate variables instead:

```bash
let DEFAULT = "fallback"
let RESULT = ${VAR:-$DEFAULT}
```

### Compatibility

AUSH's variable expansion is designed to be compatible with POSIX shell standards and bash, making it easy to port existing shell scripts.

## Error Handling

Variable expansion errors (from `:?` operator) will:
1. Display the error message to stderr
2. Return a non-zero exit code
3. Halt execution of the current command

Example:
```bash
echo ${REQUIRED:?must be set}  # Error if REQUIRED is unset
```

## Performance Considerations

- Simple expansions (`${VAR}`) are optimized for speed
- Pattern matching operations are efficient for typical use cases
- Complex pattern matching with wildcards may be slower on very long strings
