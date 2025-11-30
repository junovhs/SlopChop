// tests/integration_core.rs
//! Integration tests for the 3 Laws enforcement.

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::{Config, RuleConfig};
use warden_core::types::Violation;

// --- Helpers ---

fn strict_rules() -> RuleConfig {
    RuleConfig {
        max_file_tokens: 50,
        max_cyclomatic_complexity: 2,
        max_nesting_depth: 1,
        max_function_args: 2,
        max_function_words: 3,
        ignore_naming_on: vec![],
        ignore_tokens_on: vec![],
    }
}

fn scan_with_rules(content: &str, rules: RuleConfig) -> Result<Vec<Violation>> {
    let dir = TempDir::new()?;
    let file_path = dir.path().join("test.rs");
    let mut file = File::create(&file_path)?;
    write!(file, "{content}")?;

    let mut config = Config::new();
    config.rules = rules;

    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    Ok(report
        .files
        .into_iter()
        .flat_map(|f| f.violations)
        .collect())
}

// --- Law of Atomicity ---

#[test]
fn test_atomicity_clean_file_passes() -> Result<()> {
    let content = "fn main() { println!(\"Small file\"); }";
    let violations = scan_with_rules(content, strict_rules())?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn test_atomicity_large_file_fails() -> Result<()> {
    // Generate content > 50 tokens
    let content = "fn main() { let x = 1; } ".repeat(20);
    let violations = scan_with_rules(&content, strict_rules())?;

    assert!(!violations.is_empty());
    assert!(violations[0].message.contains("File size"));
    Ok(())
}

// --- Law of Complexity ---

#[test]
fn test_complexity_simple_function_passes() -> Result<()> {
    let content = "fn simple() { if true { println!(\"ok\"); } }";
    let violations = scan_with_rules(content, strict_rules())?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn test_complexity_branchy_function_fails() -> Result<()> {
    // Complexity > 2
    let content = r"
        fn complex() {
            if true {}
            if true {}
            if true {}
        }
    ";
    let violations = scan_with_rules(content, strict_rules())?;

    assert!(violations
        .iter()
        .any(|v| v.message.contains("High Complexity")));
    Ok(())
}

#[test]
fn test_nesting_shallow_passes() -> Result<()> {
    // Depth 1 (one if)
    let content = "fn shallow() { if true { println!(\"ok\"); } }";
    let violations = scan_with_rules(content, strict_rules())?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn test_nesting_deep_fails() -> Result<()> {
    // Depth 2 (nested if) > Max 1
    let content = "fn deep() { if true { if true { println!(\"too deep\"); } } }";
    let violations = scan_with_rules(content, strict_rules())?;

    assert!(violations.iter().any(|v| v.message.contains("Deep Nesting")));
    Ok(())
}

#[test]
fn test_arity_few_args_passes() -> Result<()> {
    let content = "fn add(a: i32, b: i32) {}";
    let violations = scan_with_rules(content, strict_rules())?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn test_arity_many_args_fails() -> Result<()> {
    // Args > 2
    let content = "fn many(a: i32, b: i32, c: i32) {}";
    let violations = scan_with_rules(content, strict_rules())?;

    assert!(violations.iter().any(|v| v.message.contains("High Arity")));
    Ok(())
}

// --- Law of Paranoia ---

#[test]
fn test_paranoia_unwrap_fails() -> Result<()> {
    let content = "fn risky() { let x = Some(1); x.unwrap(); }";
    let violations = scan_with_rules(content, strict_rules())?;

    assert!(violations
        .iter()
        .any(|v| v.message.contains("Banned: '.unwrap()'")));
    Ok(())
}

#[test]
fn test_paranoia_expect_fails() -> Result<()> {
    let content = "fn risky() { let x = Some(1); x.expect(\"boom\"); }";
    let violations = scan_with_rules(content, strict_rules())?;

    assert!(violations
        .iter()
        .any(|v| v.message.contains("Banned: '.expect()'")));
    Ok(())
}

#[test]
fn test_paranoia_no_unwrap_passes() -> Result<()> {
    let content = "fn safe() { let x = Some(1); x.unwrap_or(0); }";
    let violations = scan_with_rules(content, strict_rules())?;
    assert!(violations.is_empty());
    Ok(())
}

// --- Ignore Rules ---

#[test]
fn test_warden_ignore_skips_file() -> Result<()> {
    let content = r"
        // warden:ignore
        fn extremely_complex_and_bad(a:i32,b:i32,c:i32,d:i32,e:i32) {
             if true { if true { if true { x.unwrap(); } } }
        }
    ";
    let violations = scan_with_rules(content, strict_rules())?;

    assert!(violations.is_empty());
    Ok(())
}