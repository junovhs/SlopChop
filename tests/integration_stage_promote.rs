// tests/integration_stage_promote.rs
//! Tests for Promotion and Rollback logic.

use anyhow::Result;
use slopchop_core::stage::StageManager;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test repository structure.
fn create_test_repo() -> Result<TempDir> {
    let repo = TempDir::new()?;

    // Create typical repo structure
    fs::write(
        repo.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\n",
    )?;
    fs::create_dir(repo.path().join("src"))?;
    fs::write(
        repo.path().join("src/main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )?;
    fs::write(
        repo.path().join("src/lib.rs"),
        "pub fn greet() -> &'static str {\n    \"hello\"\n}\n",
    )?;

    Ok(repo)
}

#[test]
fn test_promote_only_applies_touched_paths() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Modify ONE file in stage
    fs::write(manager.worktree().join("src/main.rs"), "// modified\n")?;
    manager.record_write("src/main.rs")?;

    // Modify another file in stage WITHOUT recording it
    fs::write(manager.worktree().join("src/lib.rs"), "// also modified\n")?;

    // Store original lib.rs content
    let original_lib = fs::read_to_string(repo.path().join("src/lib.rs"))?;

    // Promote
    let result = manager.promote(3)?;

    // Only main.rs should be promoted (it was recorded)
    assert_eq!(result.files_written, vec!["src/main.rs"]);

    // main.rs should be updated in real workspace
    let main_content = fs::read_to_string(repo.path().join("src/main.rs"))?;
    assert_eq!(main_content, "// modified\n");

    // lib.rs should be UNCHANGED (not recorded)
    let lib_content = fs::read_to_string(repo.path().join("src/lib.rs"))?;
    assert_eq!(lib_content, original_lib);

    Ok(())
}

#[test]
fn test_promote_handles_deletions() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Create a file to delete
    fs::write(repo.path().join("to_delete.rs"), "// delete me")?;

    // Record deletion
    manager.record_delete("to_delete.rs")?;

    // Promote
    let result = manager.promote(3)?;

    assert!(result.files_deleted.contains(&"to_delete.rs".to_string()));
    assert!(!repo.path().join("to_delete.rs").exists());

    Ok(())
}

#[test]
fn test_promote_creates_new_files() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Create new file in stage
    fs::create_dir_all(manager.worktree().join("src/subdir"))?;
    fs::write(manager.worktree().join("src/subdir/new.rs"), "// brand new")?;
    manager.record_write("src/subdir/new.rs")?;

    // Verify it doesn't exist in real workspace yet
    assert!(!repo.path().join("src/subdir/new.rs").exists());

    // Promote
    manager.promote(3)?;

    // Now it should exist
    assert!(repo.path().join("src/subdir/new.rs").exists());
    let content = fs::read_to_string(repo.path().join("src/subdir/new.rs"))?;
    assert_eq!(content, "// brand new");

    Ok(())
}

#[test]
fn test_promote_creates_backup() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Original content
    let original = fs::read_to_string(repo.path().join("src/main.rs"))?;

    // Modify in stage
    fs::write(manager.worktree().join("src/main.rs"), "// modified")?;
    manager.record_write("src/main.rs")?;

    // Promote
    let result = manager.promote(3)?;

    // Backup should exist
    assert!(result.backup_path.is_some());
    let backup_path = result.backup_path.ok_or_else(|| anyhow::anyhow!("Backup path missing"))?;
    let backup_main = backup_path.join("src/main.rs");
    assert!(backup_main.exists());

    // Backup should contain original content
    let backup_content = fs::read_to_string(backup_main)?;
    assert_eq!(backup_content, original);

    Ok(())
}

#[test]
fn test_promote_clears_touched_paths() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Write a file
    fs::write(manager.worktree().join("new.rs"), "// new")?;
    manager.record_write("new.rs")?;

    assert!(!manager.state().ok_or_else(|| anyhow::anyhow!("State missing"))?.paths_to_write().is_empty());

    // Promote
    manager.promote(3)?;

    // Touched paths should be cleared
    assert!(manager.state().ok_or_else(|| anyhow::anyhow!("State missing"))?.paths_to_write().is_empty());

    Ok(())
}