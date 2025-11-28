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
