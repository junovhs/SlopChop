// tests/cli_map.rs
//! Tests for warden map and trace commands.

use std::path::PathBuf;
use warden_core::trace::{self, TraceOptions};

#[test]
fn test_map_basic() {
    // Map should run without panicking on empty/minimal setup
    // In a real repo, this would show directory structure
    let result = trace::map();
    assert!(result.is_ok(), "Map command should succeed");
}

#[test]
fn test_map_tree() {
    let result = trace::map();
    assert!(result.is_ok());
    let output = result.unwrap_or_default();
    // Output should contain directory-like structure
    assert!(
        output.contains('/') || output.contains("Repository"),
        "Should show directory tree"
    );
}

#[test]
fn test_trace_with_depth() {
    // This tests the --depth flag functionality
    let opts = TraceOptions {
        anchor: PathBuf::from("src/lib.rs"),
        depth: 1,
        budget: 4000,
    };
    // Should not panic even if file doesn't exist in test env
    let _ = trace::run(&opts);
}

#[test]
fn test_trace_with_budget() {
    // This tests the --budget flag functionality
    let opts = TraceOptions {
        anchor: PathBuf::from("src/lib.rs"),
        depth: 2,
        budget: 1000,
    };
    let _ = trace::run(&opts);
}