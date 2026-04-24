# AUSH Example Scripts

This directory contains practical example scripts demonstrating AUSH's capabilities for AI agents and automation.

## Examples

### 1. commit_message_generator.aush
Generate intelligent commit messages from staged changes.

**Usage:**
```bash
./commit_message_generator.aush
```

**Features:**
- Analyzes staged changes
- Determines commit type (feat/fix/docs/etc.)
- Shows file changes and statistics
- Generates conventional commit messages

---

### 2. find_todos.aush
Find all TODO/FIXME comments in codebase.

**Usage:**
```bash
./find_todos.aush [directory]
```

**Features:**
- Searches for TODO and FIXME comments
- Shows file and line number for each
- Provides summary statistics
- Helps track technical debt

---

### 3. code_review_prep.aush
Prepare comprehensive code review summaries.

**Usage:**
```bash
./code_review_prep.aush [base_branch]
```

**Features:**
- Lists all commits in branch
- Shows files changed with stats
- Highlights new TODOs introduced
- Includes review checklist

---

### 4. test_coverage_analyzer.aush
Analyze test coverage by finding files without tests.

**Usage:**
```bash
./test_coverage_analyzer.aush [src_dir] [test_dir]
```

**Features:**
- Finds source files without corresponding tests
- Lists public functions without tests
- Calculates test file ratio
- Provides coverage recommendations

---

### 5. dead_code_finder.aush
Find potentially unused code (functions/types not referenced).

**Usage:**
```bash
./dead_code_finder.aush [directory]
```

**Features:**
- Finds functions with no references
- Identifies unused structs/enums
- Helps reduce code bloat
- Simple heuristic analysis

---

### 6. security_audit.aush
Basic security audit for common issues.

**Usage:**
```bash
./security_audit.aush [directory]
```

**Features:**
- Finds hardcoded passwords/secrets
- Identifies API keys and tokens
- Lists unsafe code blocks
- Detects unwrap() and panic!() usage
- Shows security-related TODOs

---

### 7. performance_profiler.aush
Profile git operations performance.

**Usage:**
```bash
./performance_profiler.aush
```

**Features:**
- Benchmarks git_status, git_log, git_diff
- Shows average execution times
- Provides repository statistics
- Demonstrates AUSH performance benefits

---

### 8. branch_cleaner.aush
Find and list branches that have been merged.

**Usage:**
```bash
./branch_cleaner.aush
```

**Features:**
- Lists all local branches
- Identifies merged branches
- Shows ahead/behind status
- Generates cleanup commands

---

### 9. changelog_generator.aush
Generate changelogs from git commits.

**Usage:**
```bash
./changelog_generator.aush [since_tag]
```

**Features:**
- Categorizes commits by type
- Supports conventional commits
- Shows statistics
- Formats as markdown

---

### 10. dependency_check.aush
Check for dependency updates (demonstrates API usage).

**Usage:**
```bash
./dependency_check.aush
```

**Features:**
- Demonstrates fetch command
- Queries crates.io API
- Shows version information
- Educational example for API integration

---

### 11. file_stats.aush
Analyze file statistics in a directory.

**Usage:**
```bash
./file_stats.aush [directory]
```

**Features:**
- Total files and size
- Largest files listing
- File type distribution
- Recently modified files

---

### 12. git_author_stats.aush
Analyze git commit statistics by author.

**Usage:**
```bash
./git_author_stats.aush [commit_count]
```

**Features:**
- Commits by author
- Code contributions (lines changed)
- Recent activity timeline
- Contribution metrics

---

## Running Examples

All examples are executable AUSH scripts. Make them executable and run directly:

```bash
chmod +x examples/*.aush
./examples/commit_message_generator.aush
```

Or run with AUSH:

```bash
aush examples/commit_message_generator.aush
```

## AI Agent Integration

These examples demonstrate patterns useful for AI coding agents:

1. **JSON Processing**: All examples use AUSH's native JSON operations
2. **Error Handling**: Proper error checking and reporting
3. **Git Integration**: Leveraging AUSH's fast git builtins
4. **File Operations**: Efficient file searching and processing
5. **HTTP Requests**: API integration with fetch command

## Learning Path

**Beginners:** Start with:
- file_stats.aush
- find_todos.aush
- git_author_stats.aush

**Intermediate:** Progress to:
- commit_message_generator.aush
- code_review_prep.aush
- changelog_generator.aush

**Advanced:** Try:
- test_coverage_analyzer.aush
- dead_code_finder.aush
- security_audit.aush

## Extending Examples

Feel free to modify these examples for your specific needs. Common extensions:

- Add more sophisticated analysis
- Integrate with external tools
- Customize output formats
- Add configuration files
- Implement additional filters

## Contributing

Found a bug or have an improvement? Examples welcome!

1. Keep examples focused and practical
2. Include usage documentation
3. Test thoroughly
4. Follow AUSH best practices
5. Add to this README

## Resources

- [AI Agent Integration Guide](../docs/AI_AGENT_GUIDE.md)
- [JSON Schema Reference](../docs/AI_AGENT_JSON_REFERENCE.md)
- [AUSH Documentation](../docs/)

---

**All examples are MIT licensed and free to use, modify, and distribute.**
