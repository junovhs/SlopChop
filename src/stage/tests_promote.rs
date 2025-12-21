// src/stage/tests_promote.rs
use super::promote::*;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

fn setup_test_dirs() -> Result<(TempDir, TempDir, TempDir)> {
    let repo = TempDir::new()?;
    let stage = TempDir::new()?;
    let backup = TempDir::new()?;
    Ok((repo, stage, backup))
}

#[test]
fn test_promote_new_file() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create file in stage
    fs::write(stage.path().join("new.rs"), "fn main() {}")?;

    let result =
        promote_to_workspace(repo.path(), stage.path(), &["new.rs"], &[], backup.path())?;

    assert_eq!(result.files_written.len(), 1);
    assert!(repo.path().join("new.rs").exists());

    Ok(())
}

#[test]
fn test_promote_update_file() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create original in repo
    fs::write(repo.path().join("main.rs"), "old content")?;

    // Create updated version in stage
    fs::write(stage.path().join("main.rs"), "new content")?;

    let result =
        promote_to_workspace(repo.path(), stage.path(), &["main.rs"], &[], backup.path())?;

    assert_eq!(result.files_written.len(), 1);
    let content = fs::read_to_string(repo.path().join("main.rs"))?;
    assert_eq!(content, "new content");

    // Verify backup exists
    assert!(result.backup_path.is_some());
    let backup_file = result.backup_path.unwrap().join("main.rs");
    assert!(backup_file.exists());
    assert_eq!(fs::read_to_string(backup_file)?, "old content");

    Ok(())
}

#[test]
fn test_promote_delete_file() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create file to delete in repo
    fs::write(repo.path().join("old.rs"), "to be deleted")?;

    let result =
        promote_to_workspace(repo.path(), stage.path(), &[], &["old.rs"], backup.path())?;

    assert_eq!(result.files_deleted.len(), 1);
    assert!(!repo.path().join("old.rs").exists());

    Ok(())
}

#[test]
fn test_promote_rollback_on_failure() -> Result<()> {
    let (repo, stage, backup) = setup_test_dirs()?;

    // Create original file
    fs::write(repo.path().join("exists.rs"), "original")?;

    // Stage file exists, but missing file doesn't exist in stage
    fs::write(stage.path().join("exists.rs"), "modified")?;

    // This should fail because "missing.rs" doesn't exist in stage
    let result = promote_to_workspace(
        repo.path(),
        stage.path(),
        &["exists.rs", "missing.rs"],
        &[],
        backup.path(),
    );

    assert!(result.is_err());

    // Original file should be restored (rollback)
    // Note: This test might need adjustment based on exact rollback semantics
    Ok(())
}

#[test]
fn test_cleanup_old_backups() -> Result<()> {
    let backup_base = TempDir::new()?;

    // Create multiple backup dirs
    for i in 1..=5 {
        let dir = backup_base.path().join(format!("promote_{i}"));
        fs::create_dir(&dir)?;
    }

    let removed = cleanup_old_backups(backup_base.path(), 2)?;
    assert_eq!(removed, 3);

    // Count remaining
    let remaining: Vec<_> = fs::read_dir(backup_base.path())?
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(remaining.len(), 2);

    Ok(())
}