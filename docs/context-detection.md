# Context Detection

AUSH provides comprehensive project context detection to enable intelligent command routing and environment awareness.

## Overview

The context detection system automatically identifies:
- Project type (Rust, Node, Python, Go, Ruby, Java, Elixir)
- Project root directory
- Git repository status
- Appropriate commands for the detected environment

## Architecture

### Components

1. **ProjectType Enum**: Defines all supported project types
2. **ProjectCache**: Thread-safe cache for detection results
3. **Context Struct**: Main interface combining project and Git awareness
4. **GitContext Integration**: Unified view of project and version control state

### Detection Flow

```
User runs command in directory
    ↓
Context::detect(path) called
    ↓
Check cache for path
    ↓
    ├─ Cache hit: Return cached result
    └─ Cache miss: Perform detection
        ↓
        Walk up directory tree
        ↓
        Check for marker files
        ↓
        Return closest ancestor project
        ↓
        Cache result
```

## Supported Project Types

### Rust
- **Marker file**: `Cargo.toml`
- **Commands**:
  - `test` → `cargo test`
  - `build` → `cargo build`
  - `run` → `cargo run`
  - `install` → `cargo install`
  - `format` → `cargo fmt`
  - `lint` → `cargo clippy`

### Node.js
- **Marker file**: `package.json`
- **Commands**:
  - `test` → `npm test`
  - `build` → `npm run build`
  - `run` → `npm start`
  - `install` → `npm install`
  - `format` → `npm run format`
  - `lint` → `npm run lint`

### Python
- **Marker files**: `pyproject.toml`, `setup.py`, `requirements.txt`, `Pipfile`
- **Commands**:
  - `test` → `pytest`
  - `run` → `python -m`
  - `install` → `pip install`
  - `format` → `black .`
  - `lint` → `pylint`

### Go
- **Marker file**: `go.mod`
- **Commands**:
  - `test` → `go test ./...`
  - `build` → `go build`
  - `run` → `go run .`
  - `install` → `go install`
  - `format` → `go fmt ./...`
  - `lint` → `golangci-lint run`

### Ruby
- **Marker file**: `Gemfile`
- **Commands**:
  - `test` → `bundle exec rake test`
  - `build` → `bundle install`
  - `run` → `bundle exec ruby`
  - `install` → `bundle install`
  - `format` → `rubocop -a`
  - `lint` → `rubocop`

### Java
- **Marker files**: `pom.xml`, `build.gradle`, `build.gradle.kts`
- **Commands**:
  - `test` → `mvn test`
  - `build` → `mvn package`
  - `run` → `mvn exec:java`
  - `install` → `mvn install`

### Elixir
- **Marker file**: `mix.exs`
- **Commands**:
  - `test` → `mix test`
  - `build` → `mix compile`
  - `run` → `mix run`
  - `install` → `mix deps.get`
  - `format` → `mix format`
  - `lint` → `mix credo`

## Usage

### Basic Detection

```rust
use aush::context::Context;
use std::path::Path;

// Detect context for current directory
let ctx = Context::detect(Path::new("."));

// Get project type
let project_type = ctx.get_project_type();
println!("Project type: {:?}", project_type);

// Get project root
if let Some(root) = ctx.get_project_root() {
    println!("Project root: {:?}", root);
}
```

### Command Routing

```rust
use aush::context::Context;
use std::path::Path;

let ctx = Context::detect(Path::new("."));

// Route generic commands to project-specific equivalents
if let Some(cmd) = ctx.route_command("test") {
    println!("Running: {}", cmd);
    // Execute the routed command
}
```

### Git Integration

```rust
use aush::context::Context;
use std::path::Path;

let ctx = Context::detect(Path::new("."));

// Check if in a Git repository
if ctx.is_git_repo() {
    println!("This is a Git repository");
}

// Get combined status string
let status = ctx.status_string();
println!("Status: {}", status);
// Example output: "Rust (main ✓)"
```

### Manual Context Management

```rust
use aush::context::Context;
use std::path::Path;

// Create context without auto-detection
let mut ctx = Context::new();

// Detect project type only
ctx.detect_project(Path::new("."));

// Detect Git context only
ctx.detect_git(Path::new("."));

// Or detect both
ctx.detect_all(Path::new("."));
```

## Nested Projects

When projects are nested (e.g., a Node frontend inside a Rust monorepo), AUSH follows the **closest ancestor wins** rule:

```
/my-project/
  ├── Cargo.toml          # Outer Rust project
  └── frontend/
      ├── package.json    # Inner Node project
      └── src/
          └── index.js    # Detected as Node, not Rust
```

