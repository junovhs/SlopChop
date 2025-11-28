// src/reporting.rs
use crate::types::{FileReport, ScanReport, Violation};
use anyhow::Result;
use colored::Colorize;

/// Prints the scan report to stdout.
///
/// # Errors
/// Returns Ok(()) normally.
pub fn print_report(report: &ScanReport) -> Result<()> {
    let failures = count_failures(report);

    // Filter and print only violating files
    report
        .files
        .iter()
        .filter(|f| !f.is_clean())
        .for_each(print_file_report);

    print_summary(report, failures);
    Ok(())
}

fn count_failures(report: &ScanReport) -> usize {
    report
        .files
        .iter()
        .filter(|f| !f.is_clean())
        .map(|f| f.violations.len())
        .sum()
}

fn print_file_report(file: &FileReport) {
    for v in &file.violations {
        print_violation(&file.path, v);
    }
}

fn print_violation(path: &std::path::Path, v: &Violation) {
    let filename = path.to_string_lossy();
    let line_num = v.row + 1;

    println!("{}: {}", "error".red().bold(), v.message.bold());
    println!("  {} {}:{}:1", "-->".blue(), filename, line_num);
    println!("   {}", "|".blue());
    println!(
        "   {} {}: Action required",
        "=".blue().bold(),
        v.law.white().bold()
    );
    println!();
}

fn print_summary(report: &ScanReport, failures: usize) {
    if failures > 0 {
        let msg = format!(
            "❌ Warden found {failures} violations in {}ms.",
            report.duration_ms
        );
        println!("{}", msg.red().bold());
    } else {
        // src/types.rs
        use std::path::PathBuf;

        /// A single violation detected during analysis.
        #[derive(Debug, Clone)]
        pub struct Violation {
            pub row: usize,
            pub message: String,
            pub law: &'static str,
        }

        /// Analysis results for a single file.
        #[derive(Debug, Clone)]
        pub struct FileReport {
            pub path: PathBuf,
            pub token_count: usize,
            pub complexity_score: usize,
            pub violations: Vec<Violation>,
        }

        impl FileReport {
            /// Returns true if no violations were found.
            #[must_use]
            pub fn is_clean(&self) -> bool {
                self.violations.is_empty()
            }

            /// Returns the number of violations.
            #[must_use]
            pub fn violation_count(&self) -> usize {
                self.violations.len()
            }
        }

        /// Aggregated results from scanning multiple files.
        #[derive(Debug, Clone, Default)]
        pub struct ScanReport {
            pub files: Vec<FileReport>,
            pub total_tokens: usize,
            pub total_violations: usize,
            pub duration_ms: u128,
        }

        impl ScanReport {
            /// Returns true if any violations were found.
            #[must_use]
            pub fn has_errors(&self) -> bool {
                self.total_violations > 0
            }

            /// Returns the number of clean files.
            #[must_use]
            pub fn clean_file_count(&self) -> usize {
                self.files.iter().filter(|f| f.is_clean()).count()
            }
        }
    }
}
