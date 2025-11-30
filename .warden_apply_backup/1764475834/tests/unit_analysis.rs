use std::fs;
use tempfile::tempdir;
use warden_core::analysis::RuleEngine;
use warden_core::config::Config;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_js_complexity() -> TestResult {
    let mut config = Config::default();
    config.rules.max_cyclomatic_complexity = 1;
    let engine = RuleEngine::new(config);
    let dir = tempdir()?;
    let file = dir.path().join("test.js");
    
    // if + if = 2 points
    fs::write(&file, "function test() { if(true){} if(true){} }")?;
    
    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    Ok(())
}

#[test]
fn test_python_complexity() -> TestResult {
    let mut config = Config::default();
    config.rules.max_cyclomatic_complexity = 1;
    let engine = RuleEngine::new(config);
    let dir = tempdir()?;
    let file = dir.path().join("test.py");
    
    fs::write(&file, "def test():\n    if True: pass\n    for i in range(1): pass")?;
    
    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    Ok(())
}

#[test]
fn test_snake_case_words() -> TestResult {
    let mut config = Config::default();
    config.rules.max_function_words = 2;
    let engine = RuleEngine::new(config);
    let dir = tempdir()?;
    let file = dir.path().join("naming.rs");
    
    // three_words_func (3 words) > 2
    fs::write(&file, "fn three_words_func() {}")?;
    
    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    assert!(report.violations.iter().any(|v| v.message.contains("doing too much")));
    Ok(())
}

#[test]
fn test_camel_case_words() -> TestResult {
    let mut config = Config::default();
    config.rules.max_function_words = 2;
    let engine = RuleEngine::new(config);
    let dir = tempdir()?;
    let file = dir.path().join("naming.js");
    
    // threeWordsFunc (3 words) > 2
    fs::write(&file, "function threeWordsFunc() {}")?;
    
    let report = engine.scan(vec![file]).files.pop().ok_or("No report")?;
    assert!(!report.is_clean());
    Ok(())
}

#[test]
fn test_warden_ignore_hash() -> TestResult {
    let engine = RuleEngine::new(Config::default());
    let dir = tempdir()?;
    let file = dir.path().join("script.py");
    
    // Python style ignore
    let code = r#"
# warden:ignore
def bad_func():
    pass
    pass
    pass
    "#;
    fs::write(&file, code)?;
    
    let report = engine.scan(vec![file]);
    assert!(report.files.is_empty());
    Ok(())
}

#[test]
fn test_warden_ignore_html() -> TestResult {
    let engine = RuleEngine::new(Config::default());
    let dir = tempdir()?;
    let file = dir.path().join("doc.md");
    
    // HTML style ignore (common in markdown)
    let code = r#"
<!-- warden:ignore -->
# Some Doc
"#;
    fs::write(&file, code)?;
    
    let report = engine.scan(vec![file]);
    assert!(report.files.is_empty());
    Ok(())
}