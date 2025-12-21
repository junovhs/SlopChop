// tests/integration_stage_lifecycle.rs
//! Tests for Stage creation, Apply, Check, and Reset lifecycle.

use anyhow::Result;
use slopchop_core::stage::{effective_cwd, stage_exists, worktree_path, StageManager};
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

    // Create some dirs that should be excluded
    fs::create_dir(repo.path().join(".git"))?;
    fs::write(repo.path().join(".git/config"), "[core]\n")?;

    Ok(repo)
}

#[test]
fn test_stage_creates_worktree_dir() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    assert!(!stage_exists(repo.path()));

    manager.ensure_stage()?;

    assert!(stage_exists(repo.path()));
    assert!(worktree_path(repo.path()).is_dir());

    Ok(())
}

#[test]
fn test_stage_does_not_copy_slopchop_into_itself() -> Result<()> {
    let repo = create_test_repo()?;

    // Create a .slopchop dir with some content
    fs::create_dir_all(repo.path().join(".slopchop/old_state"))?;
    fs::write(repo.path().join(".slopchop/old_state/data.json"), "{}")?;

    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Verify .slopchop is NOT in the stage worktree
    let worktree = manager.worktree();
    assert!(!worktree.join(".slopchop").exists());

    // But regular files ARE copied
    assert!(worktree.join("src/main.rs").exists());
    assert!(worktree.join("Cargo.toml").exists());

    Ok(())
}

#[test]
fn test_stage_does_not_copy_git() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    manager.ensure_stage()?;

    let worktree = manager.worktree();
    assert!(!worktree.join(".git").exists());

    Ok(())
}

#[test]
fn test_apply_writes_to_stage_not_real_workspace() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    // Get original content
    let original_content = fs::read_to_string(repo.path().join("src/main.rs"))?;

    manager.ensure_stage()?;

    // Simulate an apply by writing to the stage worktree
    let new_content = "fn main() {\n    println!(\"modified\");\n}\n";
    fs::write(manager.worktree().join("src/main.rs"), new_content)?;
    manager.record_write("src/main.rs")?;

    // Real workspace should be UNCHANGED
    let real_content = fs::read_to_string(repo.path().join("src/main.rs"))?;
    assert_eq!(real_content, original_content);

    // Stage should have the modification
    let staged_content = fs::read_to_string(manager.worktree().join("src/main.rs"))?;
    assert_eq!(staged_content, new_content);

    Ok(())
}

#[test]
fn test_stage_tracks_written_paths() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    // Record some writes
    manager.record_write("src/main.rs")?;
    manager.record_write("src/new_file.rs")?;
    manager.record_delete("src/old_file.rs")?;

    let state = manager.state().ok_or_else(|| anyhow::anyhow!("State should exist"))?;
    let writes = state.paths_to_write();
    let deletes = state.paths_to_delete();

    assert!(writes.contains(&"src/main.rs"));
    assert!(writes.contains(&"src/new_file.rs"));
    assert!(deletes.contains(&"src/old_file.rs"));

    Ok(())
}

#[test]
fn test_effective_cwd_uses_stage_when_present() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    // Before stage: effective_cwd is repo root
    let cwd_before = effective_cwd(repo.path());
    assert_eq!(cwd_before, repo.path());

    manager.ensure_stage()?;

    // After stage: effective_cwd is worktree
    let cwd_after = effective_cwd(repo.path());
    assert_eq!(cwd_after, worktree_path(repo.path()));

    Ok(())
}

#[test]
fn test_effective_cwd_falls_back_to_repo_without_stage() -> Result<()> {
    let repo = create_test_repo()?;

    // No stage exists
    let cwd = effective_cwd(repo.path());
    assert_eq!(cwd, repo.path());

    Ok(())
}

#[test]
fn test_stage_reset_removes_everything() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    manager.ensure_stage()?;
    assert!(stage_exists(repo.path()));

    manager.reset()?;
    assert!(!stage_exists(repo.path()));

    // State should be gone too
    assert!(manager.state().is_none());

    Ok(())
}

#[test]
fn test_stage_id_persists_across_loads() -> Result<()> {
    let repo = create_test_repo()?;

    let id = {
        let mut manager = StageManager::new(repo.path());
        manager.ensure_stage()?;
        manager.stage_id().ok_or_else(|| anyhow::anyhow!("ID missing"))?.to_string()
    };

    // Load with fresh manager
    let mut manager2 = StageManager::new(repo.path());
    manager2.load_state()?;

    assert_eq!(manager2.stage_id().ok_or_else(|| anyhow::anyhow!("ID missing"))?, id);

    Ok(())
}

#[test]
fn test_apply_count_increments() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());
    manager.ensure_stage()?;

    assert_eq!(manager.apply_count(), 0);

    manager.record_apply()?;
    assert_eq!(manager.apply_count(), 1);

    manager.record_apply()?;
    assert_eq!(manager.apply_count(), 2);

    Ok(())
}

#[test]
fn test_ensure_stage_is_idempotent() -> Result<()> {
    let repo = create_test_repo()?;
    let mut manager = StageManager::new(repo.path());

    // First call creates
    let result1 = manager.ensure_stage()?;
    assert!(result1.was_created());

    let id = manager.stage_id().ok_or_else(|| anyhow::anyhow!("ID missing"))?.to_string();

    // Second call reuses
    let result2 = manager.ensure_stage()?;
    assert!(!result2.was_created());

    // Same stage ID
    assert_eq!(manager.stage_id().ok_or_else(|| anyhow::anyhow!("ID missing"))?, id);

    Ok(())
}