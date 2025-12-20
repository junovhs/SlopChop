// tests/integration_core.rs
//! Integration tests for the 3 Laws enforcement.

use anyhow::Result;
use slopchop_core::analysis::RuleEngine;
use slopchop_core::config::{Config, RuleConfig};
use slopchop_core::types::Violation;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[derive(Clone, Copy)]
enum RuleKind {
    Complexity,
    Depth,
    Arity,
    Tokens,
}

fn make_config(kind: RuleKind, limit: usize) -> RuleConfig {
    let mut cfg = RuleConfig::default();
    match kind {
        RuleKind::Complexity => cfg.max_cyclomatic_complexity = limit,
        RuleKind::Depth => cfg.max_nesting_depth = limit,
        RuleKind::Arity => cfg.max_function_args = limit,
        RuleKind::Tokens => cfg.max_file_tokens = limit,
    }
    cfg
}

fn scan(content: &str, rules: RuleConfig) -> Result<Vec<Violation>> {
    let dir = TempDir::new()?;
    let file_path = dir.path().join("source.rs");
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

#[test]
fn test_atomicity_boundary() -> Result<()> {
    // Limit 100, content small
    let content = r#"fn main() { println!("Small"); }"#;
    let v = scan(content, make_config(RuleKind::Tokens, 100))?;
    assert!(v.is_empty(), "Should pass token limit");

    // Limit 10, content large
    let content_large = "fn main() { let x = 1; } ".repeat(20);
    let v2 = scan(&content_large, make_config(RuleKind::Tokens, 10))?;
    assert!(!v2.is_empty(), "Should fail token limit");
    Ok(())
}

#[test]
fn test_complexity_boundary() -> Result<()> {
    // Complexity 2 (Base + If)
    let content = "fn f() { if true {} }";
    
    // Limit 2 (Pass)
    assert!(scan(content, make_config(RuleKind::Complexity, 2))?.is_empty());

    // Limit 1 (Fail)
    let v = scan(content, make_config(RuleKind::Complexity, 1))?;
    assert!(!v.is_empty(), "Complexity 2 should fail limit 1. Violations: {v:?}");
    Ok(())
}

#[test]
fn test_nesting_boundary() -> Result<()> {
    // Depth 2 (Fn -> If -> If)
    let content = "fn f() { if true { if true {} } }";

    // Limit 2 (Pass)
    assert!(scan(content, make_config(RuleKind::Depth, 2))?.is_empty());

    // Limit 1 (Fail)
    let v = scan(content, make_config(RuleKind::Depth, 1))?;
    assert!(!v.is_empty(), "Depth 2 should fail limit 1. Violations: {v:?}");
    Ok(())
}

#[test]
fn test_arity_boundary() -> Result<()> {
    // 2 Args
    let content = "fn f(a: i32, b: i32) {}";

    // Limit 2 (Pass)
    assert!(scan(content, make_config(RuleKind::Arity, 2))?.is_empty());

    // Limit 1 (Fail)
    let v = scan(content, make_config(RuleKind::Arity, 1))?;
    assert!(!v.is_empty(), "2 Args should fail limit 1. Violations: {v:?}");
    Ok(())
}

#[test]
fn test_paranoia_check() -> Result<()> {
    let content = "fn f() { let x = Some(1); x.unwrap(); }";
    let v = scan(content, RuleConfig::default())?;
    assert!(!v.is_empty(), "Unwrap should trigger violation");
    Ok(())
}