// src/roadmap/audit/mod.rs
//! Roadmap traceability audit system.

mod checker;
mod display;
mod scanner;
mod types;

pub use types::{AuditOptions, AuditReport, AuditViolation, ViolationReason};

use crate::roadmap::types::{Roadmap, TaskStatus};
use colored::Colorize;
use std::path::Path;

/// Runs the audit and prints results.
#[must_use]
pub fn run(roadmap: &Roadmap, root: &Path, opts: AuditOptions) -> bool {
    println!("{}", "🕵️  Roadmap Traceability Audit".bold().cyan());
    println!("{}", "─────────────────────────────────────".dimmed());

    let report = scan(roadmap, root, &opts);

    if report.total_checked == 0 {
        println!("{}", "No completed tasks to audit.".yellow());
        return true;
    }

    for v in &report.violations {
        display::print_violation(v);
    }

    display::print_summary(report.violations.len());
    report.violations.is_empty()
}

/// Scans roadmap for traceability violations.
#[must_use]
pub fn scan(roadmap: &Roadmap, root: &Path, opts: &AuditOptions) -> AuditReport {
    let tasks = roadmap.all_tasks();
    let completed: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .collect();

    if completed.is_empty() {
        return AuditReport::new();
    }

    let scanned = scanner::scan_test_files(root);
    let mut report = AuditReport::new();

    for task in completed {
        if task.text.contains("[no-test]") {
            continue;
        }
        report.total_checked += 1;

        if let Some(reason) = checker::check_task(task, root, &scanned, opts.strict) {
            report.violations.push(AuditViolation {
                task_id: task.id.clone(),
                task_text: task.text.clone(),
                reason,
            });
        }
    }

    report
}
