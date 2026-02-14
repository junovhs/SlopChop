//! External command verification pipeline.
//!
//! Runs commands defined in `[commands]` section of slopchop.toml
//! and captures output to `slopchop-report.txt`.

mod runner;

use std::path::Path;

use crate::config::Config;

/// Result of running the verification pipeline.
#[derive(Debug)]
pub struct VerificationReport {
    /// Whether all commands passed.
    pub passed: bool,
    /// Combined output from all commands.
    pub output: String,
}

impl VerificationReport {
    #[must_use]
    pub fn new(passed: bool, output: String) -> Self {
        Self { passed, output }
    }
}

/// Runs the verification pipeline using commands from config.
pub fn run(repo_root: &Path) -> VerificationReport {
    let config = Config::load();
    let commands = config.commands.get("check").cloned().unwrap_or_default();

    runner::run_commands(repo_root, &commands)
}
