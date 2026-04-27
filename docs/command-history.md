# Command History System

This document describes the comprehensive command history system implemented for the AUSH shell.

## Overview

The AUSH shell includes a powerful command history system with the following features:

- **Persistent storage** at `~/.aush_history`
- **Fuzzy search** using the SkimMatcherV2 algorithm
- **Deduplication** (consecutive and optionally all duplicates)
- **Timestamp tracking** for each command
- **Ignore patterns** for sensitive or irrelevant commands
- **Configurable settings** for history size and behavior

## Architecture

### Core Components

#### `HistoryEntry`
```rust
pub struct HistoryEntry {
    pub command: String,
    pub timestamp: DateTime<Utc>,
}
```
Represents a single command with its execution timestamp.

#### `HistoryConfig`
```rust
pub struct HistoryConfig {
    pub max_size: usize,              // Default: 10,000
    pub deduplicate_all: bool,         // Default: false
    pub show_timestamps: bool,         // Default: false
    pub ignore_patterns: Vec<String>,  // Default: []
    pub ignore_space: bool,            // Default: true
}
```
Configures history behavior.

#### `History`
The main history manager that handles:
- Loading and saving to disk
- Adding commands with deduplication
- Searching with fuzzy matching
- Managing history size

### File Format

History is stored as newline-delimited JSON for flexibility and forward compatibility:

```json
{"command":"echo hello","timestamp":"2024-01-20T12:34:56.789Z"}
{"command":"ls -la","timestamp":"2024-01-20T12:35:10.123Z"}
```

The system supports backward compatibility with plain text history files.

## Features

### 1. Persistent History

History is automatically:
- Loaded on shell startup
- Appended after each command execution
- Saved to `~/.aush_history`

```rust
let mut history = History::new();
history.load()?;  // Load from ~/.aush_history
history.add("git status".to_string())?;  // Auto-appends to file
```

### 2. Fuzzy Search

Uses the SkimMatcherV2 algorithm for intelligent fuzzy matching:

```rust
// Search for commands containing "git"
let results = history.search("git", 20);
for result in results {
    println!("[score: {}] {}", result.score, result.entry.command);
}
```

Results are ranked by relevance score, with exact matches scoring higher.

### 3. Deduplication

#### Consecutive Duplicates (Always Enabled)
```bash
$ ls
$ ls  # Won't be saved
$ pwd
$ ls  # Will be saved (not consecutive)
```

#### Full Deduplication (Optional)
```rust
let mut config = HistoryConfig::default();
config.deduplicate_all = true;
history.set_config(config);
```

With full deduplication, duplicate commands are removed and moved to the end of history.

### 4. Timestamp Tracking

Each command is timestamped when added:

```rust
let entry = history.get(0).unwrap();
println!("Executed at: {}", entry.timestamp);
```

### 5. Ignore Patterns

#### Space-Prefixed Commands
Commands starting with a space are ignored by default (bash HISTIGNORE behavior):

```bash
$  secret password  # Won't be saved
$ echo hello        # Will be saved
```

#### Custom Patterns
```rust
let mut config = HistoryConfig::default();
config.ignore_patterns = vec![
    "history".to_string(),
    "exit".to_string(),
];
history.set_config(config);
```

### 6. Size Management

History is automatically trimmed to the configured max size (default: 10,000 entries):

```rust
let mut config = HistoryConfig::default();
config.max_size = 5000;
history.set_config(config);
```

Oldest entries are removed when the limit is exceeded.

## Built-in Commands

### `history`
Show recent command history (last 100 commands):

```bash
$ history
    1 echo hello
    2 ls -la
    3 git status
```

### `history N`
Show last N commands:

```bash
$ history 5
    1 cargo build
    2 cargo test
    3 git add .
    4 git commit -m "Add feature"
    5 git push
```

### `history search <query>`
Fuzzy search for commands:

```bash
$ history search git
[score: 124] git status
[score: 112] git commit -m 'test'
[score: 98] git push origin main
```

### `history clear`
Clear all history:

```bash
$ history clear
History cleared
```

## Integration with Reedline

The history system integrates with reedline for:
- **Ctrl+R**: Reverse incremental search
- **Up/Down arrows**: Navigate through history
- **History persistence**: Automatic save/load

```rust
use reedline::{Reedline, FileBackedHistory};

let history_file = History::default_history_file();
let mut line_editor = Reedline::create()
    .with_history(Box::new(
        FileBackedHistory::with_file(100, history_file)?
    ));
```

