use warden_core::apply::manifest;
use warden_core::apply::types::Operation;

#[test]
fn test_parse_manifest() {
    let input = "\
∇∇∇ MANIFEST ∇∇∇
src/main.rs
src/lib.rs