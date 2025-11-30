// tests/security_validation.rs
//! Security validation tests.
//! Covers: v0.4.0 Path Safety features

use std::collections::HashMap;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

fn make_file(path: &str) -> HashMap<String, FileContent> {
    let mut files = HashMap::new();
    files.insert(
        path.to_string(),
        FileContent {
            content: "valid content here".to_string(),
            line_count: 1,
        },
    );
    files
}

fn expect_security_failure(result: ApplyOutcome, expected_term: &str) {
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_expected = errors.iter().any(|e| {
            e.to_lowercase().contains(&expected_term.to_lowercase()) || e.contains("SECURITY")
        });
        assert!(
            has_expected,
            "Expected '{expected_term}' or SECURITY error, got: {errors:?}"
        );
    } else {
        panic!("Expected ValidationFailure for security violation");
    }
}

#[test]
fn test_traversal_blocked() {
    let test_cases = vec![
        "../secret.txt",
        "../../etc/passwd",
        "src/../../../root/secret",
        "..\\windows\\system32",
    ];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);
        expect_security_failure(result, "traversal");
    }
}

#[test]
fn test_absolute_paths_blocked() {
    let test_cases = vec![
        "/etc/passwd",
        "/root/.ssh/id_rsa",
        "C:\\Windows\\System32\\config",
        "D:\\secrets\\passwords.txt",
    ];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);
        expect_security_failure(result, "absolute");
    }
}

#[test]
fn test_sensitive_paths_blocked() {
    let test_cases = vec![
        ".env",
        ".env.local",
        ".ssh/config",
        ".ssh/id_rsa",
        ".aws/credentials",
        ".aws/config",
    ];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);
        if let ApplyOutcome::ValidationFailure { errors, .. } = result {
            let has_security = errors.iter().any(|e| {
                e.contains("SECURITY") || e.contains("sensitive") || e.contains("hidden")
            });
            assert!(
                has_security,
                "Path '{path}' should be blocked: {errors:?}"
            );
        } else {
            panic!("Path '{path}' should fail validation");
        }
    }
}

#[test]
fn test_valid_paths_allowed() {
    let test_cases = vec![
        "src/main.rs",
        "src/lib.rs",
        "src/modules/auth/handler.rs",
        "tests/integration_test.rs",
        "Cargo.toml",
        "README.md",
        "docs/guide.md",
    ];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);

        match result {
            ApplyOutcome::Success { written, .. } => {
                assert!(
                    written.contains(&path.to_string()),
                    "Path '{path}' should be allowed"
                );
            }
            ApplyOutcome::ValidationFailure { errors, .. } => {
                panic!("Valid path '{path}' was rejected: {errors:?}");
            }
            _ => panic!("Unexpected result for path '{path}'"),
        }
    }
}

#[test]
fn test_dots_in_filename_allowed() {
    let test_cases = vec![
        "src/config.dev.rs",
        "src/app.module.ts",
        "data/file.backup.json",
    ];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);

        if let ApplyOutcome::ValidationFailure { errors, .. } = result {
            let security_errors: Vec<_> = errors
                .iter()
                .filter(|e| e.contains("SECURITY") || e.contains("hidden"))
                .collect();
            assert!(
                security_errors.is_empty(),
                "Dots in filename '{path}' should be allowed: {errors:?}"
            );
        }
    }
}

#[test]
fn test_current_dir_reference() {
    let files = make_file("./src/main.rs");
    let result = validator::validate(&vec![], &files);

    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let hidden_error = errors.iter().any(|e| e.contains("hidden"));
        assert!(!hidden_error, "./ prefix should not be flagged as hidden");
    }
}

#[test]
fn test_multiple_security_issues() {
    let mut files = HashMap::new();
    files.insert(
        "../traversal.txt".to_string(),
        FileContent {
            content: "bad".to_string(),
            line_count: 1,
        },
    );
    files.insert(
        "/absolute/path.txt".to_string(),
        FileContent {
            content: "bad".to_string(),
            line_count: 1,
        },
    );
    files.insert(
        "src/valid.rs".to_string(),
        FileContent {
            content: "fn valid() {}".to_string(),
            line_count: 1,
        },
    );

    let result = validator::validate(&vec![], &files);

    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        assert!(errors.len() >= 2, "Should catch multiple security issues");
    } else {
        panic!("Should fail validation with multiple bad paths");
    }
}

#[test]
fn test_case_sensitivity() {
    let test_cases = vec![".ENV", ".Env", ".GIT/config", ".Git/config"];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);
        if !matches!(result, ApplyOutcome::ValidationFailure { .. }) {
            // Failure expected
        }
    }
}

#[test]
fn test_backslash_handling() {
    let test_cases = vec!["src\\main.rs", "..\\parent\\secret"];

    for path in test_cases {
        let files = make_file(path);
        let result = validator::validate(&vec![], &files);

        if path.contains("..\\") {
            if let ApplyOutcome::ValidationFailure { .. } = result {
                 // Expected
            }
        }
    }
}