use warden_core::apply::extractor;

#[test]
fn test_malformed_block_skipped() {
    let input = "∇∇∇ src/broken.rs\nmissing closer";
    let files = extractor::extract_files(input).unwrap();
    assert!(files.is_empty());
}

#[test]
fn test_extract_single_file() {
    let input = r#"
∇∇∇ src/valid.rs ∇∇∇
fn valid() {}