// tests/integration_skeleton.rs
use slopchop_core::skeleton;
use std::path::Path;

fn check_clean(filename: &str, code: &str, expected_parts: &[&str]) {
    let result = skeleton::clean(Path::new(filename), code);
    for part in expected_parts {
        assert!(
            result.contains(part),
            "Cleaned code missing '{part}'. Result:\n{result}"
        );
    }
}

#[test]
fn test_clean_languages() {
    // Rust Basic
    check_clean(
        "test.rs",
        "fn main() {\n    println!(\"hi\");\n}",
        &["fn main", "{ ... }"],
    );

    // Rust Nested
    check_clean(
        "test.rs",
        "fn outer() {\n    fn inner() { 42 }\n    inner()\n}",
        &["fn outer", "{ ... }"],
    );

    // Rust Impl
    check_clean(
        "test.rs",
        "impl Foo {\n    fn bar(&self) { 42 }\n}",
        &["impl", "Foo"],
    );

    // Python
    check_clean(
        "test.py",
        "def hello():\n    print('hi')\n",
        &["def hello", "..."],
    );

    // TypeScript
    check_clean(
        "test.ts",
        "function hello() {\n    console.log('hi');\n}",
        &["function hello", "{ ... }"],
    );
}

#[test]
fn test_clean_unsupported_extension() {
    let code = "some random text";
    let result = skeleton::clean(Path::new("test.xyz"), code);
    assert_eq!(result, code);
}