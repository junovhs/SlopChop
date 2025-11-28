fn test_unwrap_detection() {
    let x: Option<i32> = Some(5);
    let _ = x.unwrap(); // Should be caught: LAW OF PARANOIA
}

fn test_expect_detection() {
    let y: Option<&str> = Some("hello");
    let _ = y.expect("this should fail"); // Should be caught: LAW OF PARANOIA
}

fn test_chained_unwrap() {
    let z: Result<Option<i32>, &str> = Ok(Some(42));
    let _ = z.unwrap().unwrap(); // Should catch BOTH
}

fn test_ok_patterns() {
    // These should NOT be flagged:
    let a: Option<i32> = Some(5);
    let _ = a.unwrap_or(0);
    let _ = a.unwrap_or_else(|| 10);

    let b: Result<i32, &str> = Ok(5);
    let _ = b.unwrap_or_default();
    let _ = b.ok();
}
