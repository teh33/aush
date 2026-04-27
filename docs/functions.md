# User-Defined Functions in AUSH

## Overview

AUSH supports user-defined functions with parameters, local scoping, and recursion. Functions are first-class constructs that can call other functions and return values through their stdout.

## Implementation Details

### Architecture

The function calling implementation consists of three main components:

1. **Runtime Scope Management** (`src/runtime/mod.rs`)
   - Maintains a stack of variable scopes for function-local variables
   - Tracks the call stack to prevent infinite recursion
   - Maximum recursion depth: 100 calls

2. **Function Execution** (`src/executor/mod.rs`)
   - Checks for user-defined functions before builtins and external commands
   - Binds arguments to parameters by position
   - Executes function body statements in order
   - Returns accumulated stdout from all statements

3. **AST Support** (`src/parser/ast.rs`)
   - `FunctionDef`: Stores function name, parameters, and body
   - `FunctionCall`: Expression type for calling functions in expressions
   - `Parameter`: Represents function parameters with optional type hints

### Execution Flow

When a command is executed:

1. Check if command name matches a user-defined function
2. If yes, call `execute_user_function()`
3. If no, check builtins, then external commands

When executing a user function:

1. Clone the function definition from runtime
2. Push function name onto call stack (error if depth exceeded)
3. Push a new scope onto the scope stack
4. Bind arguments to parameters (by position)
5. Execute each statement in the function body
6. Accumulate stdout/stderr from all statements
7. Pop scope and call stack
8. Return the accumulated result

### Scope Management

#### Variable Resolution

Variables are resolved in the following order:

1. Current function scope (most recent on stack)
2. Parent function scopes (stack traversal)
3. Global variables

#### Variable Assignment

Variables are assigned based on scope:

- Inside a function: Variable is set in the current function scope
- Outside a function: Variable is set in the global scope

This ensures:
- Function parameters shadow outer variables
- Local variables don't leak to outer scope
- Functions can access global variables if not shadowed

### Recursion Control

The runtime maintains a call stack with a maximum depth of 100:

```rust
const MAX_CALL_STACK_DEPTH: usize = 100; // In practice: 100
```

When the limit is exceeded, an error is returned:
```
Maximum recursion depth exceeded (100)
```

This prevents stack overflow from infinite recursion.

## Usage Examples

### Simple Function (No Parameters)

```sh
fn greet() {
    echo "Hello, World!"
}

greet  # Output: Hello, World!
```

### Function with Parameters

```sh
fn say(message) {
    echo $message
}

say "Hello"  # Output: Hello
```

### Multiple Parameters

```sh
fn add(a, b) {
    # In a real implementation, this would do math
    echo "$a + $b"
}

add 5 10  # Output: 5 + 10
```

### Functions Calling Other Functions

```sh
fn helper() {
    echo "Helper function"
}

fn main() {
    echo "Main function"
    helper
}

main
# Output:
# Main function
# Helper function
```

### Recursive Functions

```sh
fn countdown(n) {
    echo $n
    if [ $n -gt 0 ]; then
        countdown $(($n - 1))
    fi
}

countdown 5
# Output:
# 5
# 4
# 3
# 2
# 1
# 0
```

### Scope Isolation

```sh
x="global"

fn test_scope(x) {
    echo $x  # Uses parameter, not global
}

test_scope "local"  # Output: local
echo $x             # Output: global
```

### Local Variables Don't Leak

```sh
fn set_local() {
    local_var="I'm local"
}

set_local
echo $local_var  # Output: (empty - variable doesn't exist)
```

## Return Values

Functions return values through their stdout. The last statement's exit code is used as the function's exit code.

```sh
fn get_value() {
    echo "42"
}

result=$(get_value)
echo $result  # Output: 42
```

All statements' stdout is accumulated:

```sh
fn multi_output() {
    echo "line1"
    echo "line2"
    echo "line3"
}

result=$(multi_output)
# result contains all three lines
```

## Function Call Expressions

Functions can be called from expressions:

```rust
Expression::FunctionCall(FunctionCall {
    name: "my_func".to_string(),
    args: vec![
        Expression::Literal(Literal::String("arg1".to_string())),
    ],
})
```

The function is executed and its stdout is returned as the expression value.

## Testing

The implementation includes comprehensive tests in `tests/function_calling_test.rs`:

1. ✅ Simple function call (no parameters)
2. ✅ Function with single parameter
3. ✅ Function with multiple parameters
4. ✅ Recursive countdown function
5. ✅ Functions calling other functions
6. ✅ Scope isolation (parameters shadow variables)
7. ✅ Local variables don't leak to outer scope
8. ✅ Recursion depth limit enforcement
9. ✅ Function return values
10. ✅ Function stdout accumulation

All 10 tests pass successfully.

## Implementation Notes

### Cloning for Borrow Safety

The function definition is cloned before execution to avoid borrow checker issues:

```rust
let func = self.runtime.get_function(name)?.clone();
```

This allows the executor to mutate the runtime while executing the function body.

### stdout Accumulation

All statements in a function body have their stdout accumulated:

```rust
for statement in func.body {
    let stmt_result = self.execute_statement(statement)?;
    last_result.stdout.push_str(&stmt_result.stdout);
    last_result.stderr.push_str(&stmt_result.stderr);
    last_result.exit_code = stmt_result.exit_code;
}
```

This ensures that functions with multiple echo statements capture all output.

### Default Argument Values

If fewer arguments are provided than parameters, missing arguments default to empty strings:

```rust
let arg_value = args.get(i).cloned().unwrap_or_default();
```

## Future Enhancements

Potential improvements for future versions:

1. **Explicit Return Statement**: Add a `return` keyword to exit early
2. **Named Arguments**: Support `func(x=5, y=10)` syntax
3. **Default Parameter Values**: `fn func(x, y=10)`
4. **Variadic Functions**: `fn func(args...)`
5. **Higher Recursion Limit**: Make configurable or increase default
6. **Closure Support**: Capture variables from outer scope
7. **Function Types**: Type checking for parameters and return values
8. **Anonymous Functions**: Lambda/closure syntax

## Performance Considerations

- Functions are stored in a HashMap for O(1) lookup
- Scope stack operations are O(1) push/pop
- Variable resolution is O(n) where n = scope depth
- Function definition cloning has overhead proportional to body size

For most use cases, performance is excellent. Deep recursion or very large function bodies may have noticeable overhead.

## Error Handling

Common errors:

1. **Function not found**: When calling undefined function
2. **Maximum recursion depth exceeded**: When recursion limit hit
3. **Statement execution errors**: Propagated from function body

All errors use the `anyhow` crate for comprehensive error reporting.
