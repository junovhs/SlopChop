// src/analysis/v2/patterns/security.rs
//! Security patterns: X01, X02, X03

use crate::types::{Violation, ViolationDetails};
use tree_sitter::{Node, Query, QueryCursor};

/// Detects security violations in Rust code.
#[must_use]
pub fn detect(source: &str, root: Node) -> Vec<Violation> {
    let mut violations = Vec::new();
    detect_x01_sql(source, root, &mut violations);
    detect_x02_command(source, root, &mut violations);
    detect_x03_secrets(source, root, &mut violations);
    violations
}

/// X01: Potential SQL Injection via `format!`
/// Looks for `format!("... SELECT ... {}", ...)` patterns.
fn detect_x01_sql(source: &str, root: Node, out: &mut Vec<Violation>) {
    // We look for format! macros where the format string contains SQL keywords
    let query_str = r#"
        (macro_invocation
            macro: (identifier) @mac
            (token_tree) @args
            (#eq? @mac "format")) @fmt
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(arg_node) = m.captures.iter().find(|c| c.index == 1).map(|c| c.node) {
            let args = arg_node.utf8_text(source.as_bytes()).unwrap_or("");
            if is_suspicious_sql_format(args) {
                let row = arg_node.start_position().row + 1;
                out.push(Violation::with_details(
                    row,
                    "Potential SQL Injection detected".to_string(),
                    "X01",
                    ViolationDetails {
                        function_name: None,
                        analysis: vec![
                            "Formatting a string into a SQL query bypasses parameterization.".into(),
                            "This is the #1 cause of SQL injection vulnerabilities.".into()
                        ],
                        suggestion: Some("Use parameterized queries (e.g., `sqlx::query!(..., param)`) instead of string formatting.".into()),
                    }
                ));
            }
        }
    }
}

fn is_suspicious_sql_format(text: &str) -> bool {
    let upper = text.to_uppercase();
    // Must contain a SQL verb AND formatting braces
    let has_sql = upper.contains("SELECT ") 
        || upper.contains("INSERT INTO ") 
        || upper.contains("UPDATE ") 
        || upper.contains("DELETE FROM ");
    
    let has_interpolation = text.contains("{}") || text.contains("{:");
    
    has_sql && has_interpolation
}

/// X02: Command Injection
/// Looks for `Command::new(var)` where var is not a string literal.
fn detect_x02_command(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (call_expression
            function: (scoped_identifier
                path: (identifier) @struct
                name: (identifier) @method)
            arguments: (arguments
                (identifier) @arg)
            (#eq? @struct "Command")
            (#eq? @method "new")) @call
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(cap) = m.captures.last() {
            let row = cap.node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Potential Command Injection".to_string(),
                "X02",
                ViolationDetails {
                    function_name: None,
                    analysis: vec![
                        "Passing a variable directly to `Command::new` can be dangerous.".into(),
                        "Ensure the variable source is trusted.".into()
                    ],
                    suggestion: Some("Use hardcoded command paths where possible, or validate the input against an allowlist.".into()),
                }
            ));
        }
    }
}

/// X03: Hardcoded Secrets
/// Looks for assignments to variables named like secrets with string literal values.
fn detect_x03_secrets(source: &str, root: Node, out: &mut Vec<Violation>) {
    let query_str = r#"
        (let_declaration
            pattern: (identifier) @name
            value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @decl
        (const_item
            name: (identifier) @name
            value: (string_literal) @value
            (#match? @name "(?i)(key|secret|token|password|auth)")) @const
    "#;

    let Ok(query) = Query::new(tree_sitter_rust::language(), query_str) else { return; };
    let mut cursor = QueryCursor::new();

    for m in cursor.matches(&query, root, source.as_bytes()) {
        if let Some(val_node) = m.captures.iter().find(|c| c.index == 1).map(|c| c.node) {
            let val = val_node.utf8_text(source.as_bytes()).unwrap_or("");
            // Ignore placeholders
            if val.contains("placeholder") || val.contains("example") || val.len() < 5 {
                continue;
            }

            let row = val_node.start_position().row + 1;
            out.push(Violation::with_details(
                row,
                "Hardcoded secret detected".to_string(),
                "X03",
                ViolationDetails {
                    function_name: None,
                    analysis: vec![
                        "Storing secrets in source code is insecure.".into(),
                        "They will persist in git history forever.".into()
                    ],
                    suggestion: Some("Use environment variables (`std::env::var`) or a secrets manager.".into()),
                }
            ));
        }
    }
}