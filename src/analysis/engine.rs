// src/analysis/engine.rs
use crate::config::Config;
use crate::types::ScanReport;
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
    pub fn scan(&self, files: &[PathBuf]) -> ScanReport {
        crate::analysis::logic::run_scan(
            &self.config,
            files,
            None::<&fn(&Path)>,
            None::<&fn(&str)>
        )
    }

    /// Entry point for scanning files with progress callback.
    pub fn scan_with_progress<F, S>(
        &self,
        files: &[PathBuf],
        on_progress: &F,
        on_status: &S
    ) -> ScanReport
    where
        F: Fn(&Path) + Sync,
        S: Fn(&str) + Sync,
    {
        crate::analysis::logic::run_scan(
            &self.config,
            files,
            Some(on_progress),
            Some(on_status)
        )
    }
}