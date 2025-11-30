use warden_core::apply::extractor;

#[test]
fn test_malformed_block_skipped() {
    let input = "∇∇∇ src/broken.rs\nmissing closer";
    let files = extractor::extract_files(input).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_extract_single_file() {
    // Avoid putting ∆∆∆ at start of line to prevent self-truncation during apply
    let input = "∇∇∇ src/valid.rs ∇∇∇\nfn valid() {}\n∆∆∆";
    let files = extractor::extract_files(input).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files["src/valid.rs"].content, "fn valid() {}");
}

#[test]
fn test_extract_with_markdown() {
    let expected_content = "fn main() {}";
    let input = format!("∇∇∇ test.rs ∇∇∇\n{expected_content}\n∆∆∆");
    
    let files = extractor::extract_files(&input).unwrap();
    assert_eq!(files["test.rs"].content, expected_content);
}

#[test]
fn test_multiple_files() {
    let input = "\
∇∇∇ a.txt ∇∇∇
A