// tests/unit_analysis.rs
// slopchop:ignore
use slopchop_core::analysis::ast::Analyzer;
use slopchop_core::analysis::RuleEngine;
use slopchop_core::config::{Config, RuleConfig};
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

// --- Helper for AST Analysis ---
fn analyze(lang: &str, code: &str, complexity: usize) -> bool {
    let analyzer = Analyzer::new();
    let config = RuleConfig {
        max_cyclomatic_complexity: complexity,
        max_function_words: 5, // Default for naming tests
        ..Default::default()
    };

    let violations = analyzer.analyze(lang, "test", code, &config);
    !violations.is_empty()
}

// --- Helper for File-Level Ignores (RuleEngine) ---
fn check_ignore(content: &str) -> bool {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.txt"); // Extension doesn't matter for ignores
    let mut file = File::create(&file_path).unwrap();
    write!(file, "{content}").unwrap();

    let config = Config::default();
    let engine = RuleEngine::new(config);

    // If ignored, analyze_file returns None.
    // If scan() returns empty files list, it was ignored.
    let report = engine.scan(vec![file_path]);
    report.files.is_empty()
}

// --- Helper for Naming Analysis ---
fn check_naming_violation(lang: &str, bad_code: &str, good_code: &str) {
    let analyzer = Analyzer::new();
    let config = RuleConfig {
        max_function_words: 3,
        ..Default::default()
    };

    let v_bad = analyzer.analyze(lang, "t", bad_code, &config);
    assert!(!v_bad.is_empty(), "Should detect violation in: {bad_code}");

    let v_good = analyzer.analyze(lang, "t", good_code, &config);
    assert!(v_good.is_empty(), "Should allow valid code: {good_code}");
}

#[test]
fn test_js_complexity() {
    // 1 (Func) + 1 (If) + 1 (For) = 3
    let code = "function f() { if(true) { for(;;) {} } }";
    assert!(analyze("js", code, 2), "Should fail limit 2");
    assert!(!analyze("js", code, 3), "Should pass limit 3");
}

#[test]
fn test_python_complexity() {
    // 1 (Def) + 1 (If) + 1 (While) = 3
    let code = "def f():\n  if True:\n    while True:\n      pass";
    assert!(analyze("py", code, 2), "Should fail limit 2");
    assert!(!analyze("py", code, 3), "Should pass limit 3");
}

#[test]
fn test_snake_case_words() {
    // "this_is_too_long" = 4 words vs "short_one" = 2 words
    check_naming_violation(
        "rs",
        "fn this_is_too_long() {}",
        "fn short_one() {}",
    );
}

#[test]
fn test_camel_case_words() {
    // "ThisIsTooLong" = 4 words vs "ShortOne" = 2 words
    check_naming_violation(
        "js",
        "function ThisIsTooLong() {}",
        "function ShortOne() {}",
    );
}

#[test]
fn test_slopchop_ignore_hash() {
    let content = "# slopchop:ignore\nfn extremely_bad_function() { if true { if true { } } }";
    assert!(
        check_ignore(content),
        "Should ignore file with hash comment"
    );
}

#[test]
fn test_slopchop_ignore_html() {
    let content = "<!-- slopchop:ignore -->\nfn bad() {}";
    assert!(
        check_ignore(content),
        "Should ignore file with html comment"
    );
}