# AUSH Architecture

## Overview

AUSH is built as a modular shell with clear separation of concerns. Each component has a specific responsibility and communicates through well-defined interfaces.

## Component Details

### Lexer (`src/lexer/`)

**Purpose**: Convert raw input strings into tokens.

**Implementation**: Uses the `logos` crate for efficient lexer generation via procedural macros.

**Token Types**:
- Keywords: `let`, `if`, `else`, `fn`, `match`, `for`, `in`
- Operators: `=`, `==`, `!=`, `>`, `<`, `>=`, `<=`, `|`, `&&`, `||`
- Literals: Strings, integers, floats
- Identifiers: Command names, variable names
- Special: Pipes, redirects, flags, paths

**Performance**:
- Zero-copy lexing where possible
- Lazy evaluation of tokens
- Minimal allocations

### Parser (`src/parser/`)

**Purpose**: Transform token stream into an Abstract Syntax Tree (AST).

**AST Nodes**:
```rust
Statement
├── Command        # Simple command execution
├── Pipeline       # Chained commands (cmd1 | cmd2)
├── Assignment     # Variable assignment (let x = value)
├── FunctionDef    # Function definition
├── IfStatement    # Conditional execution
├── ForLoop        # Iteration
└── MatchExpression # Pattern matching
```

**Design Philosophy**:
- Recursive descent parser
- Error recovery (planned)
- Position tracking for error messages (planned)

### Executor (`src/executor/`)

**Purpose**: Execute AST nodes and produce results.

**Execution Flow**:
```
AST → Executor → Runtime + Builtins → External Commands
                     ↓
                ExecutionResult
```

**Features**:
- Built-in command detection
- External command spawning
- Pipeline execution with proper stdin/stdout chaining
- Variable interpolation
- Environment management

**Pipeline Execution**:
- Commands are executed sequentially
- stdout of command N → stdin of command N+1
- Proper process management with tokio (planned for async)

### Runtime (`src/runtime/`)

**Purpose**: Manage execution context (variables, functions, environment).

**State Management**:
- Variable storage (HashMap)
- Function registry
- Current working directory
- Environment variables (delegates to std::env)

**Scope Rules** (planned for future):
- Global scope
- Function scope
- Block scope

### Builtins (`src/builtins/`)

**Purpose**: Implement shell built-in commands that can't be external programs.

**Current Builtins**:

| Command | Description | Implementation |
|---------|-------------|----------------|
| `cd` | Change directory | Direct filesystem manipulation |
| `pwd` | Print working directory | Read from runtime |
| `echo` | Print arguments | Simple string concatenation |
| `exit` | Exit shell | Process termination |
| `export` | Set environment variables | Updates runtime + std::env |

**Design Pattern**:
```rust
type BuiltinFn = fn(&[String], &mut Runtime) -> Result<ExecutionResult>;
```

Each builtin is a pure function that takes arguments and runtime, returns a result.

### Completion (`src/completion/`)

**Purpose**: Provide tab completion (TODO).

**Planned Features**:
- Command name completion
- Flag completion (context-aware)
- Path completion (.gitignore-aware)
- Git branch completion
- Project-specific completion (cargo commands, npm scripts)

### History (`src/history/`)

**Purpose**: Manage command history (TODO).

**Planned Features**:
- Persistent history file (~/.aush_history)
- Fuzzy search (Ctrl+R)
- History deduplication
- Timestamp tracking

### Context (`src/context/`)

**Purpose**: Detect and maintain project context (TODO).

**Planned Features**:
- Project type detection (Rust, Node, Python, Go)
- Git repository detection
- Git status caching
- Smart command routing (test → cargo test, npm test, etc.)

### Output (`src/output/`)

**Purpose**: Format output in text or JSON (TODO).

**Planned Modes**:
- Text mode (default): Human-readable
- JSON mode (`--json`): Machine-parseable
- Pretty mode: Syntax highlighting

## Data Flow

### Simple Command Execution

```
User Input: "echo hello"
    ↓
Lexer: [Identifier("echo"), Identifier("hello")]
    ↓
Parser: Statement::Command { name: "echo", args: ["hello"] }
    ↓
Executor: Detects builtin "echo"
    ↓
Builtins::echo: Concatenates args
    ↓
ExecutionResult: { stdout: "hello\n", stderr: "", exit_code: 0 }
    ↓
REPL: Prints stdout
```

### Pipeline Execution

```
User Input: "ls | grep foo"
    ↓
Lexer: [Identifier("ls"), Pipe, Identifier("grep"), Identifier("foo")]
    ↓
Parser: Statement::Pipeline {
    commands: [
        Command { name: "ls", args: [] },
        Command { name: "grep", args: ["foo"] }
    ]
}
    ↓
Executor::execute_pipeline:
    1. Execute "ls" → capture stdout
    2. Execute "grep foo" with stdin = ls's stdout
    ↓
ExecutionResult: { stdout: "filtered results", ... }
```

### Variable Assignment

```
User Input: "let x = 42"
    ↓
Lexer: [Let, Identifier("x"), Equals, Integer(42)]
    ↓
Parser: Statement::Assignment {
    name: "x",
    value: Expression::Literal(Integer(42))
}
    ↓
Executor: Evaluates expression → "42"
    ↓
Runtime: Stores x → "42" in variables HashMap
```

## Performance Considerations

### Current Optimizations

1. **Zero-copy lexing**: Logos operates on string slices
2. **Minimal allocations**: Reuse buffers where possible
3. **Direct execution**: No intermediate bytecode

### Planned Optimizations (Phase 2+)

1. **Built-in commands**: Rust implementations of ls, grep, find
   - `ls`: Direct readdir syscalls, parallel sorting
   - `grep`: ripgrep integration (10-50x faster)
   - `find`: ignore crate (.gitignore-aware)

2. **Git integration**: Native git2 bindings (5-10x faster than git CLI)

3. **Smart caching**:
   - Git status caching
   - Completion cache
   - Path resolution cache

## Error Handling

### Current Strategy

- `anyhow::Result` for flexible error propagation
- User-facing error messages via REPL
- Parser errors include position (planned)

### Planned Improvements

- Error recovery in parser
- Suggestions for typos
- Better error context ("Did you mean X?")
- Error codes for scripting

## Testing Strategy

### Unit Tests

Each module has its own test suite:
- Lexer: Token recognition
- Parser: AST construction
- Builtins: Command behavior
- Executor: Integration tests

### Integration Tests

- End-to-end command execution
- Pipeline behavior
- Error cases

### Benchmarks (Planned)

```bash
# Startup time
hyperfine 'aush -c exit' 'bash -c exit' 'zsh -c exit'

# Built-in performance
hyperfine 'aush -c "ls"' 'bash -c "ls"'
```

## Future Architecture

### Phase 2: Performance

- Fast built-in commands (Rust implementations)
- Memory-mapped file operations
- Parallel execution where safe

### Phase 3: Ergonomics

- Git integration (native bindings)
- Project context awareness
- Advanced tab completion

### Phase 4: Automation

- JSON output mode
- Structured logging
- Operation undo capability

## Design Principles

1. **Unix Philosophy**: Text streams by default, structured data opt-in
2. **Performance First**: Sub-10ms startup, <10MB memory
3. **Safety**: Leverage Rust's type system and ownership
4. **Modularity**: Clear component boundaries
5. **Testability**: Every component is independently testable
6. **Simplicity**: Prefer obvious code over clever code

---

Last updated: 2026-01-20
