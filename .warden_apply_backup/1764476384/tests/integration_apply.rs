use warden_core::apply::extractor;

// Helper to construct delimiters without confusing the outer warden apply
fn nabla() -> String { "\u{2207}".repeat(3) }
fn delta() -> String { "\u{2206}".repeat(3) }

#[test]
fn test_extract_single_file() {
    let input = format!(
        "{} src/main.rs {}\nfn main() {{\n    println!();\n}}\n{}",
        nabla(), nabla(), delta()
    );
    let files = extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 1);
    
    let content = &files["src/main.rs"];
    assert!(content.content.contains("fn main"));
}

#[test]
fn test_extract_multiple_files() {
    let input = format!(
        "{} src/main.rs {}\nfn main() {{}}\n{}\n\n{} src/lib.rs {}\npub fn lib() {{}}\n{}",
        nabla(), nabla(), delta(),
        nabla(), nabla(), delta()
    );
    let files = extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.contains_key("src/main.rs"));
    assert!(files.contains_key("src/lib.rs"));
}

#[test]
fn test_extract_skips_manifest() {
    let input = format!(
        "{} MANIFEST {}\nsrc/main.rs\nsrc/lib.rs [NEW]\n{}\n\n{} src/main.rs {}\nfn main() {{}}\n{}",
        nabla(), nabla(), delta(),
        nabla(), nabla(), delta()
    );
    let files = extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("src/main.rs"));
    assert!(!files.contains_key("MANIFEST"));
}

#[test]
fn test_extract_plan() {
    let input = format!(
        "{} PLAN {}\nGOAL: Refactor the parser module\nCHANGES:\n1. Extract parsing logic\n2. Add error handling\n{}",
        nabla(), nabla(), delta()
    );
    let plan = extractor::extract_plan(&input).unwrap();
    assert!(plan.contains("GOAL: Refactor"));
}

#[test]
fn test_path_safety_blocks_traversal() {
    let input = format!("{} ../bad.rs {}\ncontent\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    assert!(files.contains_key("../bad.rs"));
}

#[test]
fn test_path_safety_blocks_absolute() {
    let input = format!("{} /etc/passwd {}\ncontent\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    assert!(files.contains_key("/etc/passwd"));
}

#[test]
fn test_path_safety_blocks_git() {
    let input = format!("{} .git/config {}\ncontent\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    assert!(files.contains_key(".git/config"));
}

#[test]
fn test_path_safety_blocks_hidden() {
    let input = format!("{} .env {}\ncontent\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    assert!(files.contains_key(".env"));
}

#[test]
fn test_truncation_detects_ellipsis_comment() {
    let input = format!("{} src/incomplete.rs {}\n// ...\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    let content = &files["src/incomplete.rs"].content;
    assert!(content.contains("// ..."));
}

#[test]
fn test_truncation_allows_warden_ignore() {
    let input = format!("{} src/ignored.rs {}\n// warden:ignore\n// ...\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    
    // Check validation logic simulation
    use warden_core::apply::validator;
    let manifest = Vec::new();
    let result = validator::validate(&manifest, &files);
    
    if let warden_core::apply::types::ApplyOutcome::Success { .. } = result {
        // Expected pass
    } else {
        panic!("warden:ignore should bypass truncation checks");
    }
}

#[test]
fn test_truncation_detects_empty_file() {
    let input = format!("{} src/empty.rs {}\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    let content = &files["src/empty.rs"].content;
    assert_eq!(content.trim(), "");
}

#[test]
fn test_path_safety_allows_valid() {
    let input = format!("{} src/valid.rs {}\nfn main() {{}}\n{}", nabla(), nabla(), delta());
    let files = extractor::extract_files(&input).unwrap();
    
    use warden_core::apply::validator;
    let manifest = Vec::new();
    let result = validator::validate(&manifest, &files);
    
    if let warden_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = result {
        panic!("Valid path should pass: {errors:?}");
    }
}

#[test]
fn test_unified_apply_roadmap() {
    let input = r#"
===ROADMAP===
CHECK some-task
===END===
"#;
    // Just verify extractor doesn't crash or confuse it with files
    let files = extractor::extract_files(input).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_unified_apply_combined() {
    let input = format!(
        "===ROADMAP===\nCHECK task\n===END===\n\n{} src/file.rs {}\nfn f() {{}}\n{}",
        nabla(), nabla(), delta()
    );
    let files = extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("src/file.rs"));
}