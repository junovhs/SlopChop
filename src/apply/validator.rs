// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
use std::path::Path;

// Use hex escapes for backticks so this file doesn't trigger its own validation logic
// when being applied by Warden.
const MARKDOWN_PATTERNS: &[&str] = &[
    "\x60\x60\x60", // Standard markdown code blocks
    "~~~",          // Alternative markdown code blocks
];

// Lazy markers common in AI output when it gives up
const TRUNCATION_MARKERS: &[&str] = &[
    "// ...",
    "/* ... */",
    "// ... existing code ...",
    "// ... rest of file ...",
    "# ...",
    "<!-- ... -->",
];

const DANGEROUS_PATHS: &[&str] = &[
    ".git",
    ".env",
    "id_rsa",
    "id_ed25519",
    "credentials",
    "secrets",
    ".ssh",
    ".aws",
    ".kube",
];

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
        validate_single_file(path, &file.content, &mut errors);
    }
    errors
}

fn validate_single_file(path: &str, content: &str, errors: &mut Vec<String>) {
    // 1. Path Safety
    if let Err(e) = check_path_safety(path) {
        errors.push(format!("{path}: Security Violation - {e}"));
        return;
    }

    // 2. Empty Content
    if content.trim().is_empty() {
        errors.push(format!("{path}: File is empty"));
        return;
    }

    // 3. Markdown Formatting
    if let Some(pattern) = detect_markdown_block(content) {
        errors.push(format!(
            "{path}: Contains markdown code block '{pattern}' - AI output must use <file> tags only"
        ));
    }

    // 4. Truncation / Laziness
    if let Some(marker) = detect_truncation(content) {
        errors.push(format!(
            "{path}: Detected truncation marker '{marker}'. Do not lazy-load code."
        ));
    }

    // 5. Structural Integrity (Braces)
    if !is_balanced(path, content) {
        errors.push(format!(
            "{path}: Unbalanced braces/brackets detected. File may be truncated."
        ));
    }
}

fn check_path_safety(path_str: &str) -> Result<(), String> {
    let path = Path::new(path_str);

    // Absolute paths
    if path.is_absolute() {
        return Err("Absolute paths are forbidden".into());
    }

    // Traversal
    for component in path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("Directory traversal (../) is forbidden".into());
        }
    }

    // Dangerous targets
    if path_str.starts_with(".git") || path_str.contains("/.git") {
        return Err("Modifying .git internals is forbidden".into());
    }

    for &danger in DANGEROUS_PATHS {
        if path_str.ends_with(danger) || path_str.contains(&format!("/{danger}")) {
            return Err(format!("Modifying sensitive file '{danger}' is forbidden"));
        }
    }

    Ok(())
}

fn detect_markdown_block(content: &str) -> Option<&'static str> {
    MARKDOWN_PATTERNS
        .iter()
        .find(|&&pattern| content.contains(pattern))
        .copied()
}

fn detect_truncation(content: &str) -> Option<&'static str> {
    TRUNCATION_MARKERS
        .iter()
        .find(|&&marker| content.contains(marker))
        .copied()
}

fn is_balanced(path: &str, content: &str) -> bool {
    // Python relies on indentation, not braces. Skip strict brace checking.
    if path.ends_with(".py") {
        return true;
    }

    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;

    // A very simple state machine to avoid counting braces inside strings
    for c in content.chars() {
        if escaped {
            escaped = false;
            continue;
        }

        if c == '\\' {
            escaped = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        match c {
            '{' | '(' | '[' => stack.push(c),
            '}' => {
                if stack.pop() != Some('{') {
                    return false;
                }
            }
            ')' => {
                if stack.pop() != Some('(') {
                    return false;
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return false;
                }
            }
            _ => {}
        }
    }

    stack.is_empty()
}
