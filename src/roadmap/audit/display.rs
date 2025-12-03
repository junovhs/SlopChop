// src/roadmap/audit/display.rs
//! Audit output formatting.

use super::types::{AuditViolation, ViolationReason};
use colored::Colorize;

pub fn print_violation(v: &AuditViolation) {
    let msg = format_reason(&v.reason);
    println!(
        "{} {} (id: {})",
        "⚠️  Traceability Fail:".red(),
        v.task_text.bold(),
        v.task_id.dimmed()
    );
    println!("   └─ {msg}");
}

fn format_reason(reason: &ViolationReason) -> String {
    match reason {
        ViolationReason::MissingTestFile(f) => format!("Missing File: {f}"),
        ViolationReason::MissingTestFunction { file, function } => {
            format!("Missing Function: '{function}' in {file}")
        }
        ViolationReason::NamingConventionMismatch { expected, actual } => {
            format!("Naming Mismatch: expected '{expected}', found '{actual}'")
        }
        ViolationReason::NoTraceability => "No test file found (heuristic)".to_string(),
    }
}

pub fn print_summary(missing: usize) {
    println!();
    if missing == 0 {
        println!(
            "{}",
            "✅ All completed tasks have verified tests!".green().bold()
        );
    } else {
        println!(
            "{}",
            format!("❌ Found {missing} tasks without verified tests.")
                .red()
                .bold()
        );
        println!("   (Tip: Add <!-- test: tests/my_test.rs::fn_name --> to ROADMAP.md)");
    }
}

