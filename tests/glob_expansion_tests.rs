use aush::executor::Executor;
use aush::lexer::Lexer;
use aush::parser::Parser;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_glob_asterisk_expansion() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create test files
    fs::write(temp_path.join("file1.txt"), "content1").unwrap();
    fs::write(temp_path.join("file2.txt"), "content2").unwrap();
    fs::write(temp_path.join("other.md"), "content3").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test *.txt glob pattern
    let tokens = Lexer::tokenize("cat *.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    // Should match both .txt files
    assert!(result.stdout().contains("content1"));
    assert!(result.stdout().contains("content2"));
    assert!(!result.stdout().contains("content3"));
}

#[test]
fn test_glob_question_mark() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create test files
    fs::write(temp_path.join("file1.txt"), "1").unwrap();
    fs::write(temp_path.join("file2.txt"), "2").unwrap();
    fs::write(temp_path.join("file10.txt"), "10").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test file?.txt pattern (should match file1.txt and file2.txt, not file10.txt)
    let tokens = Lexer::tokenize("cat file?.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains('1'));
    assert!(result.stdout().contains('2'));
    assert!(!result.stdout().contains("10"));
}

#[test]
fn test_glob_character_class() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create test files
    fs::write(temp_path.join("file1.txt"), "one").unwrap();
    fs::write(temp_path.join("file2.txt"), "two").unwrap();
    fs::write(temp_path.join("file3.txt"), "three").unwrap();
    fs::write(temp_path.join("file4.txt"), "four").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test file[12].txt pattern (should match only file1 and file2)
    let tokens = Lexer::tokenize("cat file[12].txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains("one"));
    assert!(result.stdout().contains("two"));
    assert!(!result.stdout().contains("three"));
    assert!(!result.stdout().contains("four"));
}

#[test]
fn test_glob_recursive() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create nested directory structure
    fs::create_dir(temp_path.join("dir1")).unwrap();
    fs::create_dir(temp_path.join("dir1/subdir")).unwrap();
    fs::write(temp_path.join("root.txt"), "root").unwrap();
    fs::write(temp_path.join("dir1/level1.txt"), "level1").unwrap();
    fs::write(temp_path.join("dir1/subdir/level2.txt"), "level2").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test **/*.txt recursive pattern
    let tokens = Lexer::tokenize("cat **/*.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    // Should match files at all levels
    assert!(result.stdout().contains("root"));
    assert!(result.stdout().contains("level1"));
    assert!(result.stdout().contains("level2"));
}

#[test]
fn test_glob_dotfiles_not_matched_by_default() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create regular and hidden files
    fs::write(temp_path.join("visible.txt"), "visible").unwrap();
    fs::write(temp_path.join(".hidden.txt"), "hidden").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test * pattern (should not match dotfiles)
    let tokens = Lexer::tokenize("cat *.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains("visible"));
    assert!(!result.stdout().contains("hidden"));
}

#[test]
fn test_glob_dotfiles_matched_explicitly() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create hidden files
    fs::write(temp_path.join(".hidden1"), "hidden1").unwrap();
    fs::write(temp_path.join(".hidden2"), "hidden2").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test .* pattern (should match dotfiles explicitly)
    let tokens = Lexer::tokenize("cat .*").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains("hidden1"));
    assert!(result.stdout().contains("hidden2"));
}

#[test]
fn test_unmatched_glob_uses_posix_literal_fallback() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // POSIX shells leave unmatched glob patterns unchanged. The command may
    // still fail later if it cannot open the literal path.
    let tokens = Lexer::tokenize("printf '%s\\n' *.nonexistent").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout(), "*.nonexistent\n");
}

#[test]
fn test_glob_multiple_patterns() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create files with different extensions
    fs::write(temp_path.join("doc1.txt"), "txt1").unwrap();
    fs::write(temp_path.join("doc2.txt"), "txt2").unwrap();
    fs::write(temp_path.join("readme.md"), "md1").unwrap();
    fs::write(temp_path.join("other.rs"), "rs1").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test multiple glob patterns in single command
    let tokens = Lexer::tokenize("cat *.txt *.md").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    // Should match .txt and .md files, but not .rs
    assert!(result.stdout().contains("txt1"));
    assert!(result.stdout().contains("txt2"));
    assert!(result.stdout().contains("md1"));
    assert!(!result.stdout().contains("rs1"));
}

#[test]
fn test_glob_mixed_with_literals() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create test files
    fs::write(temp_path.join("file1.txt"), "one").unwrap();
    fs::write(temp_path.join("file2.txt"), "two").unwrap();
    fs::write(temp_path.join("specific.txt"), "specific").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Mix glob pattern with literal filename
    let tokens = Lexer::tokenize("cat file*.txt specific.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains("one"));
    assert!(result.stdout().contains("two"));
    assert!(result.stdout().contains("specific"));
}

#[test]
fn test_glob_with_subdirectories() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create subdirectory with files
    fs::create_dir(temp_path.join("subdir")).unwrap();
    fs::write(temp_path.join("subdir/file1.txt"), "sub1").unwrap();
    fs::write(temp_path.join("subdir/file2.txt"), "sub2").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test glob in subdirectory
    let tokens = Lexer::tokenize("cat subdir/*.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains("sub1"));
    assert!(result.stdout().contains("sub2"));
}

#[test]
fn test_glob_sorted_output() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create files in non-alphabetical order
    fs::write(temp_path.join("zebra.txt"), "z").unwrap();
    fs::write(temp_path.join("apple.txt"), "a").unwrap();
    fs::write(temp_path.join("mango.txt"), "m").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Globs should be sorted alphabetically
    let tokens = Lexer::tokenize("cat *.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    // Output should be in alphabetical order: apple, mango, zebra
    let a_pos = result.stdout().find('a').unwrap();
    let m_pos = result.stdout().find('m').unwrap();
    let z_pos = result.stdout().find('z').unwrap();

    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn test_glob_with_builtin_commands() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create test files
    fs::write(temp_path.join("test1.txt"), "content1").unwrap();
    fs::write(temp_path.join("test2.txt"), "content2").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test glob with builtin cat command
    let tokens = Lexer::tokenize("cat *.txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains("content1"));
    assert!(result.stdout().contains("content2"));
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_glob_character_range() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Create files with different numbers
    fs::write(temp_path.join("file1.txt"), "1").unwrap();
    fs::write(temp_path.join("file2.txt"), "2").unwrap();
    fs::write(temp_path.join("file3.txt"), "3").unwrap();
    fs::write(temp_path.join("file5.txt"), "5").unwrap();
    fs::write(temp_path.join("file9.txt"), "9").unwrap();

    let mut executor = Executor::new();
    executor.runtime_mut().set_cwd(temp_path.to_path_buf());

    // Test character range [1-3]
    let tokens = Lexer::tokenize("cat file[1-3].txt").unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    let result = executor.execute(statements).unwrap();

    assert!(result.stdout().contains('1'));
    assert!(result.stdout().contains('2'));
    assert!(result.stdout().contains('3'));
    assert!(!result.stdout().contains('5'));
    assert!(!result.stdout().contains('9'));
}
