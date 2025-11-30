use warden_core::apply::manifest;
use warden_core::apply::types::Operation;

// Helper to construct delimiters without confusing the outer warden apply
fn nabla() -> String { "\u{2207}".repeat(3) }
fn delta() -> String { "\u{2206}".repeat(3) }

#[test]
fn test_parse_manifest() {
    let input = format!(
        "{} MANIFEST {}\nsrc/main.rs\nsrc/lib.rs\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, "src/main.rs");
    assert_eq!(entries[0].operation, Operation::Update);
}

#[test]
fn test_new_marker() {
    let input = format!(
        "{} MANIFEST {}\nsrc/existing.rs\nsrc/new_file.rs [NEW]\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries[1].path, "src/new_file.rs");
    assert_eq!(entries[1].operation, Operation::New);
}

#[test]
fn test_delete_marker() {
    let input = format!(
        "{} MANIFEST {}\nsrc/keep.rs\nsrc/remove.rs [DELETE]\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries[1].path, "src/remove.rs");
    assert_eq!(entries[1].operation, Operation::Delete);
}

#[test]
fn test_default_update() {
    let input = format!(
        "{} MANIFEST {}\nsrc/main.rs\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries[0].operation, Operation::Update);
}

#[test]
fn test_mixed_operations() {
    let input = format!(
        "{} MANIFEST {}\nsrc/update.rs\nsrc/create.rs [NEW]\nsrc/remove.rs [DELETE]\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries[0].operation, Operation::Update);
    assert_eq!(entries[1].operation, Operation::New);
    assert_eq!(entries[2].operation, Operation::Delete);
}

#[test]
fn test_markdown_lists() {
    let input = format!(
        "{} MANIFEST {}\n- src/file1.rs\n- src/file2.rs [NEW]\n* src/file3.rs\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "src/file1.rs");
    assert_eq!(entries[1].path, "src/file2.rs");
    assert_eq!(entries[2].path, "src/file3.rs");
}

#[test]
fn test_numbered_lists() {
    let input = format!(
        "{} MANIFEST {}\n1. src/first.rs\n2. src/second.rs\n3. src/third.rs [NEW]\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].path, "src/first.rs");
}

#[test]
fn test_empty_manifest() {
    let input = format!(
        "{} MANIFEST {}\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert!(entries.is_empty());
}

#[test]
fn test_no_manifest() {
    let input = format!(
        "{} src/main.rs {}\nfn main() {{}}\n{}",
        nabla(), nabla(), delta()
    );
    let result = manifest::parse_manifest(&input).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_case_insensitive_markers() {
    let input = format!(
        "{} MANIFEST {}\nsrc/file1.rs [new]\nsrc/file2.rs [delete]\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries[0].operation, Operation::New);
    assert_eq!(entries[1].operation, Operation::Delete);
}

#[test]
fn test_whitespace_tolerance() {
    let input = format!(
        "{} MANIFEST {}\n   src/file1.rs\n  src/file2.rs [NEW]\n{}",
        nabla(), nabla(), delta()
    );
    let entries = manifest::parse_manifest(&input).unwrap().unwrap();
    assert_eq!(entries[0].path, "src/file1.rs");
    assert_eq!(entries[1].path, "src/file2.rs");
}

#[test]
fn test_legacy_format() {
    let input = r#"
<delivery>
src/main.rs
src/lib.rs [NEW]
</delivery>
"#;
    let entries = manifest::parse_manifest(input).unwrap().unwrap();
    assert!(!entries.is_empty(), "Should parse legacy format");
    assert_eq!(entries[1].operation, Operation::New);
}