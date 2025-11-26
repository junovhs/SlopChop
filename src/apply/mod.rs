// src/apply/mod.rs
pub mod extractor;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod writer;

use crate::clipboard;
use anyhow::{Context, Result};
use types::{ApplyOutcome, ExtractedFiles, Manifest};

/// Runs the apply command logic.
///
/// # Errors
/// Returns error if clipboard access fails or extraction fails.
pub fn run_apply(dry_run: bool) -> Result<ApplyOutcome> {
    let content = clipboard::read_clipboard().context("Failed to read clipboard")?;

    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Clipboard is empty".to_string()));
    }

    let validation = parse_and_validate(&content);

    match validation {
        ApplyOutcome::Success { .. } => {
            if dry_run {
                return Ok(ApplyOutcome::Success {
                    written: vec!["(Dry Run) Files verified".to_string()],
                    backed_up: false,
                });
            }
            writer::write_files(&extractor::extract_files(&content)?)
        }
        _ => Ok(validation),
    }
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

fn parse_and_validate(content: &str) -> ApplyOutcome {
    let manifest = match parse_manifest_step(content) {
        Ok(m) => m,
        Err(e) => return ApplyOutcome::ParseError(e),
    };

    let extracted = match extract_files_step(content) {
        Ok(e) => e,
        Err(e) => return ApplyOutcome::ParseError(e),
    };

    validator::validate(&manifest, &extracted)
}

fn parse_manifest_step(content: &str) -> Result<Manifest, String> {
    match manifest::parse_manifest(content) {
        Ok(Some(m)) => Ok(m),
        Ok(None) => Ok(Vec::new()),
        Err(e) => Err(format!("Manifest Error: {e}")),
    }
}

fn extract_files_step(content: &str) -> Result<ExtractedFiles, String> {
    extractor::extract_files(content).map_err(|e| format!("Extraction Error: {e}"))
}
