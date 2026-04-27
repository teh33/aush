use aush::executor::Executor;
use aush::lexer::Lexer;
use aush::parser::Parser;

fn execute_script(script: &str) -> Result<aush::executor::ExecutionResult, anyhow::Error> {
    let tokens = Lexer::tokenize(script)?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut executor = Executor::new();
    executor.execute(statements)
}

#[test]
fn test_case_basic_literal_match() {
    let script = r#"
        case foo in
            foo) echo matched;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "matched");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_case_no_match() {
    let script = r#"
        case bar in
            foo) echo matched;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "");
    assert_eq!(result.exit_code, 0); // POSIX: exit code 0 when no match
}

#[test]
fn test_case_wildcard_pattern() {
    let script = r#"
        case hello in
            *) echo default;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "default");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_case_glob_star() {
    let script = r#"
        case file.txt in
            *.txt) echo text file;;
            *) echo other;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "text file");
}

#[test]
fn test_case_glob_question() {
    let script = r#"
        case a in
            ?) echo single char;;
            *) echo other;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "single char");
}

#[test]
fn test_case_multiple_patterns() {
    let script = r#"
        case bar in
            foo|bar|baz) echo matched;;
            *) echo default;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "matched");
}

#[test]
fn test_case_first_match_wins() {
    let script = r#"
        case foo in
            foo) echo first;;
            foo) echo second;;
            *) echo default;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "first");
}

#[test]
fn test_case_with_variable() {
    let script = r#"
        let x = hello
        case $x in
            hello) echo matched variable;;
            *) echo no match;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "matched variable");
}

#[test]
fn test_case_glob_bracket() {
    let script = r#"
        case a in
            [abc]) echo in set;;
            *) echo not in set;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "in set");
}

#[test]
fn test_case_multiple_commands() {
    let script = r#"
        case foo in
            foo)
                echo first
                echo second
                ;;
            *) echo default;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    let stdout = result.stdout();
    let output = stdout.trim();
    assert!(output.contains("first"));
    assert!(output.contains("second"));
}

#[test]
fn test_case_exit_code_from_command() {
    let script = r#"
        case foo in
            foo)
                echo matched
                false
                ;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    // Exit code should be from the last command (false returns 1)
    assert_ne!(result.exit_code, 0);
}

#[test]
fn test_case_nested_in_if() {
    let script = r#"
        let x = bar
        if true {
            case $x in
                bar) echo nested match;;
                *) echo no match;;
            esac
        }
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "nested match");
}

#[test]
fn test_case_pattern_with_dash() {
    let script = r#"
        case test-file in
            test-*) echo matched dash;;
            *) echo no match;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "matched dash");
}

#[test]
fn test_case_empty_body() {
    let script = r#"
        case foo in
            foo) ;;
            *) echo default;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    assert_eq!(result.stdout().trim(), "");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_case_pattern_order_matters() {
    let script = r#"
        case file.txt in
            *) echo wildcard;;
            *.txt) echo text file;;
        esac
    "#;

    let result = execute_script(script).unwrap();
    // First pattern should match
    assert_eq!(result.stdout().trim(), "wildcard");
}
