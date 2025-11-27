//! Integration tests for path security validation
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
let sensitive: &[&str] = &[
".git/", ".env", ".ssh/", ".aws/", ".gnupg/",
"id_rsa", "id_ed25519", "credentials", ".warden_apply_backup/",
];
let lower = path.to_lowercase();
sensitive.iter().any(|s| lower.contains(s))
}
fn is_hidden_file(path: &str) -> bool {
path.split('/')
.filter(|s| !s.is_empty())
.any(|seg| seg.starts_with('.') && seg != "." && seg != "..")
}
fn is_blocked(path: &str) -> bool {
has_traversal(path) || is_absolute_path(path) || is_sensitive_path(path) || is_hidden_file(path)
}
#[test]
fn test_traversal_blocked() {
assert!(is_blocked("../etc/passwd"));
assert!(is_blocked("foo/../../etc"));
assert!(is_blocked(".."));
}
#[test]
fn test_absolute_paths_blocked() {
assert!(is_blocked("/etc/passwd"));
assert!(is_blocked("/usr/bin/bash"));
assert!(is_blocked("C:/Windows"));
assert!(is_blocked("D:/Users"));
}
#[test]
fn test_sensitive_paths_blocked() {
assert!(is_blocked(".git/config"));
assert!(is_blocked("foo/.git/hooks"));
assert!(is_blocked(".env"));
assert!(is_blocked(".ssh/id_rsa"));
assert!(is_blocked(".aws/credentials"));
}
#[test]
fn test_hidden_files_blocked() {
assert!(is_blocked(".secrets"));
assert!(is_blocked("config/.private"));
assert!(is_blocked(".hidden/file.txt"));
}
#[test]
fn test_valid_paths_allowed() {
assert!(!is_blocked("src/main.rs"));
assert!(!is_blocked("src/apply/mod.rs"));
assert!(!is_blocked("tests/test.rs"));
assert!(!is_blocked("Cargo.toml"));
assert!(!is_blocked("README.md"));
}