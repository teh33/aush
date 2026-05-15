use crate::executor::{ExecutionResult, Output};
use crate::runtime::Runtime;
use anyhow::Result;

pub fn builtin_shopt(args: &[String], _runtime: &mut Runtime) -> Result<ExecutionResult> {
    if args.iter().any(|arg| arg == "--help") {
        return Ok(ExecutionResult::success(
            "shopt: shopt [-pqsu] [-o] [optname ...]\n".to_string(),
        ));
    }

    Ok(ExecutionResult {
        output: Output::Text(String::new()),
        stderr: String::new(),
        exit_code: 0,
        error: None,
    })
}
