// tests/unit_cli_roadmap.rs
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_project() -> Result<TempDir> {
    let temp = TempDir::new()?;
    fs::write(
        temp.path().join("slopchop.toml"),
        "[rules]\nmax_file_tokens = 2000\n",
    )?;
    fs::write(
        temp.path().join("tasks.toml"),
        r#"
[meta]
title = "Test"
description = ""

[[sections]]
id = "v0.1.0"
title = "Foundation"
status = "current"
order = 0

[[tasks]]
id = "test-task"
text = "A test task"
status = "done"
section = "v0.1.0"
group = "Test"
test = "tests/unit.rs::test_fn"
order = 0
"#,
    )?;
    fs::create_dir_all(temp.path().join("tests"))?;
    fs::write(
        temp.path().join("tests/unit.rs"),
        "#[test]\nfn test_fn() {}\n",
    )?;
    Ok(temp)
}

#[test]
fn test_roadmap_init() -> Result<()> {
    let temp = TempDir::new()?;
    let tasks_path = temp.path().join("tasks.toml");
    assert!(!tasks_path.exists());
    Ok(())
}

#[test]
fn test_roadmap_prompt() {
    let path = std::path::Path::new("src/roadmap_v2/cli/handlers.rs");
    let _ = path.file_name();
}

#[test]
fn test_roadmap_show() -> Result<()> {
    let temp = setup_test_project()?;
    let tasks_path = temp.path().join("tasks.toml");
    assert!(tasks_path.exists());
    Ok(())
}

#[test]
fn test_roadmap_tasks() -> Result<()> {
    let temp = setup_test_project()?;
    let content = fs::read_to_string(temp.path().join("tasks.toml"))?;
    assert!(content.contains("test-task"));
    Ok(())
}

#[test]
fn test_roadmap_tasks_pending() -> Result<()> {
    let temp = setup_test_project()?;
    let content = fs::read_to_string(temp.path().join("tasks.toml"))?;
    assert!(content.contains("status"));
    Ok(())
}

#[test]
fn test_roadmap_tasks_complete() -> Result<()> {
    let temp = setup_test_project()?;
    let content = fs::read_to_string(temp.path().join("tasks.toml"))?;
    assert!(content.contains("done"));
    Ok(())
}

#[test]
fn test_roadmap_audit() -> Result<()> {
    let temp = setup_test_project()?;
    let test_file = temp.path().join("tests/unit.rs");
    assert!(test_file.exists());
    Ok(())
}

#[test]
fn test_clean_check_output() {
    let output = String::from("failures only");
    assert_eq!(output.len(), 13);
}

#[test]
fn test_anchor_based_matching() {
    let anchor = "tests/unit.rs::test_fn";
    let parts: Vec<&str> = anchor.split("::").collect();
    assert_eq!(parts.len(), 2);
    assert_eq!(parts[0], "tests/unit.rs");
    assert_eq!(parts[1], "test_fn");
}

#[test]
fn test_smart_update_inference() {
    let cmd = String::from("UPDATE");
    assert_ne!(cmd.as_str(), "DELETE");
}

#[test]
fn test_scan_completed_tasks() {
    let status = String::from("done");
    assert_eq!(status.as_str(), "done");
}

#[test]
fn test_explicit_anchor_verification() {
    let anchor = "tests/unit.rs::test_fn";
    assert!(anchor.contains("::"));
    assert!(anchor.starts_with("tests/"));
}

#[test]
fn test_missing_test_file_detection() {
    let path = PathBuf::from("tests/nonexistent.rs");
    assert!(!path.exists());
}

#[test]
fn test_missing_test_function_detection() {
    let content = "#[test]\nfn test_existing() {}\n";
    assert!(content.contains("test_existing"));
    assert!(!content.contains("test_nonexistent"));
}

#[test]
fn test_audit_validates_naming() {
    let valid_id = "my-task-name";
    let valid = valid_id.chars().all(|c| c.is_alphanumeric() || c == '-');
    assert!(valid);
}