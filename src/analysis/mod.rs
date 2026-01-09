// src/analysis/mod.rs
//! Core analysis logic (The "Rule Engine").

pub mod ast;
pub mod checks;
pub mod metrics;
pub mod safety;
pub mod v2;

use crate::config::Config;
use crate::lang::Lang;
use crate::types::{FileReport, Violation, ScanReport};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

/// Orchestrates the analysis of multiple files.
pub struct RuleEngine {
    config: Config,
}

impl RuleEngine {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Entry point for scanning files.
    #[must_use]
    pub fn scan(&self, files: Vec<PathBuf>) -> ScanReport {
        let start = std::time::Instant::now();

        // 1. AST Analysis
        let mut results: Vec<FileReport> = files
            .par_iter()
            .map(|path| analyze_file(path, &self.config))
            .collect();

        // 2. Structural Analysis
        let v2_engine = v2::ScanEngineV2::new(self.config.clone());
        let deep_violations = v2_engine.run(&files);

        // 3. Merge
        merge_violations(&mut results, &deep_violations);

        let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
        let total_tokens: usize = results.iter().map(|r| r.token_count).sum();

        ScanReport {
            files: results,
            total_violations,
            total_tokens,
            duration_ms: start.elapsed().as_millis(),
        }
    }
}

fn analyze_file(path: &Path, config: &Config) -> FileReport {
    let mut report = FileReport {
        path: path.to_path_buf(),
        token_count: 0,
        complexity_score: 0,
        violations: Vec::new(),
    };

    let Ok(source) = std::fs::read_to_string(path) else {
        return report;
    };

    if has_ignore_directive(&source) {
        return report;
    }

    let tokens = crate::tokens::Tokenizer::count(&source);
    report.token_count = tokens;

    if tokens > config.rules.max_file_tokens && !is_ignored(path, &config.rules.ignore_tokens_on) {
        report.violations.push(Violation::simple(
            1,
            format!("File size is {tokens} tokens (Limit: {})", config.rules.max_file_tokens),
            "LAW OF ATOMICITY",
        ));
    }

    if let Some(lang) = Lang::from_ext(path.extension().and_then(|s| s.to_str()).unwrap_or("")) {
        let result = ast::Analyzer::new().analyze(lang, path.to_str().unwrap_or(""), &source, &config.rules);
        report.violations.extend(result.violations);
        report.complexity_score = result.max_complexity;
    }

    report
}

fn merge_violations(results: &mut [FileReport], deep: &std::collections::HashMap<PathBuf, Vec<Violation>>) {
    for r in results {
        if let Some(v) = deep.get(&r.path) {
            r.violations.extend(v.clone());
        }
    }
}

fn is_ignored(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| path_str.contains(p))
}

fn has_ignore_directive(source: &str) -> bool {
    source.lines().take(5).any(|line| line.contains("slopchop:ignore"))
}