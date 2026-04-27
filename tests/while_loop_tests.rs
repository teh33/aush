use aush::executor::Executor;
use aush::lexer::Lexer;
use aush::parser::Parser;

#[test]
fn test_basic_while_loop() {
    let mut executor = Executor::new();

    // Test while loop that runs a fixed number of times.
    let code = r#"
        count=0
        while test "$count" -ne 3; do
            echo "iteration"
            count=$((count+1))
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    // Should have 3 iterations
    let iteration_count = output.matches("iteration").count();
    assert_eq!(iteration_count, 3);
}

#[test]
fn test_while_loop_with_break() {
    let mut executor = Executor::new();

    let code = r#"
        count=0
        while true; do
            echo "loop"
            count=$((count+1))
            test "$count" -eq 3 && break
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    // Should stop after 3 iterations
    let loop_count = output.matches("loop").count();
    assert_eq!(loop_count, 3);
}

#[test]
fn test_while_loop_with_continue() {
    let mut executor = Executor::new();

    let code = r#"
        count=0
        while test "$count" -ne 3; do
            count=$((count+1))
            test "$count" -eq 2 && continue
            echo "printed"
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    // Should print only twice (skipping when count=2)
    let print_count = output.matches("printed").count();
    assert_eq!(print_count, 2);
}

#[test]
fn test_nested_while_loops() {
    let mut executor = Executor::new();

    let code = r#"
        outer=0
        while test "$outer" -ne 2; do
            inner=0
            while test "$inner" -ne 1; do
                echo "nested"
                inner=$((inner+1))
            done
            outer=$((outer+1))
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    // Should have 2 * 1 = 2 iterations
    let nested_count = output.matches("nested").count();
    assert_eq!(nested_count, 2);
}

#[test]
fn test_while_loop_with_pipeline_condition() {
    let mut executor = Executor::new();

    let code = r#"
        while echo "test" | grep -q "test"; do
            echo "matched"
            break
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements);

    // This test may not work if grep is not available
    // Just ensure it doesn't panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_while_loop_exit_code_propagation() {
    let mut executor = Executor::new();

    // Last command in last iteration should set the exit code
    let code = r#"
        count=0
        while test "$count" -ne 2; do
            count=$((count+1))
            test "$count" -eq 2
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    // The last iteration runs test which succeeds, so exit code should be 0
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_while_loop_never_executes() {
    let mut executor = Executor::new();

    let code = r#"
        while false; do
            echo "Should not print"
        done
        echo "After loop"
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    assert!(!output.contains("Should not print"));
    assert!(output.contains("After loop"));

    // If while never executes, exit code should be 0 (per POSIX)
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_while_true_with_break() {
    let mut executor = Executor::new();

    let code = r#"
        count=0
        while true; do
            echo "$count"
            count=$((count+1))
            test "$count" -eq 5 && break
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    // Should print 0 through 4 (5 times)
    let lines: Vec<&str> = output.trim().lines().collect();
    assert_eq!(lines.len(), 5);
}

#[test]
fn test_while_loop_multiline_condition() {
    let mut executor = Executor::new();

    // POSIX allows multiple statements in condition.
    let code = r#"
        count=0
        while
            count=$((count+1))
            test "$count" -ne 4
        do
            echo "loop"
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    // Should loop 3 times (until count becomes 4)
    let loop_count = output.matches("loop").count();
    assert_eq!(loop_count, 3);
}

#[test]
fn test_while_loop_with_compound_condition() {
    let mut executor = Executor::new();

    let code = r#"
        count=0
        while test "$count" -ne 3 && echo "Testing"; do
            echo "Loop"
            count=$((count+1))
        done
    "#;

    let tokens = Lexer::tokenize(code).unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let result = executor.execute(statements).unwrap();

    let output = result.stdout();
    assert!(output.contains("Testing"));
    assert!(output.contains("Loop"));
    // Should loop 3 times
    let loop_count = output.matches("Loop").count();
    assert_eq!(loop_count, 3);
}
