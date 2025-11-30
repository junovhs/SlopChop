use std::fs;
use tempfile::tempdir;
use warden_core::apply::{writer, types::ManifestEntry, types::Operation};
use std::collections::HashMap;
use warden_core::apply::types::FileContent;

#[test]
fn test_backup_dir_created() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "original").unwrap();

    let manifest = vec![ManifestEntry {
        path: "test.rs".to_string(),
        operation: Operation::Update,
    }];
    
    let mut files = HashMap::new();
    files.insert("test.rs".to_string(), FileContent { content: "new".to_string(), line_count: 1 });

    writer::write_files(&manifest, &files, Some(dir.path())).unwrap();

    let backup_root = dir.path().join(".warden_apply_backup");
    assert!(backup_root.exists(), "Backup dir should exist");
}

#[test]
fn test_timestamp_folder() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "original").unwrap();

    let manifest = vec![ManifestEntry {
        path: "test.rs".to_string(),
        operation: Operation::Update,
    }];
    
    let mut files = HashMap::new();
    files.insert("test.rs".to_string(), FileContent { content: "new".to_string(), line_count: 1 });

    writer::write_files(&manifest, &files, Some(dir.path())).unwrap();

    let backup_root = dir.path().join(".warden_apply_backup");
    let entries = fs::read_dir(backup_root).unwrap();
    
    let folders: Vec<_> = entries
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();
        
    assert!(!folders.is_empty());
    
    let name = folders[0].file_name();
    let name_str = name.to_string_lossy();
    assert!(name_str.chars().all(|c| c.is_numeric()));
}

#[test]
fn test_existing_backed_up() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rs");
    fs::write(&file_path, "original content").unwrap();

    let manifest = vec![ManifestEntry {
        path: "test.rs".to_string(),
        operation: Operation::Update,
    }];
    
    let mut files = HashMap::new();
    files.insert("test.rs".to_string(), FileContent { content: "new content".to_string(), line_count: 1 });

    writer::write_files(&manifest, &files, Some(dir.path())).unwrap();

    let backup_root = dir.path().join(".warden_apply_backup");
    let timestamp_dir = fs::read_dir(backup_root).unwrap()
        .filter_map(std::result::Result::ok)
        .next().unwrap().path();
        
    let backup_file = timestamp_dir.join("test.rs");
    assert!(backup_file.exists());
    
    let content = fs::read_to_string(backup_file).unwrap();
    assert_eq!(content, "original content");
}

#[test]
fn test_new_file_no_backup() {
    let dir = tempdir().unwrap();
    // File is absent

    let manifest = vec![ManifestEntry {
        path: "new.rs".to_string(),
        operation: Operation::New,
    }];
    
    let mut files = HashMap::new();
    files.insert("new.rs".to_string(), FileContent { content: "new".to_string(), line_count: 1 });

    writer::write_files(&manifest, &files, Some(dir.path())).unwrap();

    let backup_root = dir.path().join(".warden_apply_backup");
    // Backup dir might not even be created if nothing to backup
    if backup_root.exists() {
        let entries = fs::read_dir(backup_root).unwrap();
        assert_eq!(entries.count(), 0);
    }
}

#[test]
fn test_path_structure() {
    let dir = tempdir().unwrap();
    let nested = dir.path().join("src/nested");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("file.rs"), "orig").unwrap();

    let manifest = vec![ManifestEntry {
        path: "src/nested/file.rs".to_string(),
        operation: Operation::Update,
    }];
    
    let mut files = HashMap::new();
    files.insert("src/nested/file.rs".to_string(), FileContent { content: "new".to_string(), line_count: 1 });

    writer::write_files(&manifest, &files, Some(dir.path())).unwrap();

    let backup_root = dir.path().join(".warden_apply_backup");
    let timestamp_dir = fs::read_dir(backup_root).unwrap()
        .filter_map(std::result::Result::ok)
        .next().unwrap().path();

    assert!(timestamp_dir.join("src/nested/file.rs").exists());
}