## API Reference

### Creating History

```rust
// Default configuration
let history = History::new();

// Custom configuration
let mut config = HistoryConfig::default();
config.max_size = 5000;
config.deduplicate_all = true;
let history = History::with_config(config);

// Custom file path
let history = History::with_file("/custom/path/.history", config);
```

### Loading and Saving

```rust
// Load from file
history.load()?;

// Save all to file
history.save()?;

// Append single entry (incremental)
let entry = HistoryEntry::new("echo test".to_string());
history.append_to_file(&entry)?;
```

### Adding Commands

```rust
// Add command (with automatic deduplication and file append)
history.add("ls -la".to_string())?;
```

### Searching

```rust
// Fuzzy search with max results
let results = history.search("cargo", 20);

// Substring search (exact match)
let results = history.search_substring("git", 10);
```

### Retrieving Entries

```rust
// Get specific entry
if let Some(entry) = history.get(0) {
    println!("{}", entry.command);
}

// Get all entries
let all = history.entries();

// Get last N entries
let recent = history.last_n(10);
```

### Clearing History

```rust
history.clear()?;
```

## Testing

The implementation includes 16 comprehensive tests covering:

1. **Basic Operations**
   - `test_add_command`: Adding commands to history
   - `test_empty_command_ignored`: Empty commands are not saved

2. **Deduplication**
   - `test_consecutive_duplicate_prevention`: Consecutive duplicates blocked
   - `test_non_consecutive_duplicates_allowed_by_default`: Non-consecutive allowed
   - `test_deduplicate_all`: Full deduplication mode

3. **Ignore Patterns**
   - `test_ignore_space`: Space-prefixed commands ignored
   - `test_ignore_patterns`: Custom pattern matching

4. **Persistence**
   - `test_persistence`: Save and load from file
   - `test_max_size_enforcement`: Size limits enforced

5. **Search**
   - `test_fuzzy_search`: Basic fuzzy matching
   - `test_fuzzy_search_ranking`: Score-based ranking
   - `test_substring_search`: Exact substring matches

6. **Retrieval**
   - `test_last_n`: Get recent commands
   - `test_timestamps`: Timestamp accuracy

7. **Management**
   - `test_clear`: Clear all history

Run tests with:
```bash
cargo test --lib history
```

## Performance Considerations

- **Incremental Saves**: Each command is appended to the file individually to prevent data loss
- **In-Memory Operations**: All searches operate on in-memory data for speed
- **Size Limits**: Automatic trimming prevents unbounded memory growth
- **Efficient Search**: SkimMatcherV2 provides O(n) fuzzy matching with good practical performance

## Future Enhancements

Potential improvements for future versions:

1. **History Sync**: Share history across multiple shell sessions
2. **Advanced Search**: Regular expression support, date range filtering
3. **Statistics**: Most used commands, command frequency analysis
4. **Export/Import**: Export to different formats (CSV, JSON)
5. **Privacy Mode**: Temporary session with no history saving
6. **Cloud Backup**: Optional cloud sync for history preservation

## Configuration Example

Here's a complete configuration example:

```rust
use aush::history::{History, HistoryConfig};

let mut config = HistoryConfig {
    max_size: 5000,
    deduplicate_all: true,
    show_timestamps: true,
    ignore_patterns: vec![
        "history".to_string(),
        "exit".to_string(),
        "clear".to_string(),
    ],
    ignore_space: true,
};

let mut history = History::with_config(config);
history.load()?;

// Use throughout session
history.add("git status".to_string())?;

// Save on exit
history.save()?;
```

## Error Handling

All I/O operations return `Result<T, anyhow::Error>` and handle errors gracefully:

- **File Not Found**: Creates new history file
- **Parse Errors**: Falls back to plain text format
- **Write Errors**: Warns but doesn't crash
- **Permission Errors**: Provides clear error messages

Example error handling:
```rust
match history.load() {
    Ok(_) => println!("History loaded successfully"),
    Err(e) => eprintln!("Warning: Could not load history: {}", e),
}
```

## References

- [Fuzzy Matcher Crate](https://docs.rs/fuzzy-matcher/)
- [Chrono for Timestamps](https://docs.rs/chrono/)
- [Reedline Line Editor](https://docs.rs/reedline/)
- [Bash HISTIGNORE](https://www.gnu.org/software/bash/manual/html_node/Bash-Variables.html)
