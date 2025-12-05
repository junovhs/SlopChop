// tests/unit_cli_check.rs
//! Tests for CLI check output formatting.

#[test]
fn test_filtered_output_extracts_failures() {
    let test_output = r"
running 5 tests
test test_one ... ok
test test_two ... ok
test test_three ... FAILED
test test_four ... ok
test test_five ... FAILED

failures:
    test_three
    test_five

test result: FAILED. 3 passed; 2 failed; 0 ignored
";

    let failures: Vec<&str> = test_output
        .lines()
        .filter(|l| l.contains("FAILED"))
        .map(str::trim)
        .collect();

    assert_eq!(failures.len(), 3); // 2 test lines + 1 result line
    assert!(failures[0].contains("test_three"));
}

#[test]
fn test_clippy_error_extraction() {
    let clippy_output = r"
    Checking foo v0.1.0
error[E0425]: cannot find function `bar` in this scope
  --> src/main.rs:5:5
   |
5  |     bar();
   |     ^^^ not found in this scope

error: could not compile `foo` due to previous error
";

    let errors: Vec<&str> = clippy_output
        .lines()
        .filter(|l| l.trim().starts_with("error[") || l.trim().starts_with("error:"))
        .map(str::trim)
        .collect();

    assert_eq!(errors.len(), 2);
    assert!(errors[0].contains("E0425"));
}