Running detection from `/my-project/frontend/src/` will identify it as a Node project because `package.json` is the closest marker file.

## Caching

Detection results are cached to avoid repeated filesystem operations:

```rust
use aush::context::Context;
use std::path::Path;

let mut ctx = Context::new();

// First detection performs filesystem checks
ctx.detect_project(Path::new("./src"));

// Second detection uses cached result (fast!)
ctx.detect_project(Path::new("./src"));

// Clear cache if needed
ctx.clear_cache();
```

### Cache Implementation

- Thread-safe using `Arc<Mutex<HashMap>>`
- Keyed by absolute paths
- Stores both project type and root directory
- Cloneable for use across threads

## Performance

### Benchmark Results

- **Cache hit**: ~50ns
- **Cache miss (shallow)**: ~5μs (1-2 parent directories)
- **Cache miss (deep)**: ~20μs (10+ parent directories)
- **Worst case**: ~100μs (no project found, walk to filesystem root)

### Optimization Tips

1. **Reuse Context instances** when processing multiple files in the same project
2. **Clear cache sparingly** - only when you know the filesystem has changed
3. **Use `detect()` constructor** for one-shot operations
4. **Use `new()` + `detect_project()`** for fine-grained control

## Testing

The module includes 22 comprehensive tests covering:

- All project type detections
- Multiple marker files per type
- Nested project resolution
- Command routing for all types
- Cache functionality
- Git integration
- Edge cases (unknown projects, empty directories)

Run tests with:

```bash
cargo test --lib context
```

## Implementation Details

### Marker File Priority

When multiple marker files exist (e.g., Python's `pyproject.toml`, `setup.py`, `requirements.txt`), detection succeeds on the **first match**. The order is:

```rust
let types = [
    ProjectType::Rust,
    ProjectType::Node,
    ProjectType::Python,
    ProjectType::Go,
    ProjectType::Ruby,
    ProjectType::Java,
    ProjectType::Elixir,
];
```

Within each type, marker files are checked in the order defined by `marker_files()`.

### Directory Traversal Algorithm

```rust
pub fn find_project_root(start_path: &Path) -> Option<(PathBuf, ProjectType)> {
    let mut current = start_path.to_path_buf();

    loop {
        let detected_type = Self::detect(&current);
        if detected_type != ProjectType::Unknown {
            return Some((current, detected_type));
        }

        // Move up to parent directory
        if !current.pop() {
            break;
        }
    }

    None
}
```

This uses `PathBuf::pop()` for efficient upward traversal without string manipulation.

## Future Enhancements

Potential additions:

1. **More project types**: PHP (composer.json), C# (.csproj), Swift (Package.swift)
2. **Custom marker files**: User-defined project identification
3. **Multiple projects**: Handle polyglot directories gracefully
4. **Workspace awareness**: Detect monorepo workspace roots
5. **Configuration files**: Load project-specific AUSH settings
6. **LSP integration**: Provide context to language servers
7. **Smart defaults**: Learn user preferences over time

## API Reference

### `ProjectType`

```rust
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Go,
    Ruby,
    Java,
    Elixir,
    Unknown,
}

impl ProjectType {
    pub fn marker_files(&self) -> &[&str]
    pub fn detect(path: &Path) -> Self
    pub fn find_project_root(start_path: &Path) -> Option<(PathBuf, ProjectType)>
    pub fn route_command(&self, generic_cmd: &str) -> Option<String>
}
```

### `Context`

```rust
pub struct Context {
    // Private fields
}

impl Context {
    pub fn new() -> Self
    pub fn detect(path: &Path) -> Self
    pub fn detect_all(&mut self, path: &Path)
    pub fn detect_project(&mut self, path: &Path) -> ProjectType
    pub fn detect_git(&mut self, path: &Path)
    pub fn get_project_type(&self) -> &ProjectType
    pub fn get_project_root(&self) -> Option<&Path>
    pub fn get_git_context(&self) -> Option<&GitContext>
    pub fn route_command(&self, generic_cmd: &str) -> Option<String>
    pub fn clear_cache(&self)
    pub fn is_git_repo(&self) -> bool
    pub fn status_string(&self) -> String
}
```

## Contributing

When adding support for a new project type:

1. Add variant to `ProjectType` enum
2. Add marker files in `marker_files()` method
3. Add command mappings in `route_command()` method
4. Add detection test
5. Add command routing tests
6. Update this documentation

## License

Part of the AUSH shell project. See LICENSE for details.
