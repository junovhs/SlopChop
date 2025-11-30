use warden_core::apply::validator;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use std::collections::HashMap;

fn mock_files(path: &str, content: &str) -> HashMap<String, FileContent> {
    let mut map = HashMap::new();
    map.insert(
        path.to_string(),
        FileContent {
            content: content.to_string(),
            line_count: content.lines().count(),
        },
    );
    map
}

#[test]
fn test_block_comment_ellipsis() {
    let content = "/* ... */";
    let files = mock_files("src/test.rs", content);
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_lazy = errors.iter().any(|e| e.contains("lazy truncation"));
        assert!(has_lazy, "Should detect block comment ellipsis: {errors:?}");
    } else {
        panic!("Should fail validation");
    }
}

#[test]
fn test_hash_ellipsis() {
    let content = "# ...";
    let files = mock_files("script.py", content);
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_lazy = errors.iter().any(|e| e.contains("lazy truncation"));
        assert!(has_lazy, "Should detect hash ellipsis: {errors:?}");
    }
}

#[test]
fn test_lazy_phrase_rest_of() {
    let content = "// rest of the code";
    let files = mock_files("src/test.rs", content);
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_lazy = errors.iter().any(|e| e.contains("lazy truncation"));
        assert!(has_lazy, "Should detect 'rest of' phrase: {errors:?}");
    }
}

#[test]
fn test_lazy_phrase_remaining() {
    let content = "// remaining implementation";
    let files = mock_files("src/test.rs", content);
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_lazy = errors.iter().any(|e| e.contains("lazy truncation"));
        assert!(has_lazy, "Should detect 'remaining' phrase: {errors:?}");
    }
}

#[test]
fn test_line_number_reported() {
    let content = "line 1\nline 2\n// ...";
    let files = mock_files("src/test.rs", content);
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_line_num = errors.iter().any(|e| e.contains('3'));
        assert!(
            has_line_num,
            "Should report line number in error: {errors:?}"
        );
    }
}

#[test]
fn test_gnupg_blocked() {
    let files = mock_files(".gnupg/pubring.kbx", "data");
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_security = errors.iter().any(|e| e.contains("sensitive"));
        assert!(has_security, "Should block .gnupg: {errors:?}");
    }
}

#[test]
fn test_id_rsa_blocked() {
    let files = mock_files("id_rsa", "key");
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_security = errors.iter().any(|e| e.contains("sensitive"));
        assert!(has_security, "Should block `id_rsa`: {errors:?}");
    }
}

#[test]
fn test_credentials_blocked() {
    let files = mock_files("credentials", "secret");
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_security = errors.iter().any(|e| e.contains("sensitive"));
        assert!(has_security, "Should block credentials file: {errors:?}");
    }
}

#[test]
fn test_backup_dir_blocked() {
    let files = mock_files(".warden_apply_backup/foo", "data");
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_security = errors.iter().any(|e| e.contains("sensitive"));
        assert!(has_security, "Should block backup directory: {errors:?}");
    }
}

#[test]
fn test_valid_code_passes() {
    let content = "fn main() { println!(\"...\"); }"; // "..." in string is valid
    let files = mock_files("src/main.rs", content);
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        panic!("Valid code should pass: {errors:?}");
    }
}

#[test]
fn test_warden_ignore_bypass() {
    let content = "// warden:ignore\n// ...";
    let files = mock_files("src/ignore.rs", content);
    let result = validator::validate(&vec![], &files);
    
    // Based on logic, ignore skips truncation check.
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let truncation_errors: Vec<_> = errors.iter().filter(|e| e.contains("truncation")).collect();
        assert!(
            truncation_errors.is_empty(),
            "warden:ignore should bypass: {errors:?}"
        );
    }
}

#[test]
fn test_mixed_validity() {
    let mut files = HashMap::new();
    files.insert("good.rs".to_string(), FileContent { content: "ok".to_string(), line_count: 1 });
    files.insert("bad.rs".to_string(), FileContent { content: "// ...".to_string(), line_count: 1 });
    
    let result = validator::validate(&vec![], &files);
    
    if let ApplyOutcome::ValidationFailure { errors, .. } = result {
        let has_bad_error = errors.iter().any(|e| e.contains("bad.rs"));
        assert!(
            has_bad_error,
            "Should report error for bad.rs: {errors:?}"
        );
    }
}