// tests/integration_backup.rs
use slopchop_core::apply::types::{FileContent, ManifestEntry, Operation};
use slopchop_core::apply::writer;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_backup_dir_created() -> Result<()> {
    let d = tempdir()?;
    let backup = d.path().join(".slopchop_apply_backup");

    let manifest = vec![ManifestEntry {
        path: "test.txt".to_string(),
        operation: Operation::Update,
    }];

    fs::write(d.path().join("test.txt"), "original")?;

    let files = HashMap::new();
    writer::write_files(&manifest, &files, Some(d.path()), 5)?;

    assert!(backup.exists());
    Ok(())
}

#[test]
fn test_timestamp_folder() -> Result<()> {
    let d = tempdir()?;
    let manifest = vec![ManifestEntry {
        path: "test.txt".to_string(),
        operation: Operation::Update,
    }];
    fs::write(d.path().join("test.txt"), "original")?;

    writer::write_files(&manifest, &HashMap::new(), Some(d.path()), 5)?;

    let backup_root = d.path().join(".slopchop_apply_backup");
    let entries: Vec<_> = fs::read_dir(backup_root)?.collect();

    assert!(
        !entries.is_empty(),
        "Should have created a timestamp directory"
    );

    let first_entry = entries[0]
        .as_ref()
        .map_err(std::string::ToString::to_string)?;
    assert!(first_entry.path().is_dir());
    Ok(())
}

#[test]
fn test_existing_backed_up() -> Result<()> {
    let d = tempdir()?;
    let file_path = "config.toml";

    fs::write(d.path().join(file_path), "version = 1")?;

    let manifest = vec![ManifestEntry {
        path: file_path.to_string(),
        operation: Operation::Update,
    }];
    let mut files = HashMap::new();
    files.insert(
        file_path.to_string(),
        FileContent {
            content: "version = 2".to_string(),
            line_count: 1,
        },
    );

    writer::write_files(&manifest, &files, Some(d.path()), 5)?;

    let backup_root = d.path().join(".slopchop_apply_backup");
    let timestamp_dir = fs::read_dir(backup_root)?
        .next()
        .ok_or("No backup dir")??
        .path();
    let backed_up_file = timestamp_dir.join(file_path);

    let content = fs::read_to_string(backed_up_file)?;
    assert_eq!(content, "version = 1");
    Ok(())
}

#[test]
fn test_new_file_no_backup() -> Result<()> {
    let d = tempdir()?;
    let file_path = "new.txt";

    let manifest = vec![ManifestEntry {
        path: file_path.to_string(),
        operation: Operation::New,
    }];
    let mut files = HashMap::new();
    files.insert(
        file_path.to_string(),
        FileContent {
            content: "hello".to_string(),
            line_count: 1,
        },
    );

    writer::write_files(&manifest, &files, Some(d.path()), 5)?;

    let backup_root = d.path().join(".slopchop_apply_backup");
    if backup_root.exists() {
        let count = fs::read_dir(backup_root)?.count();
        assert_eq!(
            count, 0,
            "Should not create backup folder for strictly new files"
        );
    }
    Ok(())
}

#[test]
fn test_path_structure() -> Result<()> {
    let d = tempdir()?;
    let deep_path = "src/nested/mod.rs";
    let full_path = d.path().join(deep_path);

    fs::create_dir_all(full_path.parent().ok_or("no parent")?)?;
    fs::write(&full_path, "old")?;

    let manifest = vec![ManifestEntry {
        path: deep_path.to_string(),
        operation: Operation::Update,
    }];

    writer::write_files(&manifest, &HashMap::new(), Some(d.path()), 5)?;

    let backup_root = d.path().join(".slopchop_apply_backup");
    let timestamp_dir = fs::read_dir(backup_root)?
        .next()
        .ok_or("No backup")??
        .path();

    let backed_up = timestamp_dir.join(deep_path);
    assert!(backed_up.exists());
    assert_eq!(fs::read_to_string(backed_up)?, "old");
    Ok(())
}