//! Integration tests for the getopts builtin.
//!
//! Tests POSIX-compliant option parsing for shell scripts.

use aush::executor::Executor;
use aush::lexer::Lexer;
use aush::parser::Parser;

fn execute_line(line: &str, executor: &mut Executor) -> Result<String, String> {
    let tokens = Lexer::tokenize(line).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().map_err(|e| e.to_string())?;
    let result = executor.execute(statements).map_err(|e| e.to_string())?;
    Ok(result.stdout().trim().to_string())
}

fn execute_line_with_code(line: &str, executor: &mut Executor) -> Result<(String, i32), String> {
    let tokens = Lexer::tokenize(line).map_err(|e| e.to_string())?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().map_err(|e| e.to_string())?;
    let result = executor.execute(statements).map_err(|e| e.to_string())?;
    Ok((result.stdout().trim().to_string(), result.exit_code))
}

fn execute_script(script: &str, executor: &mut Executor) -> Result<String, String> {
    let mut output = String::new();
    for line in script.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let tokens = Lexer::tokenize(line).map_err(|e| e.to_string())?;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse().map_err(|e| e.to_string())?;
        let result = executor.execute(statements).map_err(|e| e.to_string())?;
        if !result.stdout().is_empty() {
            output.push_str(&result.stdout());
        }
    }
    Ok(output.trim().to_string())
}

// ============================================================================
// Basic getopts tests
// ============================================================================

#[test]
fn test_getopts_simple_option() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-a".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    let (_, code) = execute_line_with_code("getopts 'a' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );
}

#[test]
fn test_getopts_multiple_options() {
    let mut executor = Executor::new();
    executor.runtime_mut().set_positional_params(vec![
        "-a".to_string(),
        "-b".to_string(),
        "-c".to_string(),
    ]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First call gets 'a'
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );

    // Second call gets 'b'
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("b".to_string())
    );

    // Third call gets 'c'
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("c".to_string())
    );

    // Fourth call returns 1 (no more options)
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 1);
}

#[test]
fn test_getopts_bundled_options() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-abc".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First call gets 'a'
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );

    // Second call gets 'b'
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("b".to_string())
    );

    // Third call gets 'c'
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("c".to_string())
    );

    // Fourth call returns 1
    let (_, code) = execute_line_with_code("getopts 'abc' opt", &mut executor).unwrap();
    assert_eq!(code, 1);
}

// ============================================================================
// Options with arguments
// ============================================================================

#[test]
fn test_getopts_option_with_argument_separate() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-f".to_string(), "value".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    let (_, code) = execute_line_with_code("getopts 'f:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("f".to_string())
    );
    assert_eq!(
        executor.runtime_mut().get_variable("OPTARG"),
        Some("value".to_string())
    );
}

#[test]
fn test_getopts_option_with_argument_attached() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-fvalue".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    let (_, code) = execute_line_with_code("getopts 'f:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("f".to_string())
    );
    assert_eq!(
        executor.runtime_mut().get_variable("OPTARG"),
        Some("value".to_string())
    );
}

#[test]
fn test_getopts_mixed_bundled_with_arg() {
    let mut executor = Executor::new();
    // -avfvalue: -a, -v, then -f with argument "value"
    executor
        .runtime_mut()
        .set_positional_params(vec!["-avfvalue".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First: 'a'
    let (_, code) = execute_line_with_code("getopts 'avf:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );

    // Second: 'v'
    let (_, code) = execute_line_with_code("getopts 'avf:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("v".to_string())
    );

    // Third: 'f' with OPTARG="value"
    let (_, code) = execute_line_with_code("getopts 'avf:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("f".to_string())
    );
    assert_eq!(
        executor.runtime_mut().get_variable("OPTARG"),
        Some("value".to_string())
    );
}

// ============================================================================
// OPTIND tracking
// ============================================================================

#[test]
fn test_getopts_optind_tracking() {
    let mut executor = Executor::new();
    executor.runtime_mut().set_positional_params(vec![
        "-a".to_string(),
        "-b".to_string(),
        "arg1".to_string(),
        "arg2".to_string(),
    ]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // Process -a
    execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    // Process -b
    execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    // No more options
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 1);

    // OPTIND should be 3 (pointing to "arg1")
    assert_eq!(
        executor.runtime_mut().get_variable("OPTIND"),
        Some("3".to_string())
    );
}

// ============================================================================
// Unknown options and error handling
// ============================================================================

#[test]
fn test_getopts_unknown_option() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-x".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 0); // Still returns 0, but sets opt='?'
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("?".to_string())
    );
}

#[test]
fn test_getopts_silent_mode_unknown() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-x".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // Optstring starts with ':' for silent mode
    let (_, code) = execute_line_with_code("getopts ':ab' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("?".to_string())
    );
    assert_eq!(
        executor.runtime_mut().get_variable("OPTARG"),
        Some("x".to_string())
    );
}

