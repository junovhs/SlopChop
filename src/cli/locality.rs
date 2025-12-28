// src/cli/locality.rs
//! Handler for locality scanning.

use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

use crate::config::Config;
use crate::discovery;
use crate::exit::SlopChopExit;
use crate::graph::imports;
use crate::graph::locality::{validate_graph, ValidationReport};
use crate::graph::resolver;

/// Runs locality validation on the codebase.
///
/// # Errors
/// Returns error if file discovery or import extraction fails.
pub fn handle_locality() -> Result<SlopChopExit> {
    let config = Config::load();
    let locality_config = config.rules.locality.to_validator_config();

    if !config.rules.locality.is_enabled() {
        println!("{}", "Locality checking is disabled.".yellow());
        return Ok(SlopChopExit::Success);
    }

    let project_root = std::env::current_dir()?;
    let files = discovery::discover(&config)?;
    let edges = collect_edges(&project_root, &files)?;

    let report = validate_graph(
        edges.iter().map(|(a, b)| (a.as_path(), b.as_path())),
        &locality_config,
    );

    print_report(&report);

    if report.is_clean() || !config.rules.locality.is_error_mode() {
        Ok(SlopChopExit::Success)
    } else {
        Ok(SlopChopExit::CheckFailed)
    }
}

fn collect_edges(
    root: &std::path::Path,
    files: &[PathBuf],
) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut edges = Vec::new();

    for file in files {
        let content = std::fs::read_to_string(file)?;
        let raw_imports = imports::extract(file, &content);

        for import_str in raw_imports {
            if let Some(resolved) = resolver::resolve(root, file, &import_str) {
                edges.push((file.clone(), resolved));
            }
        }
    }

    Ok(edges)
}

fn print_report(report: &ValidationReport) {
    println!(
        "\n{} Locality Analysis: {} edges, {} passed, {} failed",
        "→".cyan().bold(),
        report.total_edges,
        report.passed.len().to_string().green(),
        format_failed_count(report.failed.len()),
    );

    if report.failed.is_empty() {
        println!("{}", "  ✓ All dependencies respect locality.".green());
        return;
    }

    println!("\n{}", "  Violations:".red().bold());
    for edge in &report.failed {
        println!(
            "    {} → {} (D={}, K={:.2}, {})",
            edge.from.display(),
            edge.to.display().to_string().red(),
            edge.distance,
            edge.target_skew,
            edge.target_identity.label(),
        );
    }

    print_entropy(report);
}

fn format_failed_count(count: usize) -> String {
    if count == 0 {
        count.to_string().green().to_string()
    } else {
        count.to_string().red().to_string()
    }
}

fn print_entropy(report: &ValidationReport) {
    let entropy_pct = report.entropy * 100.0;
    let entropy_str = format!("{entropy_pct:.1}%");

    let colored = if entropy_pct > 30.0 {
        entropy_str.red()
    } else if entropy_pct > 10.0 {
        entropy_str.yellow()
    } else {
        entropy_str.green()
    };

    println!("\n  Topological Entropy: {colored}");
}