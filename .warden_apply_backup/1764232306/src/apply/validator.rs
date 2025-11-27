// src/apply/validator.rs
use crate::apply::messages;
use crate::apply::types::{ApplyOutcome, ExtractedFiles, Manifest, Operation};
const BACKTICK: char = '`';
const TILDE: char = '~';
const SENSITIVE_PATHS: &[&str] = &[
".git/",
".env",
".ssh/",
".aws/",
".gnupg/",
"id_rsa",
"id_ed25519",
"credentials",
".warden_apply_backup/",
];
#[must_use]
pub fn validate(manifest: &Manifest, extracted: &ExtractedFiles) -> ApplyOutcome {
let mut errors = Vec::new();
check_path_safety(extracted, &mut errors);

if !errors.is_empty() {
    let ai_message = messages::format_ai_rejection(&[], &errors);
    return ApplyOutcome::ValidationFailure {
        errors,
        missing: Vec::new(),
        ai_message,
    };
}

let missing = check_missing(manifest, extracted);
let content_errors = check_content(extracted);

if !missing.is_empty() || !content_errors.is_empty() {
    let ai_message = messages::format_ai_rejection(&missing, &content_errors);
    return ApplyOutcome::ValidationFailure {
        errors: content_errors,
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
fn check_path_safety(extracted: &ExtractedFiles, errors: &mut Vec<String>) {
for path in extracted.keys() {
validate_single_path(path, errors);
}
}
fn validate_single_path(path: &str, errors: &mut Vec<String>) {
if has_traversal(path) {
errors.push(format!("SECURITY: path contains directory traversal: {path}"));
return;
}
if is_absolute_path(path) {
    errors.push(format!("SECURITY: absolute path not allowed: {path}"));
    return;
}

if is_sensitive_path(path) {
    errors.push(format!("SECURITY: sensitive path blocked: {path}"));
    return;
}

if is_hidden_file(path) {
    errors.push(format!("SECURITY: hidden file not allowed: {path}"));
}
}
fn has_traversal(path: &str) -> bool {
path.contains("../") || path.starts_with("..")
}
fn is_absolute_path(path: &str) -> bool {
if path.starts_with('/') {
return true;
}
if path.len() >= 2 {
let bytes = path.as_bytes();
if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
return true;
}
}
false
}
fn is_sensitive_path(path: &str) -> bool {
let lower = path.to_lowercase();
SENSITIVE_PATHS.iter().any(|s| lower.contains(s))
}
fn is_hidden_file(path: &str) -> bool {
path.split('/')
.filter(|s| !s.is_empty())
.any(|seg| seg.starts_with('.') && seg != "." && seg != "..")
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
check_single_file(path, &file.content, &mut errors);
}
errors
}
fn check_single_file(path: &str, content: &str, errors: &mut Vec<String>) {
if content.trim().is_empty() {
errors.push(format!("{path} is empty"));
return;
}
if has_markdown_fence(content) {
    errors.push(format!(
        "{path} contains markdown code block - AI output must use <file> tags, not markdown"
    ));
}
}
fn has_markdown_fence(content: &str) -> bool {
let triple_backtick: String = std::iter::repeat_n(BACKTICK, 3).collect();
let triple_tilde: String = std::iter::repeat_n(TILDE, 3).collect();
content.contains(&triple_backtick) || content.contains(&triple_tilde)
}