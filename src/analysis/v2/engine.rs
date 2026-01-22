// src/analysis/v2/engine.rs
//! Main execution logic for Scan V2.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::types::Violation;

use super::aggregator::Aggregator;
use super::deep::DeepAnalyzer;
use super::worker;

/// Source files below this threshold skip structural metrics (LCOM4, CBO, AHF, SFOUT).
/// Rationale: For small projects, modularity metrics are noise, not signal.
/// See: case-study-gittrek.md
pub const SMALL_CODEBASE_THRESHOLD: usize = 10;

/// Returns true if the source file count is below the small codebase threshold.
/// Only counts files in src/ directory, excluding tests.
#[must_use]
pub fn is_small_codebase(file_count: usize) -> bool {
    file_count < SMALL_CODEBASE_THRESHOLD
}

/// Returns the small codebase threshold for external display.
#[must_use]
pub const fn small_codebase_threshold() -> usize {
    SMALL_CODEBASE_THRESHOLD
}

/// Counts only source files (in src/ directory), excluding tests.
#[must_use]
pub fn count_source_files(files: &[PathBuf]) -> usize {
    files
        .iter()
        .filter(|p| is_source_file(p))
        .count()
}

/// Returns true if path is a source file (not test/bench/example).
fn is_source_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Must be in src/ directory
    if !path_str.contains("src/") && !path_str.starts_with("src/") {
        return false;
    }
    
    // Exclude test files
    if path_str.contains("/tests/") || path_str.contains("_test.") || path_str.contains("tests.rs") {
        return false;
    }
    
    true
}

pub struct ScanEngineV2 {
    config: Config,
}

impl ScanEngineV2 {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the Scan v2 engine and returns violations mapped by file path.
    #[must_use]
    pub fn run(&self, files: &[PathBuf]) -> HashMap<PathBuf, Vec<Violation>> {
        run_analysis(files, &self.config)
    }
}

/// Core analysis logic extracted to reduce struct fan-out.
fn run_analysis(files: &[PathBuf], config: &Config) -> HashMap<PathBuf, Vec<Violation>> {
    // Small codebase detection: count only src/ files, skip structural metrics.
    let source_count = count_source_files(files);
    if source_count < SMALL_CODEBASE_THRESHOLD {
        return HashMap::new();
    }

    let aggregator = collect_local_analysis(files);
    compute_deep_violations(aggregator, config)
}

/// Phase 1: Local analysis (parallelizable).
fn collect_local_analysis(files: &[PathBuf]) -> Aggregator {
    let mut aggregator = Aggregator::new();
    for path in files {
        if let Some(analysis) = worker::scan_file(path) {
            aggregator.ingest(path, analysis);
        }
    }
    aggregator
}

/// Phase 2: Global/Deep analysis (metrics).
fn compute_deep_violations(
    mut aggregator: Aggregator,
    config: &Config,
) -> HashMap<PathBuf, Vec<Violation>> {
    let deep_analyzer = DeepAnalyzer::new(&config.rules);
    let deep_violations = deep_analyzer.compute_violations(&aggregator);
    aggregator.merge(deep_violations);
    aggregator.violations
}