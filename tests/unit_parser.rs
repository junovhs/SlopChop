// tests/unit_parser.rs
//! Tests for roadmap parser edge cases.

use slopchop_core::roadmap::types::Roadmap;

#[test]
fn test_empty_id_skipped() {
    // Task with only whitespace/symbols that would generate empty slug
    let content = r"# Test Roadmap

## v0.1.0 - Section

- [ ] **** <!-- empty bold -->
- [ ] **Valid task**
- [ ] **   ** <!-- whitespace only -->
";
    let roadmap = Roadmap::parse(content);

    // Should only have the valid task, empty IDs filtered
    let tasks: Vec<_> = roadmap
        .sections
        .iter()
        .flat_map(|s| &s.tasks)
        .filter(|t| !t.id.is_empty())
        .collect();

    assert_eq!(tasks.len(), 1, "Should filter empty ID tasks");
    assert!(tasks[0].text.contains("Valid task"));
}

#[test]
fn test_id_collision_resolved() {
    let content = r"# Test

## v0.1.0

- [ ] **Feature A**
- [ ] **Feature A** <!-- duplicate text -->
";
    let roadmap = Roadmap::parse(content);
    let tasks: Vec<_> = roadmap.sections.iter().flat_map(|s| &s.tasks).collect();

    // Both should exist with unique IDs
    assert_eq!(tasks.len(), 2);
    assert_ne!(tasks[0].id, tasks[1].id, "Duplicate tasks should have unique IDs");
}

#[test]
fn test_anchor_id_extraction() {
    let content = r"# Test

## v0.1.0

- [ ] **Feature** <!-- test: tests/foo.rs::test_bar -->
";
    let roadmap = Roadmap::parse(content);
    let task = &roadmap.sections[0].tasks[0];

    assert!(!task.tests.is_empty(), "Should extract test anchor");
    assert!(task.tests[0].contains("test_bar"));
}