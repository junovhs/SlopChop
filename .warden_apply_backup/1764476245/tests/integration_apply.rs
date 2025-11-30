use warden_core::apply::extractor;

#[test]
fn test_extract_single_file() {
    let input = r#"
∇∇∇ src/main.rs ∇∇∇
fn main() {
    println!();
}