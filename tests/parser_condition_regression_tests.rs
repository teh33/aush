use aush::{executor::Executor, lexer::Lexer, parser::Parser};

fn run(script: &str) -> anyhow::Result<String> {
    let tokens = Lexer::tokenize(script)?;
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut executor = Executor::new();
    let result = executor.execute(statements)?;
    Ok(result.stdout())
}

#[test]
fn double_bracket_condition_can_feed_and_if() -> anyhow::Result<()> {
    assert_eq!(run("[[ -n abc ]] && echo ok")?, "ok\n");
    Ok(())
}
