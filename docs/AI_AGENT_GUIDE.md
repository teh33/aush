# AUSH for AI Coding Agents

A comprehensive guide for integrating AUSH into AI coding assistants and automated workflows.

## Table of Contents

1. [Why AUSH for AI Agents?](#why-aush-for-ai-agents)
2. [Quick Start](#quick-start)
3. [Structured Git Operations](#structured-git-operations)
4. [JSON Native Operations](#json-native-operations)
5. [File Operations with JSON](#file-operations-with-json)
6. [HTTP Client](#http-client)
7. [Error Handling](#error-handling)
8. [Performance Optimization](#performance-optimization)
9. [Best Practices](#best-practices)
10. [Migration from Bash](#migration-from-bash)

## Why AUSH for AI Agents?

AI coding agents make hundreds or thousands of shell calls per task. AUSH is specifically designed to make these operations:

- **10x faster**: Native Rust implementation with zero subprocess overhead for built-ins
- **Structured output**: All commands support `--json` flag with well-defined schemas
- **Typed errors**: Machine-readable error types for intelligent error recovery
- **Reliable**: Type-safe, predictable behavior, no text parsing edge cases
- **All-in-one**: Git, JSON manipulation, HTTP requests, file operations all built-in
- **Zero dependencies**: No need for jq, curl, git, find, grep - everything is native

### Performance Comparison

| Operation | AUSH | Bash+jq | Speedup |
|-----------|------|---------|---------|
| git_status 100x | 500ms | 2000ms | 4x |
| find + filter | 10ms | 100ms | 10x |
| git_log + parse | 50ms | 200ms | 4x |
| JSON operations | 5ms | 50ms | 10x |
| Complex pipeline | 100ms | 500ms | 5x |

### Real-World Impact

For a typical AI agent task that:
- Checks git status (3x)
- Searches files (5x)
- Parses JSON (10x)
- Makes HTTP requests (2x)

**Total time**: 2-5 seconds in AUSH vs 10-20 seconds in bash+external tools

## Quick Start

### Installation

```bash
# From source
git clone https://github.com/opus-workshop/aush
cd aush
cargo install --path .

# Or via cargo (when published)
cargo install aush
```

### First Steps

```bash
# Enable JSON error format (recommended for agents)
export AUSH_ERROR_FORMAT=json

# Get repository status as JSON
aush -c "git_status --json"

# Find and process files
aush -c "find --json src/ -name '*.rs'"

# Fetch API data
aush -c "fetch --json https://api.github.com/repos/rust-lang/rust"
```

### Basic Agent Integration

```python
import subprocess
import json

def run_rush(command):
    """Run a AUSH command and return parsed JSON output."""
    result = subprocess.run(
        ['aush', '-c', command],
        capture_output=True,
        text=True,
        env={'AUSH_ERROR_FORMAT': 'json'}
    )

    if result.returncode != 0:
        error = json.loads(result.stderr)
        raise Exception(f"AUSH error: {error}")

    return json.loads(result.stdout)

# Example: Get git status
status = run_rush("git_status --json")
print(f"Branch: {status['branch']}")
print(f"Staged files: {len(status['staged'])}")
```

## Structured Git Operations

All git operations return structured JSON with consistent schemas.

### git_status

Get repository status with all changes categorized:

```bash
aush -c "git_status --json"
```

**Output:**
```json
{
  "branch": "main",
  "tracking": "origin/main",
  "ahead": 2,
  "behind": 0,
  "state": "clean",
  "staged": [
    {
      "path": "src/main.rs",
      "status": "modified"
    }
  ],
  "unstaged": [
    {
      "path": "README.md",
      "status": "modified"
    }
  ],
  "untracked": ["temp.txt"],
  "conflicted": [],
  "summary": {
    "staged_count": 1,
    "unstaged_count": 1,
    "untracked_count": 1,
    "conflicted_count": 0
  }
}
```

**Use cases:**
- Check if branch is clean before committing
- List modified files for review
- Detect merge conflicts
- Track ahead/behind status

### git_log

Get commit history with stats:

```bash
# Last 10 commits
aush -c "git_log --json -n 10"

# Commits since date
aush -c "git_log --json --since '2 weeks ago'"

# Filter by message
aush -c "git_log --json --grep 'fix:'"
```

**Output:**
```json
[
  {
    "hash": "abc123def456...",
    "short_hash": "abc123d",
    "author": "John Doe",
    "author_email": "john@example.com",
    "date": "2024-01-20T10:30:00Z",
    "timestamp": 1705747800,
    "message": "fix: resolve authentication bug",
    "files_changed": 3,
    "insertions": 45,
    "deletions": 12
  }
]
```

**Use cases:**
- Generate commit message suggestions
- Analyze change patterns
- Find specific commits
- Calculate churn metrics

### git_diff

Get detailed diff information:

```bash
# Unstaged changes
aush -c "git_diff --json"

# Staged changes
aush -c "git_diff --json --staged"

# Commit range
aush -c "git_diff --json HEAD~1..HEAD"

# Specific file
aush -c "git_diff --json src/main.rs"
```

**Output:**
```json
{
  "files": [
    {
      "path": "src/main.rs",
      "status": "modified",
      "additions": 10,
      "deletions": 3,
      "hunks": [
        {
          "old_start": 42,
          "old_lines": 5,
          "new_start": 42,
          "new_lines": 12,
          "header": "@@ -42,5 +42,12 @@ fn main() {",
          "changes": [
            {
              "type": "context",
              "line": "fn main() {"
            },
            {
              "type": "add",
              "line": "    println!(\"Hello, AUSH!\");"
            },
            {
              "type": "delete",
              "line": "    println!(\"Hello, World!\");"
            }
          ]
        }
      ]
    }
  ],
  "summary": {
    "files_changed": 1,
    "insertions": 10,
    "deletions": 3
  }
}
```

**Use cases:**
- Generate commit messages from changes
- Code review preparation
- Detect sensitive data in changes
- Calculate change complexity

## JSON Native Operations

AUSH includes built-in JSON manipulation commands that are 10x faster than jq.

### json_get

Extract values from JSON:

```bash
# Get a field
echo '{"name":"John","age":30}' | aush -c "json_get '.name'"
# Output: John

# Get nested field
echo '{"user":{"name":"John"}}' | aush -c "json_get '.user.name'"
# Output: John

# Get array element
echo '[1,2,3]' | aush -c "json_get '.[1]'"
# Output: 2

# Iterate array
echo '[{"id":1},{"id":2}]' | aush -c "json_get '.[].id'"
# Output: 1
#         2
```

### json_set

Modify JSON values:

```bash
# Set a field
echo '{"name":"John"}' | aush -c "json_set '.age' 30"
# Output: {"name":"John","age":30}

# Set nested field
echo '{}' | aush -c "json_set '.user.name' 'John'"
# Output: {"user":{"name":"John"}}

# Modify array element
echo '[1,2,3]' | aush -c "json_set '.[1]' 99"
# Output: [1,99,3]
```

### json_query

Filter and transform JSON (jq-compatible):

```bash
# Filter array
echo '[{"age":25},{"age":35}]' | aush -c "json_query '.[] | select(.age > 30)'"
# Output: {"age":35}

# Transform objects
echo '{"users":[{"name":"John"},{"name":"Jane"}]}' | \
  aush -c "json_query '.users[] | {name: .name, greeting: \"Hello \\(.name)!\"}'"
# Output: {"name":"John","greeting":"Hello John!"}
#         {"name":"Jane","greeting":"Hello Jane!"}
```

### Combining with Git Commands

```bash
# Get all modified file paths
aush -c "git_status --json | json_get '.unstaged[].path'"

# Count staged files
aush -c "git_status --json | json_get '.summary.staged_count'"

# Get commit messages from last 5 commits
aush -c "git_log --json -n 5 | json_get '.[].message'"

# Find commits by specific author
aush -c "git_log --json | json_query '.[] | select(.author == \"John Doe\")'"
```

## File Operations with JSON

All file operation builtins support JSON output.

### ls

List directory contents:

```bash
aush -c "ls --json"
```

**Output:**
```json
[
  {
    "name": "src",
    "path": "/home/user/project/src",
    "type": "directory",
    "size": 4096,
    "modified": "2024-01-20T10:30:00Z",
    "modified_timestamp": 1705747800,
    "permissions": "rwxr-xr-x",
    "mode": 493
  },
  {
    "name": "main.rs",
    "path": "/home/user/project/src/main.rs",
    "type": "file",
    "size": 1024,
    "modified": "2024-01-20T09:15:00Z",
    "modified_timestamp": 1705743300,
    "permissions": "rw-r--r--",
    "mode": 420
  }
]
```

### find

Search for files:

```bash
# Find all Rust files
aush -c "find --json src/ -name '*.rs'"

# Find large files
aush -c "find --json . -size +1M"

# Find recently modified
aush -c "find --json . -mtime -7d"
```

**Output:**
```json
[
  {
    "path": "/home/user/project/src/main.rs",
    "type": "file",
    "size": 2048,
    "modified": "2024-01-20T10:30:00Z",
    "permissions": "rw-r--r--"
  }
]
```

### grep

Search file contents:

```bash
# Search for pattern
aush -c "grep --json 'TODO' src/**/*.rs"

# Case-insensitive
aush -c "grep --json -i 'fixme' src/**/*.rs"

# With context lines
aush -c "grep --json -C 2 'error' src/main.rs"
```

**Output:**
```json
[
  {
    "file": "src/main.rs",
    "line_number": 42,
    "column": 8,
    "match": "TODO",
    "full_line": "    // TODO: implement error handling",
    "context_before": [
      "fn process_data() {",
      "    let data = load_data();"
    ],
    "context_after": [
      "    save_data(data);",
      "}"
    ]
  }
]
```

## HTTP Client

Built-in HTTP client with JSON support.

### Basic Requests

```bash
# GET request
aush -c "fetch --json https://api.github.com/repos/rust-lang/rust"

# POST request with JSON body
aush -c "fetch --json -X POST https://api.example.com/data -d '{\"key\":\"value\"}'"

# Custom headers
aush -c "fetch --json -H 'Authorization: Bearer token123' https://api.example.com/user"
```

### Response Format

```json
{
  "status": 200,
  "status_text": "OK",
  "headers": {
    "content-type": "application/json",
    "content-length": "1024"
  },
  "body": {
    "id": 1,
    "name": "Rust"
  },
  "response_time_ms": 150,
  "url": "https://api.github.com/repos/rust-lang/rust"
}
```

### Advanced Usage

```bash
# Extract specific field from API response
aush -c "fetch --json https://api.github.com/repos/rust-lang/rust | json_get '.body.stargazers_count'"

# Check HTTP status
aush -c "fetch --json https://example.com | json_get '.status'"

# Download file
aush -c "fetch -o output.zip https://example.com/file.zip"

# Timeout control
aush -c "fetch --timeout 10 https://slow-api.example.com"
```

## Error Handling

AUSH provides structured error information when `AUSH_ERROR_FORMAT=json` is set.

### Error Response Schema

```json
{
  "error": "CommandNotFound",
  "message": "Command 'nonexistent' not found",
  "exit_code": 127,
  "command": "nonexistent",
  "context": {
    "cwd": "/home/user/project",
    "shell_pid": 12345
  }
}
```

### Error Types

- `CommandNotFound`: Command doesn't exist
- `PermissionDenied`: Insufficient permissions
- `InvalidArgument`: Invalid command arguments
- `GitError`: Git operation failed
- `IOError`: File I/O error
- `ParseError`: JSON/data parsing error
- `NetworkError`: HTTP request failed
- `TimeoutError`: Operation timed out

### Error Handling in Agents

```python
import subprocess
import json

def safe_rush_command(cmd):
    """Execute AUSH command with proper error handling."""
    result = subprocess.run(
        ['aush', '-c', cmd],
        capture_output=True,
        text=True,
        env={'AUSH_ERROR_FORMAT': 'json'}
    )

    if result.returncode != 0:
        try:
            error = json.loads(result.stderr)
            error_type = error.get('error', 'Unknown')
            message = error.get('message', 'No message')

            # Handle specific error types
            if error_type == 'GitError':
                if 'not a git repository' in message:
                    print("Not in a git repository, skipping git operations")
                    return None
                elif 'no changes' in message:
                    print("No changes to commit")
                    return None
            elif error_type == 'NetworkError':
                print(f"Network error: {message}, retrying...")
                # Implement retry logic

            raise Exception(f"{error_type}: {message}")
        except json.JSONDecodeError:
            raise Exception(f"Command failed: {result.stderr}")

    return json.loads(result.stdout) if result.stdout else None
```

## Performance Optimization

### Batch Operations

Instead of multiple calls:
```bash
# SLOW: Multiple AUSH invocations
aush -c "git_status --json" > status.json
aush -c "git_log --json -n 10" > log.json
aush -c "git_diff --json --staged" > diff.json
```

Do this:
```bash
# FAST: Single AUSH session
aush -c "
  git_status --json > status.json
  git_log --json -n 10 > log.json
  git_diff --json --staged > diff.json
"
```

### Use JSON Pipelines

Instead of:
```bash
# SLOW: Multiple JSON parse cycles
files=$(aush -c "find --json src/ -name '*.rs'")
for file in $files; do
  aush -c "grep --json 'TODO' $file"
done
```

Do this:
```bash
# FAST: Single pipeline
aush -c "find --json src/ -name '*.rs' | json_get '.[].path' | xargs grep --json 'TODO'"
```

### Minimize Context Switching

Keep AUSH session alive for interactive agents:
```python
import subprocess
import json

class AUSHSession:
    def __init__(self):
        # Start persistent AUSH session
        self.process = subprocess.Popen(
            ['aush'],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )

    def execute(self, command):
        """Execute command in persistent session."""
        self.process.stdin.write(f"{command}\n")
        self.process.stdin.flush()
        # Read until prompt
        output = []
        # Implementation depends on your needs
        return output
```

## Best Practices

### 1. Always Use --json for Programmatic Access

```bash
# GOOD: Structured output
aush -c "git_status --json | json_get '.unstaged[].path'"

# BAD: Parsing text output
aush -c "git status | grep modified | awk '{print $2}'"
```

### 2. Set Environment Variables

```bash
# Set in agent initialization
export AUSH_ERROR_FORMAT=json  # Structured errors
export AUSH_COLOR=never        # Disable colors in JSON mode
export AUSH_PAGER=cat          # Disable pager for automated use
```

### 3. Validate Input

```bash
# Check if in git repository before git operations
if aush -c "git_status --json" 2>/dev/null; then
  # Proceed with git operations
fi
```

### 4. Handle Empty Results

```python
def get_modified_files():
    result = run_rush("git_status --json")
    unstaged = result.get('unstaged', [])

    if not unstaged:
        print("No unstaged changes")
        return []

    return [f['path'] for f in unstaged]
```

### 5. Use Timeouts for Network Operations

```bash
# Always set timeout for fetch operations
aush -c "fetch --timeout 10 --json https://api.example.com/data"
```

### 6. Combine Operations

```bash
# GOOD: Single pipeline
aush -c "find --json src/ -name '*.rs' | json_query '.[] | select(.size > 1000) | .path'"

# BAD: Multiple commands
files=$(aush -c "find --json src/ -name '*.rs'")
aush -c "echo '$files' | json_query '.[] | select(.size > 1000) | .path'"
```

### 7. Cache When Appropriate

```python
class GitAwareAgent:
    def __init__(self):
        self._status_cache = None
        self._status_time = None

    def get_status(self, max_age=5):
        """Get git status with caching."""
        now = time.time()
        if (self._status_cache is None or
            self._status_time is None or
            now - self._status_time > max_age):
            self._status_cache = run_rush("git_status --json")
            self._status_time = now
        return self._status_cache
```

## Migration from Bash

Common patterns translated from bash+jq to AUSH.

### Check Git Status

**Bash:**
```bash
git status --porcelain | grep "^M" | cut -c 4-
```

**AUSH:**
```bash
aush -c "git_status --json | json_get '.unstaged[] | select(.status == \"modified\") | .path'"
```

### Find TODO Comments

**Bash:**
```bash
find src -name '*.rs' -exec grep -n "TODO" {} + | cut -d: -f1,2
```

**AUSH:**
```bash
aush -c "grep --json 'TODO' src/**/*.rs | json_get '.[] | \"\\(.file):\\(.line_number)\"'"
```

### Get Recent Commits

**Bash:**
```bash
git log -n 10 --pretty=format:'%h %s' | cat
```

**AUSH:**
```bash
aush -c "git_log --json -n 10 | json_get '.[] | \"\\(.short_hash) \\(.message)\"'"
```

### Fetch API Data

**Bash:**
```bash
curl -s https://api.github.com/repos/rust-lang/rust | jq '.stargazers_count'
```

**AUSH:**
```bash
aush -c "fetch --json https://api.github.com/repos/rust-lang/rust | json_get '.body.stargazers_count'"
```

### Count Lines of Code

**Bash:**
```bash
find src -name '*.rs' -exec wc -l {} + | tail -1 | awk '{print $1}'
```

**AUSH:**
```bash
aush -c "find --json src -name '*.rs' | json_query '[.[].size] | add'"
```

### List Large Files

**Bash:**
```bash
find . -type f -size +1M -exec ls -lh {} + | awk '{print $9, $5}'
```

**AUSH:**
```bash
aush -c "find --json . -size +1M | json_get '.[] | \"\\(.path) \\(.size)\"'"
```

## Example Workflows

See the [examples/](../examples/) directory for complete working examples:

- `commit_message_generator.aush` - Generate intelligent commit messages
- `find_todos.aush` - Find and categorize TODO comments
- `dependency_check.aush` - Check for outdated dependencies
- `code_review_prep.aush` - Prepare code review summaries
- `test_coverage_analyzer.aush` - Analyze test coverage
- `dead_code_finder.aush` - Find unused code
- `security_audit.aush` - Basic security checks
- `performance_profiler.aush` - Profile git operations
- `branch_cleaner.aush` - Clean up merged branches
- `changelog_generator.aush` - Generate changelogs

## Resources

- [JSON Schema Reference](AI_AGENT_JSON_REFERENCE.md) - Complete JSON schemas
- [AUSH Performance Guide](PERFORMANCE.md) - Performance optimization
- [AUSH Architecture](daemon-architecture.md) - How AUSH works internally
- [Examples Directory](../examples/) - Working example scripts

## Support

For issues or questions:
- GitHub Issues: https://github.com/opus-workshop/aush/issues
- Documentation: https://aush.dev/docs
- Examples: https://github.com/opus-workshop/aush/tree/main/examples
