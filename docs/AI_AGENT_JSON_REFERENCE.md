# AUSH JSON Schema Reference

Complete reference for all JSON output formats in AUSH. All commands that support `--json` flag are documented here with schemas, examples, and field descriptions.

## Table of Contents

1. [Git Operations](#git-operations)
   - [git_status](#git_status)
   - [git_log](#git_log)
   - [git_diff](#git_diff)
2. [File Operations](#file-operations)
   - [ls](#ls)
   - [find](#find)
   - [grep](#grep)
3. [JSON Manipulation](#json-manipulation)
   - [json_get](#json_get)
   - [json_set](#json_set)
   - [json_query](#json_query)
4. [HTTP Operations](#http-operations)
   - [fetch](#fetch)
5. [Error Responses](#error-responses)

---

## Git Operations

### git_status

Get repository status with all changes categorized.

#### Usage

```bash
aush -c "git_status --json"
```

#### Schema

```typescript
{
  branch: string | null,           // Current branch name, null if detached HEAD
  tracking: string | null,         // Remote tracking branch, null if none
  ahead: number,                   // Commits ahead of tracking branch
  behind: number,                  // Commits behind tracking branch
  state: "clean" | "dirty",        // Repository state
  staged: FileStatusEntry[],       // Files in staging area
  unstaged: FileStatusEntry[],     // Modified files not staged
  untracked: string[],             // Untracked files
  conflicted: string[],            // Files with merge conflicts
  summary: {
    staged_count: number,          // Number of staged files
    unstaged_count: number,        // Number of unstaged files
    untracked_count: number,       // Number of untracked files
    conflicted_count: number       // Number of conflicted files
  }
}

interface FileStatusEntry {
  path: string,                    // File path relative to repo root
  status: "modified" | "added" | "deleted" | "renamed" | "typechange"
}
```

#### Example Output

```json
{
  "branch": "feature/ai-integration",
  "tracking": "origin/feature/ai-integration",
  "ahead": 3,
  "behind": 1,
  "state": "dirty",
  "staged": [
    {
      "path": "src/main.rs",
      "status": "modified"
    },
    {
      "path": "docs/README.md",
      "status": "added"
    }
  ],
  "unstaged": [
    {
      "path": "Cargo.toml",
      "status": "modified"
    }
  ],
  "untracked": [
    "temp.txt",
    ".vscode/settings.json"
  ],
  "conflicted": [],
  "summary": {
    "staged_count": 2,
    "unstaged_count": 1,
    "untracked_count": 2,
    "conflicted_count": 0
  }
}
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `branch` | string \| null | Current branch name. `null` if in detached HEAD state |
| `tracking` | string \| null | Remote tracking branch (e.g., "origin/main"). `null` if no upstream |
| `ahead` | number | Number of commits the local branch is ahead of remote |
| `behind` | number | Number of commits the local branch is behind remote |
| `state` | enum | "clean" if no changes, "dirty" if there are changes |
| `staged` | array | List of files in the staging area ready to commit |
| `unstaged` | array | List of modified files not yet staged |
| `untracked` | array | List of files not tracked by git (string paths) |
| `conflicted` | array | List of files with merge conflicts (string paths) |
| `summary` | object | Aggregate counts for quick checks |

#### Use Cases

**Check if repository is clean:**
```bash
aush -c "git_status --json | json_get '.summary.staged_count + .summary.unstaged_count + .summary.untracked_count'"
# Output: 0 (if clean)
```

**Get only modified files:**
```bash
aush -c "git_status --json | json_get '.unstaged[] | select(.status == \"modified\") | .path'"
```

**Check if ahead of remote:**
```bash
aush -c "git_status --json | json_get '.ahead > 0'"
```

---

### git_log

Get commit history with statistics.

#### Usage

```bash
# Last 10 commits (default limit: 100)
aush -c "git_log --json -n 10"

# Filter by date
aush -c "git_log --json --since '2 weeks ago'"

# Filter by message
aush -c "git_log --json --grep 'fix:'"

# For specific path
aush -c "git_log --json src/main.rs"
```

#### Schema

```typescript
[
  {
    hash: string,                  // Full commit hash (40 chars)
    short_hash: string,            // Short commit hash (7 chars)
    author: string,                // Author name
    author_email: string,          // Author email
    date: string,                  // ISO 8601 timestamp
    timestamp: number,             // Unix timestamp (seconds)
    message: string,               // Commit message (full)
    files_changed: number,         // Number of files changed
    insertions: number,            // Lines added
    deletions: number              // Lines deleted
  }
]
```

#### Example Output

```json
[
  {
    "hash": "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0",
    "short_hash": "a1b2c3d",
    "author": "Jane Smith",
    "author_email": "jane@example.com",
    "date": "2024-01-20T14:30:00Z",
    "timestamp": 1705759800,
    "message": "feat: add AI agent integration\n\nImplemented JSON output for all git commands\nAdded comprehensive documentation",
    "files_changed": 5,
    "insertions": 245,
    "deletions": 18
  },
  {
    "hash": "b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1",
    "short_hash": "b2c3d4e",
    "author": "John Doe",
    "author_email": "john@example.com",
    "date": "2024-01-19T09:15:00Z",
    "timestamp": 1705654500,
    "message": "fix: resolve merge conflict in parser",
    "files_changed": 2,
    "insertions": 12,
    "deletions": 8
  }
]
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `hash` | string | Full SHA-1 commit hash (40 hexadecimal characters) |
| `short_hash` | string | Abbreviated commit hash (first 7 characters) |
| `author` | string | Name of the commit author |
| `author_email` | string | Email address of the commit author |
| `date` | string | Commit date in ISO 8601 format (UTC) |
| `timestamp` | number | Unix timestamp in seconds since epoch |
| `message` | string | Full commit message including body |
| `files_changed` | number | Total number of files modified in commit |
| `insertions` | number | Total lines added across all files |
| `deletions` | number | Total lines removed across all files |

#### Use Cases

**Get commit messages for changelog:**
```bash
aush -c "git_log --json -n 20 | json_get '.[].message' | grep '^feat:'"
```

**Find commits by author:**
```bash
aush -c "git_log --json | json_query '.[] | select(.author == \"Jane Smith\")'"
```

**Calculate total churn in last 10 commits:**
```bash
aush -c "git_log --json -n 10 | json_query '[.[].insertions] | add'"
```

---

### git_diff

Get detailed diff information with hunks and line-by-line changes.

#### Usage

```bash
# Unstaged changes
aush -c "git_diff --json"

# Staged changes
aush -c "git_diff --json --staged"

# Commit range
aush -c "git_diff --json HEAD~1..HEAD"

# Specific file
aush -c "git_diff --json src/main.rs"

# Summary only
aush -c "git_diff --json --stat"

# File names only
aush -c "git_diff --json --name-only"
```

#### Schema

```typescript
{
  files: FileDiff[],
  summary: {
    files_changed: number,
    insertions: number,
    deletions: number
  }
}

interface FileDiff {
  path: string,                    // Current file path
  old_path?: string,               // Previous path (for renames)
  status: FileStatus,              // Type of change
  additions: number,               // Lines added in this file
  deletions: number,               // Lines deleted in this file
  hunks: Hunk[]                    // Diff hunks (empty with --stat or --name-only)
}

type FileStatus =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "copied"
  | "untracked"
  | "binary";

interface Hunk {
  old_start: number,               // Starting line in old file
  old_lines: number,               // Number of lines in old file
  new_start: number,               // Starting line in new file
  new_lines: number,               // Number of lines in new file
  header: string,                  // Hunk header line
  changes: LineChange[]            // Individual line changes
}

interface LineChange {
  type: "context" | "add" | "delete",
  line: string                     // Line content (without +/- prefix)
}
```

#### Example Output

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
          "old_start": 15,
          "old_lines": 8,
          "new_start": 15,
          "new_lines": 15,
          "header": "@@ -15,8 +15,15 @@ fn process_data(input: &str) -> Result<String> {",
          "changes": [
            {
              "type": "context",
              "line": "fn process_data(input: &str) -> Result<String> {"
            },
            {
              "type": "context",
              "line": "    let mut result = String::new();"
            },
            {
              "type": "add",
              "line": "    // Validate input first"
            },
            {
              "type": "add",
              "line": "    if input.is_empty() {"
            },
            {
              "type": "add",
              "line": "        return Err(\"Input cannot be empty\".into());"
            },
            {
              "type": "add",
              "line": "    }"
            },
            {
              "type": "context",
              "line": "    for line in input.lines() {"
            },
            {
              "type": "delete",
              "line": "        result.push_str(line);"
            },
            {
              "type": "add",
              "line": "        result.push_str(&format!(\"Processed: {}\\n\", line));"
            },
            {
              "type": "context",
              "line": "    }"
            },
            {
              "type": "context",
              "line": "    Ok(result)"
            },
            {
              "type": "context",
              "line": "}"
            }
          ]
        }
      ]
    },
    {
      "path": "docs/README.md",
      "status": "added",
      "additions": 45,
      "deletions": 0,
      "hunks": [
        {
          "old_start": 0,
          "old_lines": 0,
          "new_start": 1,
          "new_lines": 45,
          "header": "@@ -0,0 +1,45 @@",
          "changes": [
            {
              "type": "add",
              "line": "# AUSH AI Integration"
            },
            {
              "type": "add",
              "line": ""
            },
            {
              "type": "add",
              "line": "This document describes..."
            }
          ]
        }
      ]
    }
  ],
  "summary": {
    "files_changed": 2,
    "insertions": 55,
    "deletions": 3
  }
}
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `files` | array | List of all changed files with their diffs |
| `files[].path` | string | Current file path relative to repo root |
| `files[].old_path` | string? | Previous path (only present for renamed files) |
| `files[].status` | enum | Type of change: modified/added/deleted/renamed/copied/binary |
| `files[].additions` | number | Number of lines added in this file |
| `files[].deletions` | number | Number of lines deleted in this file |
| `files[].hunks` | array | Diff hunks (contiguous changed regions) |
| `hunks[].old_start` | number | Starting line number in original file |
| `hunks[].old_lines` | number | Number of lines from original file in hunk |
| `hunks[].new_start` | number | Starting line number in new file |
| `hunks[].new_lines` | number | Number of lines from new file in hunk |
| `hunks[].header` | string | Diff header line (e.g., "@@ -15,8 +15,15 @@...") |
| `hunks[].changes` | array | Individual line changes in this hunk |
| `changes[].type` | enum | "context" (unchanged), "add" (added), "delete" (removed) |
| `changes[].line` | string | Line content without +/- prefix |
| `summary` | object | Aggregate statistics across all files |

#### Use Cases

**Generate commit message from diff:**
```bash
aush -c "git_diff --json --staged | json_query '.files[] | \"- Modified \\(.path): +\\(.additions) -\\(.deletions)\"'"
```

**Find files with large changes:**
```bash
aush -c "git_diff --json | json_query '.files[] | select(.additions + .deletions > 50)'"
```

**Detect TODO comments in changes:**
```bash
aush -c "git_diff --json | json_query '.files[].hunks[].changes[] | select(.type == \"add\" and (.line | contains(\"TODO\")))'"
```

**Get summary only:**
```bash
aush -c "git_diff --json --stat | json_get '.summary'"
```

---

## File Operations

### ls

List directory contents with metadata.

#### Usage

```bash
# Current directory
aush -c "ls --json"

# Specific directory
aush -c "ls --json /path/to/dir"

# Show hidden files
aush -c "ls --json -a"

# Long format (same as --json, included for compatibility)
aush -c "ls --json -l"
```

#### Schema

```typescript
[
  {
    name: string,                  // File/directory name
    path: string,                  // Absolute path
    type: "file" | "directory" | "symlink" | "other",
    size: number,                  // Size in bytes
    modified: string,              // ISO 8601 timestamp
    modified_timestamp: number,    // Unix timestamp (seconds)
    permissions: string,           // Human-readable (e.g., "rwxr-xr-x")
    mode: number,                  // Unix mode as number
    symlink_target?: string        // Link target (only for symlinks)
  }
]
```

#### Example Output

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
    "size": 2048,
    "modified": "2024-01-20T09:15:00Z",
    "modified_timestamp": 1705743300,
    "permissions": "rw-r--r--",
    "mode": 420
  },
  {
    "name": "config",
    "path": "/home/user/project/config",
    "type": "symlink",
    "size": 15,
    "modified": "2024-01-19T14:00:00Z",
    "modified_timestamp": 1705673600,
    "permissions": "rwxrwxrwx",
    "mode": 511,
    "symlink_target": "/etc/app/config"
  }
]
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | File or directory name (basename) |
| `path` | string | Full absolute path |
| `type` | enum | Entry type: file/directory/symlink/other |
| `size` | number | Size in bytes (for directories, typically block size) |
| `modified` | string | Last modification time in ISO 8601 format |
| `modified_timestamp` | number | Unix timestamp (seconds since epoch) |
| `permissions` | string | Human-readable permissions (e.g., "rwxr-xr-x") |
| `mode` | number | Unix file mode as octal number (e.g., 493 = 0755) |
| `symlink_target` | string? | Target path for symbolic links (only present for symlinks) |

#### Use Cases

**Find large files:**
```bash
aush -c "ls --json | json_query '.[] | select(.size > 1000000)'"
```

**Get only directories:**
```bash
aush -c "ls --json | json_query '.[] | select(.type == \"directory\") | .name'"
```

**Sort by modification time:**
```bash
aush -c "ls --json | json_query 'sort_by(.modified_timestamp) | reverse'"
```

---

### find

Search for files and directories with filters.

#### Usage

```bash
# Find all files in directory
aush -c "find --json /path/to/search"

# Filter by name pattern (glob)
aush -c "find --json . -name '*.rs'"

# Filter by type
aush -c "find --json . -type f"  # files only
aush -c "find --json . -type d"  # directories only

# Filter by size
aush -c "find --json . -size +1M"      # larger than 1MB
aush -c "find --json . -size -100k"    # smaller than 100KB

# Filter by modification time
aush -c "find --json . -mtime -7d"     # modified in last 7 days
aush -c "find --json . -mtime +30d"    # modified more than 30 days ago

# Maximum depth
aush -c "find --json . -maxdepth 2"

# Disable gitignore
aush -c "find --json . --no-ignore"
```

#### Schema

```typescript
[
  {
    path: string,                  // Absolute file path
    type: "file" | "directory",    // Entry type
    size: number,                  // Size in bytes
    modified: string,              // ISO 8601 timestamp
    permissions?: string           // Unix permissions (e.g., "rw-r--r--")
  }
]
```

#### Example Output

```json
[
  {
    "path": "/home/user/project/src/main.rs",
    "type": "file",
    "size": 2048,
    "modified": "2024-01-20T10:30:00Z",
    "permissions": "rw-r--r--"
  },
  {
    "path": "/home/user/project/src/lib.rs",
    "type": "file",
    "size": 1536,
    "modified": "2024-01-20T09:00:00Z",
    "permissions": "rw-r--r--"
  },
  {
    "path": "/home/user/project/src/utils",
    "type": "directory",
    "size": 4096,
    "modified": "2024-01-19T15:30:00Z",
    "permissions": "rwxr-xr-x"
  }
]
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | Full absolute path to the file/directory |
| `type` | enum | "file" for regular files, "directory" for directories |
| `size` | number | Size in bytes |
| `modified` | string | Last modification time in ISO 8601 format |
| `permissions` | string? | Unix-style permissions (may be omitted on some platforms) |

#### Use Cases

**Find all Rust source files:**
```bash
aush -c "find --json . -name '*.rs' | json_get '.[].path'"
```

**Find large files:**
```bash
aush -c "find --json . -type f -size +10M"
```

**Find recently modified files:**
```bash
aush -c "find --json . -mtime -1d | json_query 'sort_by(.modified) | reverse'"
```

---

### grep

Search file contents with pattern matching.

#### Usage

```bash
# Search for pattern
aush -c "grep --json 'pattern' file.txt"

# Case-insensitive
aush -c "grep --json -i 'pattern' file.txt"

# Recursive search
aush -c "grep --json -r 'pattern' src/"

# With context lines
aush -c "grep --json -C 2 'pattern' file.txt"  # 2 lines before and after
aush -c "grep --json -B 3 'pattern' file.txt"  # 3 lines before
aush -c "grep --json -A 1 'pattern' file.txt"  # 1 line after

# Respect/ignore .gitignore
aush -c "grep --json --no-ignore 'pattern' ."  # ignore .gitignore
```

#### Schema

```typescript
[
  {
    file: string,                  // File path
    line_number: number,           // Line number (1-indexed)
    column?: number,               // Column number (0-indexed), optional
    match: string,                 // Matched text
    full_line: string,             // Complete line containing match
    context_before?: string[],     // Lines before match (with -B or -C)
    context_after?: string[]       // Lines after match (with -A or -C)
  }
]
```

#### Example Output

```json
[
  {
    "file": "src/main.rs",
    "line_number": 42,
    "column": 8,
    "match": "TODO",
    "full_line": "    // TODO: implement error handling",
    "context_before": [
      "fn process_data(input: &str) -> Result<String> {",
      "    let mut result = String::new();"
    ],
    "context_after": [
      "    for line in input.lines() {",
      "        result.push_str(line);"
    ]
  },
  {
    "file": "src/lib.rs",
    "line_number": 156,
    "column": 4,
    "match": "TODO",
    "full_line": "    // TODO: add tests for edge cases"
  }
]
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `file` | string | Path to file containing the match |
| `line_number` | number | Line number where match was found (1-indexed) |
| `column` | number? | Column where match starts (0-indexed), may be omitted |
| `match` | string | The exact text that matched the pattern |
| `full_line` | string | Complete line containing the match |
| `context_before` | array? | Lines before the match (only with -B or -C flags) |
| `context_after` | array? | Lines after the match (only with -A or -C flags) |

#### Use Cases

**Find all TODO comments:**
```bash
aush -c "grep --json 'TODO' src/**/*.rs | json_get '.[].file' | sort | uniq"
```

**Get TODO count per file:**
```bash
aush -c "grep --json 'TODO' src/ | json_query 'group_by(.file) | map({file: .[0].file, count: length})'"
```

**Find security issues:**
```bash
aush -c "grep --json -i 'password.*=.*\"' src/ -C 2"
```

---

## JSON Manipulation

### json_get

Extract values from JSON data using path expressions.

#### Usage

```bash
# Get top-level field
echo '{"name": "John", "age": 30}' | aush -c "json_get '.name'"

# Get nested field
echo '{"user": {"name": "John"}}' | aush -c "json_get '.user.name'"

# Get array element
echo '[10, 20, 30]' | aush -c "json_get '.[1]'"

# Iterate array
echo '[{"id": 1}, {"id": 2}]' | aush -c "json_get '.[].id'"

# From file
aush -c "json_get '.version' package.json"
```

#### Path Syntax

- `.field` - Access object field
- `.field.nested` - Access nested field
- `.[0]` - Access array element by index
- `.[]` - Iterate array elements
- `.field[0]` - Access field then array element

#### Output Format

Returns raw values (not JSON-encoded):
- Strings: unquoted
- Numbers: as-is
- Booleans: true/false
- Null: null
- Arrays: one element per line
- Objects: JSON-encoded

#### Examples

**Input:** `{"name": "John", "age": 30}`

| Command | Output |
|---------|--------|
| `json_get '.name'` | `John` |
| `json_get '.age'` | `30` |

**Input:** `[{"id": 1, "name": "A"}, {"id": 2, "name": "B"}]`

| Command | Output |
|---------|--------|
| `json_get '.[0].id'` | `1` |
| `json_get '.[].name'` | `A`<br>`B` |

---

### json_set

Modify JSON values at specified paths.

#### Usage

```bash
# Set field value
echo '{"name": "John"}' | aush -c "json_set '.age' 30"
# Output: {"name":"John","age":30}

# Set nested field
echo '{}' | aush -c "json_set '.user.name' 'John'"
# Output: {"user":{"name":"John"}}

# Modify array element
echo '[1, 2, 3]' | aush -c "json_set '.[1]' 99"
# Output: [1,99,3]

# Set from file
aush -c "json_set '.version' '2.0.0' package.json > package.json.new"
```

#### Output Format

Always returns valid JSON (formatted).

#### Examples

**Set simple field:**
```bash
echo '{"name": "John"}' | aush -c "json_set '.age' 30"
```
Output:
```json
{
  "name": "John",
  "age": 30
}
```

**Create nested structure:**
```bash
echo '{}' | aush -c "json_set '.config.database.host' 'localhost'"
```
Output:
```json
{
  "config": {
    "database": {
      "host": "localhost"
    }
  }
}
```

---

### json_query

Filter and transform JSON using jq-compatible syntax.

#### Usage

```bash
# Filter array
echo '[{"age": 25}, {"age": 35}]' | aush -c "json_query '.[] | select(.age > 30)'"

# Transform objects
echo '{"users": [{"name": "John"}, {"name": "Jane"}]}' | \
  aush -c "json_query '.users[] | {name: .name, greeting: \"Hello \\(.name)!\"}'"

# Map and reduce
echo '[1, 2, 3, 4, 5]' | aush -c "json_query 'map(. * 2) | add'"
```

#### Supported Operations

- `select(condition)` - Filter elements
- `map(expr)` - Transform each element
- `{key: value}` - Construct objects
- `[expr]` - Construct arrays
- Arithmetic: `+`, `-`, `*`, `/`, `%`
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Logic: `and`, `or`, `not`
- String interpolation: `"\\(expr)"`
- Functions: `add`, `length`, `keys`, `values`, `sort_by`, `group_by`, `unique`

#### Examples

**Filter and project:**
```bash
echo '[{"name":"John","age":30},{"name":"Jane","age":25}]' | \
  aush -c "json_query '.[] | select(.age > 25) | {name}'"
```
Output:
```json
{"name": "John"}
```

**Group and count:**
```bash
echo '[{"type":"bug"},{"type":"feature"},{"type":"bug"}]' | \
  aush -c "json_query 'group_by(.type) | map({type: .[0].type, count: length})'"
```
Output:
```json
[
  {"type": "bug", "count": 2},
  {"type": "feature", "count": 1}
]
```

---

## HTTP Operations

### fetch

Make HTTP requests and receive structured responses.

#### Usage

```bash
# GET request
aush -c "fetch --json https://api.github.com/repos/rust-lang/rust"

# POST with JSON body
aush -c "fetch --json -X POST https://api.example.com/data -d '{\"key\":\"value\"}'"

# Custom headers
aush -c "fetch --json -H 'Authorization: Bearer token' https://api.example.com/user"

# Timeout
aush -c "fetch --json --timeout 10 https://slow-api.com"

# Save to file (no --json needed)
aush -c "fetch -o output.zip https://example.com/file.zip"

# Read body from stdin
echo '{"data":"value"}' | aush -c "fetch --json -X POST https://api.example.com -d @-"
```

#### Schema

```typescript
{
  status: number,                  // HTTP status code
  status_text: string,             // Status text (e.g., "OK")
  headers: {                       // Response headers
    [key: string]: string
  },
  body: any,                       // Parsed response body (JSON or text)
  response_time_ms: number,        // Request duration in milliseconds
  url: string                      // Final URL (after redirects)
}
```

#### Example Output

```json
{
  "status": 200,
  "status_text": "OK",
  "headers": {
    "content-type": "application/json; charset=utf-8",
    "content-length": "2847",
    "server": "GitHub.com",
    "date": "Fri, 20 Jan 2024 10:30:00 GMT",
    "cache-control": "public, max-age=60, s-maxage=60"
  },
  "body": {
    "id": 724712,
    "name": "rust",
    "full_name": "rust-lang/rust",
    "description": "Empowering everyone to build reliable and efficient software.",
    "stargazers_count": 89234,
    "forks_count": 12456,
    "open_issues_count": 9876
  },
  "response_time_ms": 145,
  "url": "https://api.github.com/repos/rust-lang/rust"
}
```

#### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `status` | number | HTTP status code (200, 404, 500, etc.) |
| `status_text` | string | Human-readable status ("OK", "Not Found", etc.) |
| `headers` | object | All response headers (keys lowercase) |
| `body` | any | Response body (parsed JSON if content-type is JSON, else string) |
| `response_time_ms` | number | Total request duration in milliseconds |
| `url` | string | Final URL after following any redirects |

#### Use Cases

**Extract specific field:**
```bash
aush -c "fetch --json https://api.github.com/repos/rust-lang/rust | json_get '.body.stargazers_count'"
```

**Check status code:**
```bash
status=$(aush -c "fetch --json https://example.com | json_get '.status'")
if [ "$status" = "200" ]; then
  echo "Success"
fi
```

**Handle errors:**
```bash
response=$(aush -c "fetch --json https://api.example.com/data")
status=$(echo "$response" | aush -c "json_get '.status'")
if [ "$status" -ge 400 ]; then
  error=$(echo "$response" | aush -c "json_get '.body.error'")
  echo "API error: $error"
fi
```

---

## Error Responses

When `AUSH_ERROR_FORMAT=json` is set, all errors are returned as structured JSON on stderr.

### Schema

```typescript
{
  error: ErrorType,                // Error type identifier
  message: string,                 // Human-readable error message
  exit_code: number,               // Exit code (non-zero)
  command?: string,                // Command that failed
  context?: {                      // Additional context
    cwd?: string,                  // Current working directory
    shell_pid?: number,            // Shell process ID
    [key: string]: any             // Other context-specific fields
  }
}

type ErrorType =
  | "CommandNotFound"              // Command doesn't exist
  | "PermissionDenied"             // Insufficient permissions
  | "InvalidArgument"              // Invalid command arguments
  | "GitError"                     // Git operation failed
  | "IOError"                      // File I/O error
  | "ParseError"                   // JSON/data parsing error
  | "NetworkError"                 // HTTP request failed
  | "TimeoutError"                 // Operation timed out
  | "UnknownError";                // Unclassified error
```

### Example Errors

**Command not found:**
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

**Git error:**
```json
{
  "error": "GitError",
  "message": "fatal: not a git repository (or any of the parent directories): .git",
  "exit_code": 128,
  "command": "git_status",
  "context": {
    "cwd": "/tmp"
  }
}
```

**Invalid arguments:**
```json
{
  "error": "InvalidArgument",
  "message": "Invalid option: --invalid-flag",
  "exit_code": 2,
  "command": "ls",
  "context": {
    "cwd": "/home/user/project"
  }
}
```

**Network error:**
```json
{
  "error": "NetworkError",
  "message": "Failed to connect to https://api.example.com: Connection refused",
  "exit_code": 1,
  "command": "fetch",
  "context": {
    "url": "https://api.example.com",
    "timeout_ms": 30000
  }
}
```

### Error Handling

```python
import subprocess
import json

def run_rush_safe(command):
    """Run AUSH command with error handling."""
    result = subprocess.run(
        ['aush', '-c', command],
        capture_output=True,
        text=True,
        env={'AUSH_ERROR_FORMAT': 'json'}
    )

    if result.returncode != 0:
        try:
            error = json.loads(result.stderr)
            error_type = error['error']
            message = error['message']

            # Handle specific error types
            if error_type == 'GitError':
                if 'not a git repository' in message:
                    return None  # Not in repo, handle gracefully
            elif error_type == 'NetworkError':
                print(f"Network error, retrying...")
                # Implement retry logic
            elif error_type == 'TimeoutError':
                print(f"Operation timed out")

            raise RuntimeError(f"{error_type}: {message}")
        except json.JSONDecodeError:
            raise RuntimeError(f"Command failed: {result.stderr}")

    return json.loads(result.stdout) if result.stdout else None
```

---

## Quick Reference

### All JSON-Enabled Commands

| Command | Flag | Output |
|---------|------|--------|
| `git_status` | `--json` | Repository status |
| `git_log` | `--json` | Commit history |
| `git_diff` | `--json` | File differences |
| `ls` | `--json` | Directory listing |
| `find` | `--json` | File search results |
| `grep` | `--json` | Content matches |
| `fetch` | `--json` | HTTP response |
| `json_get` | (always JSON) | Extracted values |
| `json_set` | (always JSON) | Modified JSON |
| `json_query` | (always JSON) | Filtered/transformed JSON |

### Common Patterns

**Pipe git status to json processing:**
```bash
aush -c "git_status --json | json_get '.unstaged[].path'"
```

**Combine find and grep:**
```bash
aush -c "find --json src/ -name '*.rs' | json_get '.[].path' | xargs grep --json 'TODO'"
```

**Fetch and extract:**
```bash
aush -c "fetch --json https://api.example.com | json_get '.body.data'"
```

**Complex query:**
```bash
aush -c "git_log --json -n 100 | json_query '.[] | select(.insertions + .deletions > 100)'"
```
