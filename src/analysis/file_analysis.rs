// src/analysis/file_analysis.rs
use crate::config::Config;
use crate::lang::Lang;
use crate::types::{FileReport, Violation};
use crate::analysis::ast;
use std::path::Path;

#[must_use]
pub fn analyze_file(path: &Path, config: &Config) -> FileReport {
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

    report.token_count = crate::tokens::Tokenizer::count(&source);

    if report.token_count > config.rules.max_file_tokens && !is_ignored(path, &config.rules.ignore_tokens_on) {
        report.violations.push(Violation::simple(
            1,
            format!("File size is {} tokens (Limit: {})", report.token_count, config.rules.max_file_tokens),
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

#[must_use]
pub fn is_ignored(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|p| path_str.contains(p))
}

#[must_use]
pub fn has_ignore_directive(source: &str) -> bool {
    source.lines().take(5).any(|line| line.contains("slopchop:ignore"))
}
