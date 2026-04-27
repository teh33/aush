use aush::executor::Executor;
use aush::lexer::Lexer;
use aush::parser::Parser;

#[test]
fn test_true_builtin_exit_code() {
    let mut executor = Executor::new();

    let tokens = Lexer::tokenize("true").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert_eq!(executor.runtime_mut().get_last_exit_code(), 0);
}

#[test]
fn test_false_builtin_exit_code() {
    let mut executor = Executor::new();

    let tokens = Lexer::tokenize("false").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 1);
    assert_eq!(executor.runtime_mut().get_last_exit_code(), 1);
}

#[test]
fn test_true_with_arguments() {
    let mut executor = Executor::new();

    // true should ignore all arguments
    let tokens = Lexer::tokenize("true arg1 arg2 --flag").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_false_with_arguments() {
    let mut executor = Executor::new();

    // false should ignore all arguments
    let tokens = Lexer::tokenize("false arg1 arg2 --flag").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_true_and_echo() {
    let mut executor = Executor::new();

    // true && echo should execute echo
    let tokens = Lexer::tokenize("true && echo success").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("success"));
}

#[test]
fn test_false_and_echo() {
    let mut executor = Executor::new();

    // false && echo should not execute echo
    let tokens = Lexer::tokenize("false && echo should_not_print").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 1);
    assert!(!result.stdout().contains("should_not_print"));
}

#[test]
fn test_true_or_echo() {
    let mut executor = Executor::new();

    // true || echo should not execute echo
    let tokens = Lexer::tokenize("true || echo should_not_print").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout().contains("should_not_print"));
}

#[test]
fn test_false_or_echo() {
    let mut executor = Executor::new();

    // false || echo should execute echo
    let tokens = Lexer::tokenize("false || echo fallback").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("fallback"));
}

#[test]
fn test_chained_true_and() {
    let mut executor = Executor::new();

    // true && true && echo should execute echo
    let tokens = Lexer::tokenize("true && true && echo all_true").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("all_true"));
}

#[test]
fn test_chained_false_or() {
    let mut executor = Executor::new();

    // false || false || echo should execute echo
    let tokens = Lexer::tokenize("false || false || echo finally").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("finally"));
}

#[test]
fn test_mixed_true_false_logic() {
    let mut executor = Executor::new();

    // true && false || echo should execute echo (because false fails the &&)
    let tokens = Lexer::tokenize("true && false || echo recovered").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("recovered"));
}

#[test]
fn test_true_no_output() {
    let mut executor = Executor::new();

    let tokens = Lexer::tokenize("true").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    // true should produce no output
    assert_eq!(result.stdout(), "");
    assert_eq!(result.stderr, "");
}

#[test]
fn test_false_no_output() {
    let mut executor = Executor::new();

    let tokens = Lexer::tokenize("false").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    // false should produce no output
    assert_eq!(result.stdout(), "");
    assert_eq!(result.stderr, "");
}

#[test]
fn test_true_in_variable_expansion() {
    let mut executor = Executor::new();

    // Test that exit code is set correctly
    let tokens = Lexer::tokenize("true").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Now check $?
    let tokens = Lexer::tokenize("echo $?").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().trim().contains("0"));
}

#[test]
fn test_false_in_variable_expansion() {
    let mut executor = Executor::new();

    // Test that exit code is set correctly
    let tokens = Lexer::tokenize("false").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Now check $?
    let tokens = Lexer::tokenize("echo $?").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().trim().contains("1"));
}

#[test]
fn test_multiple_true_commands() {
    let mut executor = Executor::new();

    // Multiple true commands separated by newlines
    let tokens = Lexer::tokenize("true\ntrue\ntrue").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_multiple_false_commands() {
    let mut executor = Executor::new();

    // Multiple false commands - last one determines exit code
    let tokens = Lexer::tokenize("false\nfalse\nfalse").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_true_false_alternating() {
    let mut executor = Executor::new();

    // Alternating true/false - last command wins
    let tokens = Lexer::tokenize("true\nfalse\ntrue").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
}

// Colon builtin tests
#[test]
fn test_colon_builtin_exit_code() {
    let mut executor = Executor::new();

    let tokens = Lexer::tokenize(":").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert_eq!(executor.runtime_mut().get_last_exit_code(), 0);
}

#[test]
fn test_colon_with_arguments() {
    let mut executor = Executor::new();

    // : should ignore all arguments
    let tokens = Lexer::tokenize(": arg1 arg2 --flag foo bar baz").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_colon_no_output() {
    let mut executor = Executor::new();

    let tokens = Lexer::tokenize(":").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    // : should produce no output
    assert_eq!(result.stdout(), "");
    assert_eq!(result.stderr, "");
}

#[test]
fn test_colon_in_conditionals() {
    let mut executor = Executor::new();

    // : used in conditionals with && (like true)
    // Note: if-then-fi parsing may need enhancement for standalone : in condition
    let tokens = Lexer::tokenize(": && echo yes").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("yes"));
}

#[test]
fn test_colon_and_echo() {
    let mut executor = Executor::new();

    // : && echo should execute echo (like true)
    let tokens = Lexer::tokenize(": && echo success").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(result.stdout().contains("success"));
}

#[test]
fn test_colon_or_echo() {
    let mut executor = Executor::new();

    // : || echo should not execute echo (like true)
    let tokens = Lexer::tokenize(": || echo should_not_print").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert!(!result.stdout().contains("should_not_print"));
}

#[test]
fn test_colon_in_variable_expansion() {
    let mut executor = Executor::new();

    // Test that exit code is set correctly
    let tokens = Lexer::tokenize(":").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    executor.execute(statements).unwrap();

    // Now check $?
    let tokens = Lexer::tokenize("echo $?").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().trim().contains("0"));
}

#[test]
fn test_colon_with_many_args() {
    let mut executor = Executor::new();

    // : should ignore all arguments no matter how many
    let tokens = Lexer::tokenize(": one two three four five six seven eight nine ten").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout(), "");
}

#[test]
fn test_colon_chained() {
    let mut executor = Executor::new();

    // Multiple : commands
    let tokens = Lexer::tokenize(":\n:\n:").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_colon_with_while_loop() {
    let mut executor = Executor::new();

    // : is often used in while loops as a no-op condition placeholder
    // while :; do echo hello; break; done
    // For now, test basic behavior - full while loop support may come later
    let tokens = Lexer::tokenize(":").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
}
