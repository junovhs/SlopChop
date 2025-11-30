use std::fs;
use tempfile::tempdir;
use warden_core::analysis::RuleEngine;
use warden_core::config::Config;

type TestResult = Result<(), Box<dyn std::error::Error>>;

// --- Law of Atomicity ---

#[test]
fn test_atomicity_clean_file_passes() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("small.rs");
    fs::write(&file, "fn main() { println!(\"Hello\"); }")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(report.is_clean());
    Ok(())
}

#[test]
fn test_atomicity_large_file_fails() -> TestResult {
    let mut config = Config::default();
    config.rules.max_file_tokens = 10;
    let engine = RuleEngine::new(config);
    
    let dir = tempdir()?;
    let file = dir.path().join("large.rs");
    
    // Create > 10 tokens
    let content = "fn main() { } ".repeat(10); 
    fs::write(&file, content)?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    assert!(report.violations.iter().any(|v| v.law == "LAW OF ATOMICITY"));
    Ok(())
}

// --- Law of Complexity ---

#[test]
fn test_complexity_simple_function_passes() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("simple.rs");
    fs::write(&file, "fn simple() { if true { } }")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(report.is_clean());
    Ok(())
}

#[test]
fn test_complexity_branchy_function_fails() -> TestResult {
    let mut config = Config::default();
    config.rules.max_cyclomatic_complexity = 2;
    let engine = RuleEngine::new(config);

    let dir = tempdir()?;
    let file = dir.path().join("complex.rs");
    
    // 3 branches: if, if, if
    let code = r#"
        fn complex() {
            if true {}
            if true {}
            if true {}
        }
    "#;
    fs::write(&file, code)?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    let v = &report.violations[0];
    assert!(v.message.contains("Complexity"));
    Ok(())
}

#[test]
fn test_nesting_shallow_passes() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("shallow.rs");
    fs::write(&file, "fn shallow() { { } }")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(report.is_clean());
    Ok(())
}

#[test]
fn test_nesting_deep_fails() -> TestResult {
    let mut config = Config::default();
    config.rules.max_nesting_depth = 2;
    let engine = RuleEngine::new(config);
    
    let dir = tempdir()?;
    let file = dir.path().join("deep.rs");
    
    // Function (1) -> Block (2) -> Block (3) -> Block (4)
    // Note: depth calculation might count from function body
    let code = "fn deep() { { { { } } } }";
    fs::write(&file, code)?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    assert!(report.violations.iter().any(|v| v.message.contains("Deep Nesting")));
    Ok(())
}

#[test]
fn test_arity_few_args_passes() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("few.rs");
    fs::write(&file, "fn few(a: i32, b: i32) {}")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(report.is_clean());
    Ok(())
}

#[test]
fn test_arity_many_args_fails() -> TestResult {
    let mut config = Config::default();
    config.rules.max_function_args = 2;
    let engine = RuleEngine::new(config);
    
    let dir = tempdir()?;
    let file = dir.path().join("many.rs");
    fs::write(&file, "fn many(a: i32, b: i32, c: i32) {}")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    assert!(report.violations.iter().any(|v| v.message.contains("High Arity")));
    Ok(())
}

// --- Law of Paranoia ---

#[test]
fn test_paranoia_unwrap_fails() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("unwrap.rs");
    fs::write(&file, "fn dangerous() { let x = Some(1).unwrap(); }")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    assert!(report.violations.iter().any(|v| v.law == "LAW OF PARANOIA"));
    Ok(())
}

#[test]
fn test_paranoia_expect_fails() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("expect.rs");
    fs::write(&file, "fn dangerous() { let x = Some(1).expect(\"fail\"); }")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    assert!(report.violations.iter().any(|v| v.message.contains("expect")));
    Ok(())
}

#[test]
fn test_paranoia_no_unwrap_passes() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("safe.rs");
    fs::write(&file, "fn safe() { let x = Some(1).unwrap_or(0); }")?;

    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(report.is_clean());
    Ok(())
}

#[test]
fn test_warden_ignore_skips_file() -> TestResult {
    let (engine, dir) = setup_engine()?;
    let file = dir.path().join("ignored.rs");
    // Even with unwrap, it should pass because of ignore
    let code = r#"
        // warden:ignore
        fn ignore_me() {
            let x = Some(1).unwrap();
        }
    "#;
    fs::write(&file, code)?;

    // Scan should return None for this file, or a report with 0 violations?
    // internal details: analyze_file returns Option<FileReport>. 
    // If ignored, it returns None.
    let reports = engine.scan(vec![file]);
    assert!(reports.files.is_empty());
    Ok(())
}

fn setup_engine() -> Result<(RuleEngine, tempfile::TempDir), Box<dyn std::error::Error>> {
    let config = Config::default();
    let dir = tempdir()?;
    Ok((RuleEngine::new(config), dir))
}