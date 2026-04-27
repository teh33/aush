# Glob Expansion (Wildcard Expansion)

AUSH implements full glob (wildcard) expansion support for file matching, similar to bash and other Unix shells.

## Features

### Basic Wildcards

#### Asterisk (`*`)
Matches zero or more characters.

```bash
# Match all .txt files in current directory
cat *.txt

# Match all files starting with 'test'
ls test*

# Match all files with any extension
echo *.*
```

#### Question Mark (`?`)
Matches exactly one character.

```bash
# Match file1.txt, file2.txt, but not file10.txt
cat file?.txt

# Match single-character filenames
ls ?
```

#### Character Classes (`[...]`)
Matches any one of the enclosed characters.

```bash
# Match file1.txt, file2.txt, file3.txt only
cat file[123].txt

# Match files with vowels
ls [aeiou]*

# Character ranges
cat file[1-5].txt
cat [a-z]*.txt
```

### Advanced Patterns

#### Recursive Glob (`**`)
Matches directories recursively.

```bash
# Find all .rs files in current directory and subdirectories
cat **/*.rs

# Match files at any depth
grep "pattern" **/*.txt
```

## Dotfile Handling

By default, glob patterns do **not** match dotfiles (files starting with `.`), following Unix conventions.

```bash
# Does NOT match .hidden or .config
ls *

# Explicitly match dotfiles
ls .*

# Match all files including dotfiles in subdirs
ls .*/*.txt
```

## Empty Glob Results

When a glob pattern matches no files, AUSH returns an error instead of treating it as a literal string.

```bash
# If no .xyz files exist, this returns an error
cat *.xyz
# Error: No matches found for pattern: *.xyz

# This is different from literal arguments
cat literally.xyz
# Tries to open "literally.xyz" (may fail if file doesn't exist, but for a different reason)
```

## Multiple Patterns

You can use multiple glob patterns in a single command.

```bash
# Concatenate all .txt and .md files
cat *.txt *.md

# Mix globs with literal filenames
cat *.txt specific_file.log
```

## Pattern Expansion Order

1. Variable substitution is performed first
2. Glob patterns are then expanded
3. Results are sorted alphabetically
4. Command is executed with expanded arguments

```bash
let pattern = "*.txt"
cat $pattern  # Variable expanded first, then glob pattern

# Results are always sorted
ls zebra.txt apple.txt  # Printed as: apple.txt zebra.txt (when globbed)
```

## Implementation Details

### Module: `src/glob_expansion/mod.rs`

The glob expansion module provides:

- `expand_globs(pattern: &str, cwd: &Path) -> Result<Vec<String>>` - Expand a single pattern
- `expand_multiple_globs(patterns: &[String], cwd: &Path) -> Result<Vec<String>>` - Expand multiple patterns
- `should_expand_glob(arg: &str) -> bool` - Check if a string contains glob metacharacters

### Integration

Glob expansion is integrated into the executor:
- Happens automatically for all command arguments
- Works with both builtin and external commands
- Supports parallel execution (`|||`)
- Works in pipelines

### Configuration Options

The glob expansion uses the following match options (from the `glob` crate):

```rust
MatchOptions {
    case_sensitive: true,                  // Case-sensitive matching
    require_literal_separator: false,      // * can match /
    require_literal_leading_dot: true,     // Dotfiles not matched by default
}
```

## Examples

### Basic Usage

```bash
# List all Rust source files
ls *.rs

# Count lines in all text files
cat *.txt | wc -l

# Copy all PDFs to backup directory
cp *.pdf ~/backup/
```

### Character Classes

```bash
# Match files with digits
ls file[0-9].txt

# Match specific characters
ls [abc]*.rs

# Exclude pattern (use character class negation)
ls [!.]* # Match all non-dotfiles (alternative to *)
```

### Recursive Patterns

```bash
# Find all config files in project
cat **/*.config

# Search all Rust files recursively
grep "TODO" **/*.rs

# Count all source files
ls **/*.{rs,toml} | wc -l
```

### Combining Patterns

```bash
# Multiple patterns
cat *.txt *.md *.rst

# Mix with literal paths
cat README.md *.txt docs/*.md

# Use in complex commands
for file in *.log {
    grep "ERROR" $file
}
```

## Edge Cases

### No Matches
```bash
cat *.nonexistent
# Error: No matches found for pattern: *.nonexistent
```

### Single Match
```bash
# If only one.txt exists
cat *.txt
# Equivalent to: cat one.txt
```

### Special Characters in Filenames
```bash
# Glob metacharacters in actual filenames need escaping
# or quoting (when we implement escaping)
cat "file*.txt"  # Literal file named "file*.txt"
```

## Differences from Bash

AUSH glob expansion aims for bash compatibility, but with some enhancements:

1. **Better Error Messages**: Clear error when no matches found
2. **Consistent Sorting**: Always alphabetically sorted
3. **Modern Patterns**: Full support for `**` recursive glob

## Performance

The glob expansion module is optimized for performance:
- Uses the battle-tested `glob` crate
- Results are cached per pattern
- Minimal overhead for non-glob arguments
- Parallel-safe for concurrent execution

## Testing

Comprehensive tests are located in `tests/glob_expansion_tests.rs`:
- Basic wildcard patterns (`*`, `?`, `[...]`)
- Recursive glob (`**`)
- Dotfile handling
- Empty result error handling
- Multiple patterns
- Integration with builtins and external commands
- Sorted output verification

## Future Enhancements

Potential future improvements:
- Brace expansion: `{a,b,c}` and `{1..10}`
- Extended glob patterns: `@(pattern)`, `+(pattern)`, etc.
- Glob options: `nullglob`, `dotglob`, `nocaseglob`
- Tilde expansion: `~/docs/*.txt`

## See Also

- [Command History](command-history.md)
- [Tab Completion](tab-completion.md)
- [Context Detection](context-detection.md)
