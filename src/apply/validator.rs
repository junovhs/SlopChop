// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};

#[must_use]
pub fn validate(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
    let missing = check_missing(manifest, extracted);
    let errors = check_content(extracted);

    if !missing.is_empty() || !errors.is_empty() {
        let ai_message = messages::format_ai_rejection(&missing, &errors);
        return ApplyOutcome::ValidationFailure {
            errors,
            missing,
            ai_message,
        };
    }

    let written = extracted.keys().cloned().collect();
    ApplyOutcome::Success {
        written,
        backed_up: true,
    }
}

fn check_missing(manifest: &Manifest, extracted: &ExtractedFiles) -> Vec<String> {
    let mut missing = Vec::new();
    for entry in manifest {
        if entry.operation != Operation::Delete && !extracted.contains_key(&entry.path) {
            missing.push(entry.path.clone());
        }
    }
    missing
}

fn check_content(extracted: &ExtractedFiles) -> Vec<String> {
    let mut errors = Vec::new();
    for (path, file) in extracted {
        if file.content.trim().is_empty() {
            errors.push(format!("{path} is empty"));
        }
    }
    errors
}