#[test]
fn test_getopts_missing_required_arg() {
    let mut executor = Executor::new();
    // -f requires an argument but none provided
    executor
        .runtime_mut()
        .set_positional_params(vec!["-f".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    let (_, code) = execute_line_with_code("getopts 'f:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("?".to_string())
    );
}

#[test]
fn test_getopts_silent_mode_missing_arg() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-f".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // Silent mode with required argument missing
    let (_, code) = execute_line_with_code("getopts ':f:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some(":".to_string())
    );
    assert_eq!(
        executor.runtime_mut().get_variable("OPTARG"),
        Some("f".to_string())
    );
}

// ============================================================================
// Special cases
// ============================================================================

#[test]
fn test_getopts_double_dash_ends_options() {
    let mut executor = Executor::new();
    executor.runtime_mut().set_positional_params(vec![
        "-a".to_string(),
        "--".to_string(),
        "-b".to_string(),
    ]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First call gets 'a'
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );

    // Second call hits -- and returns 1
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 1);
}

#[test]
fn test_getopts_non_option_stops_parsing() {
    let mut executor = Executor::new();
    executor.runtime_mut().set_positional_params(vec![
        "-a".to_string(),
        "arg".to_string(),
        "-b".to_string(),
    ]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First call gets 'a'
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 0);

    // Second call hits "arg" (non-option) and returns 1
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 1);
}

#[test]
fn test_getopts_single_dash_is_not_option() {
    let mut executor = Executor::new();
    executor.runtime_mut().set_positional_params(vec![
        "-a".to_string(),
        "-".to_string(),
        "-b".to_string(),
    ]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First: 'a'
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 0);

    // Second: "-" is not an option, stops parsing
    let (_, code) = execute_line_with_code("getopts 'ab' opt", &mut executor).unwrap();
    assert_eq!(code, 1);
}

#[test]
fn test_getopts_with_explicit_args() {
    let mut executor = Executor::new();
    // Positional params are ignored when explicit args provided
    executor
        .runtime_mut()
        .set_positional_params(vec!["ignored".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // Pass explicit args after varname
    let (_, code) = execute_line_with_code("getopts 'ab' opt -a -b", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );
}

#[test]
fn test_getopts_reset_optind() {
    let mut executor = Executor::new();
    executor
        .runtime_mut()
        .set_positional_params(vec!["-a".to_string()]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // First pass
    let (_, code) = execute_line_with_code("getopts 'a' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    let (_, code) = execute_line_with_code("getopts 'a' opt", &mut executor).unwrap();
    assert_eq!(code, 1);

    // Reset OPTIND to parse again
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // Second pass
    let (_, code) = execute_line_with_code("getopts 'a' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("a".to_string())
    );
}

// ============================================================================
// Real-world usage pattern
// ============================================================================

#[test]
fn test_getopts_typical_script_pattern() {
    let mut executor = Executor::new();
    executor.runtime_mut().set_positional_params(vec![
        "-v".to_string(),
        "-o".to_string(),
        "output.txt".to_string(),
        "input.txt".to_string(),
    ]);
    executor
        .runtime_mut()
        .set_variable("OPTIND".to_string(), "1".to_string());

    // Parse -v
    let (_, code) = execute_line_with_code("getopts 'vo:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("v".to_string())
    );

    // Parse -o output.txt
    let (_, code) = execute_line_with_code("getopts 'vo:' opt", &mut executor).unwrap();
    assert_eq!(code, 0);
    assert_eq!(
        executor.runtime_mut().get_variable("opt"),
        Some("o".to_string())
    );
    assert_eq!(
        executor.runtime_mut().get_variable("OPTARG"),
        Some("output.txt".to_string())
    );

    // No more options
    let (_, code) = execute_line_with_code("getopts 'vo:' opt", &mut executor).unwrap();
    assert_eq!(code, 1);

    // OPTIND should point to "input.txt" (index 4)
    assert_eq!(
        executor.runtime_mut().get_variable("OPTIND"),
        Some("4".to_string())
    );
}
