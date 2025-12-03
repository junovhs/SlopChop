// src/roadmap/audit/scanner.rs
//! Test file discovery via filesystem scanning.

use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub fn scan_test_files(root: &Path) -> Vec<String> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
        .flatten()
        .filter(is_test_file)
        .filter_map(|e| e.path().to_str().map(str::to_lowercase))
        .collect()
}

fn is_ignored_dir(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor"
}

fn is_test_file(entry: &DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }
    if !has_code_ext(entry.path()) {
        return false;
    }
    let Some(name) = entry.file_name().to_str() else {
        return false;
    };
    name.contains("test")
        || name.contains("spec")
        || entry.path().components().any(|c| c.as_os_str() == "tests")
}

fn has_code_ext(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| matches!(e, "rs" | "ts" | "js" | "py" | "go"))
}
