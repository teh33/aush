# Tab Completion System

The AUSH shell features an advanced tab completion system that provides context-aware suggestions for commands, paths, flags, and special contexts like git branches, cargo commands, and npm scripts.

## Features

### 1. Command Name Completion

The completion system suggests command names from three sources:

- **Built-in Commands**: Core shell commands like `cd`, `pwd`, `echo`, `ls`, `grep`, `find`, `cat`, etc.
- **PATH Executables**: All executable files in directories listed in the `$PATH` environment variable
- **User-Defined Functions**: Functions defined in the current shell session

Example:
```bash
$ ec<TAB>
echo  # completes to built-in echo command
```

### 2. Path Completion

The system provides intelligent path completion with:

- File and directory suggestions
- `.gitignore` support (respects git ignore rules)
- Tilde (`~`) expansion for home directory
- Support for both relative and absolute paths
- Automatic trailing slash for directories

Example:
```bash
$ cat src/com<TAB>
src/completion/  # suggests completion directory with trailing slash
```

The path completion uses the `ignore` crate to respect `.gitignore` patterns, ensuring that ignored files and directories don't clutter your suggestions.

### 3. Context-Aware Completion

The system recognizes the context of your command and provides relevant suggestions:

#### Git Commands

- **Branch Names**: After `git checkout`, `git merge`, `git branch`, or `git rebase`
- **Git Subcommands**: After the `git` command itself

Examples:
```bash
$ git chec<TAB>
checkout  # suggests git subcommand

$ git checkout mai<TAB>
main  # suggests branch name from current repository
```

#### Cargo Commands

After typing `cargo`, the system suggests common cargo subcommands:

```bash
$ cargo bu<TAB>
build  # suggests cargo build command
```

Supported commands: build, check, clean, doc, test, bench, run, publish, install, update, search, add, remove

#### NPM Scripts

After typing `npm run`, the system reads your `package.json` and suggests available scripts:

```bash
$ npm run te<TAB>
test  # suggests test script from package.json
```

#### Rust File Completion

For Rust-specific commands like `rustc` and `rustdoc`, the completion filters to only show `.rs` files and directories:

```bash
$ rustc src/m<TAB>
src/main.rs  src/mod.rs  # only suggests .rs files
```

### 4. Flag Completion

Common flags are suggested for built-in commands:

#### ls flags
```bash
$ ls -<TAB>
-l  -a  -h  -R  -t  -r  --long  --all  --human-readable
```

#### grep flags
```bash
$ grep -<TAB>
-i  -r  -n  -v  -w  -E  --ignore-case  --recursive  --line-number  --invert-match
```

#### find flags
```bash
$ find -<TAB>
-name  -type  -size  -mtime  -exec  -print
```

#### cat flags
```bash
$ cat -<TAB>
-n  -b  -s  --number  --number-nonblank
```

### 5. Caching

The completion system implements smart caching for expensive operations:

- **PATH Scanning**: Cached for 5 minutes to avoid repeated filesystem scans
- **Git Branches**: Cached for 5 minutes to avoid repeated git operations

Cache entries automatically expire after their TTL (Time To Live) and are refreshed on the next access.

## Architecture

### Core Components

#### `Completer` Struct

The main completion engine that:
- Holds references to builtins and runtime
- Manages caches for PATH executables and git branches
- Implements the `reedline::Completer` trait

#### `CompletionContext` Enum

Defines different completion contexts:
- `Command`: Completing a command name
- `Path`: Completing a file or directory path
- `GitBranch`: Completing a git branch name
- `GitSubcommand`: Completing a git subcommand
- `CargoCommand`: Completing a cargo subcommand
- `NpmScript`: Completing an npm script
- `RustFile`: Completing a .rs file
- `Flag(String)`: Completing flags for a specific command

#### `CacheEntry<T>`

A generic cache entry with:
- Cached data of type `T`
- Timestamp for expiration checking
- TTL-based validity checking

### Integration

The completion system integrates with AUSH through:

1. **Reedline Integration**: Implements the `reedline::Completer` trait
2. **Shared State**: Uses `Arc<RwLock<Runtime>>` to share state with the executor
3. **Builtin Access**: References the `Builtins` struct for command information

## Usage in Code

Creating a completer:

```rust
use std::sync::{Arc, RwLock};
use aush::completion::Completer;
use aush::builtins::Builtins;
use aush::runtime::Runtime;

let builtins = Arc::new(Builtins::new());
let runtime = Arc::new(RwLock::new(Runtime::new()));
let completer = Box::new(Completer::new(builtins, runtime));

let mut line_editor = Reedline::create()
    .with_completer(completer);
```

## Performance Considerations

### PATH Scanning

Scanning the PATH can be expensive on systems with many directories. The completion system:
- Caches results for 5 minutes
- Only scans executable files (checks file permissions on Unix)
- Uses a `HashSet` to deduplicate entries

### Git Branch Scanning

Reading git branches requires git2 operations. The completion system:
- Caches branch lists for 5 minutes
- Only scans when in a git repository
- Returns empty list if not in a git repository

### Path Traversal

Path completion uses the `ignore` crate which:
- Efficiently walks directories
- Respects `.gitignore` patterns
- Only traverses one level deep (max_depth=1)
- Shows hidden files but respects git ignore rules

## Testing

The completion system includes comprehensive tests covering:

1. **Command Completion**: Tests builtin command suggestions
2. **Path Completion**: Tests file and directory suggestions
3. **Context Detection**: Tests for git, cargo, npm, and rust contexts
4. **Flag Completion**: Tests flag suggestions for various commands
5. **Git Branch Context**: Tests git branch completion after checkout/merge
6. **Cargo Context**: Tests cargo subcommand completion
7. **NPM Context**: Tests npm script completion
8. **Cache Behavior**: Tests cache population and reuse

Run tests with:
```bash
cargo test --lib completion
```

## Future Enhancements

Potential improvements for the completion system:

1. **Smart Suggestions**: Learn from command history to prioritize frequent commands
2. **Fuzzy Matching**: Support fuzzy matching instead of just prefix matching
3. **Context from --help**: Parse `--help` output to dynamically learn flags
4. **Remote Branch Completion**: Complete remote git branches
5. **File Type Detection**: Context-aware file completion based on command
6. **Environment Variable Completion**: Complete `$VAR` names
7. **Custom Completions**: Allow users to define custom completion rules
8. **Completion Descriptions**: Add descriptions to suggestions
9. **Multi-word Completion**: Handle quoted strings with spaces
10. **SSH Host Completion**: Complete hostnames from ~/.ssh/config

## Implementation Notes

### Thread Safety

The completer uses `Arc<RwLock<T>>` for shared access to runtime state:
- Multiple readers can access cached data simultaneously
- Writers (cache updates) block readers temporarily
- No deadlocks due to careful lock scoping

### Error Handling

The completion system is designed to never fail:
- Returns empty suggestions on errors
- Handles missing files gracefully
- Works even if git/cargo/npm are not available

### Gitignore Support

Uses the `ignore` crate (from ripgrep) which:
- Supports all gitignore patterns
- Respects global git ignore
- Respects git exclude files
- Fast and well-tested implementation
