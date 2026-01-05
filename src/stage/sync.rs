// src/stage/sync.rs
//! Nuclear Sync - mirrors the staged worktree to the real workspace.
//!
//! This implements the "Agent-in-Stage" workflow where the stage becomes
//! the source of truth and is mirrored atomically to the workspace.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Paths that must NEVER be deleted or overwritten during sync.
/// These are infrastructure directories that would break the project.
const PRESERVED_DIRS: &[&str] = &[
    ".git",
    ".slopchop",
    "target",
    "node_modules",
    ".vscode",
    ".idea",
    "__pycache__",
    ".venv",
    "venv",
];

/// Files that must NEVER be deleted during sync.
const PRESERVED_FILES: &[&str] = &[".env", ".env.local", ".env.production"];

/// Result of a sync operation.
#[derive(Debug, Default)]
pub struct SyncResult {
    pub files_written: usize,
    pub files_deleted: usize,
    pub files_preserved: usize,
}

/// Checks if a path should be preserved (never deleted/overwritten).
///
/// This is the most critical safety function in the sync module.
#[must_use]
pub fn should_preserve(rel_path: &Path) -> bool {
    // Check if any component is a preserved directory
    for component in rel_path.components() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy();
            if PRESERVED_DIRS.contains(&name_str.as_ref()) {
                return true;
            }
        }
    }

    // Check if the file itself is preserved
    if let Some(file_name) = rel_path.file_name() {
        let name_str = file_name.to_string_lossy();
        if PRESERVED_FILES.contains(&name_str.as_ref()) {
            return true;
        }
    }

    false
}

/// Mirrors the stage worktree to the workspace root.
///
/// This is a "nuclear" sync that:
/// 1. Copies all files from stage to workspace (overwriting)
/// 2. Deletes files in workspace that don't exist in stage
/// 3. Preserves infrastructure paths (`.git`, `.slopchop`, etc.)
///
/// # Errors
/// Returns error if filesystem operations fail.
pub fn mirror_stage_to_workspace(repo_root: &Path, worktree: &Path) -> Result<SyncResult> {
    let mut result = SyncResult::default();

    // Phase 1: Collect all paths in stage
    let stage_paths = collect_relative_paths(worktree)?;

    // Phase 2: Copy all stage files to workspace
    for rel_path in &stage_paths {
        let src = worktree.join(rel_path);
        let dest = repo_root.join(rel_path);

        if src.is_file() {
            copy_file_with_parents(&src, &dest)?;
            result.files_written += 1;
        }
    }

    // Phase 3: Delete workspace files not in stage (respecting preserves)
    let workspace_paths = collect_relative_paths(repo_root)?;
    for rel_path in workspace_paths {
        // Skip if path exists in stage
        if stage_paths.contains(&rel_path) {
            continue;
        }

        // Skip preserved paths
        if should_preserve(&rel_path) {
            result.files_preserved += 1;
            continue;
        }

        let target = repo_root.join(&rel_path);
        if target.is_file() {
            fs::remove_file(&target)
                .with_context(|| format!("Failed to delete {}", target.display()))?;
            result.files_deleted += 1;
        }
    }

    // Phase 4: Clean up empty directories in workspace
    cleanup_empty_dirs(repo_root, &stage_paths)?;

    Ok(result)
}

/// Collects all file paths relative to the given root.
fn collect_relative_paths(root: &Path) -> Result<HashSet<PathBuf>> {
    let mut paths = HashSet::new();

    for entry in WalkDir::new(root).min_depth(1) {
        let entry = entry?;
        let rel_path = entry
            .path()
            .strip_prefix(root)
            .context("Failed to strip prefix")?;

        // Skip preserved directories entirely during collection
        if should_preserve(rel_path) {
            continue;
        }

        if entry.file_type().is_file() {
            paths.insert(rel_path.to_path_buf());
        }
    }

    Ok(paths)
}

/// Copies a file, creating parent directories as needed.
fn copy_file_with_parents(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create dir: {}", parent.display()))?;
    }
    fs::copy(src, dest)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;
    Ok(())
}

/// Removes empty directories that aren't in the stage.
fn cleanup_empty_dirs(repo_root: &Path, stage_paths: &HashSet<PathBuf>) -> Result<()> {
    // Collect directories from stage paths
    let stage_dirs: HashSet<PathBuf> = stage_paths
        .iter()
        .filter_map(|p| p.parent().map(Path::to_path_buf))
        .collect();

    // Walk workspace and find empty dirs not in stage
    let mut dirs_to_check: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(repo_root).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            let rel_path = entry.path().strip_prefix(repo_root)?;
            if !should_preserve(rel_path) {
                dirs_to_check.push(entry.path().to_path_buf());
            }
        }
    }

    // Sort by depth (deepest first) to remove children before parents
    dirs_to_check.sort_by_key(|p| std::cmp::Reverse(p.components().count()));

    for dir in dirs_to_check {
        let rel_path = dir.strip_prefix(repo_root)?;
        if !stage_dirs.contains(rel_path) && is_empty_dir(&dir)? {
            let _ = fs::remove_dir(&dir); // Best-effort, may fail if not empty
        }
    }

    Ok(())
}

/// Checks if a directory is empty.
fn is_empty_dir(path: &Path) -> Result<bool> {
    Ok(fs::read_dir(path)?.next().is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_preserve_git() {
        assert!(should_preserve(Path::new(".git")));
        assert!(should_preserve(Path::new(".git/config")));
        assert!(should_preserve(Path::new(".git/objects/abc")));
    }

    #[test]
    fn test_should_preserve_slopchop() {
        assert!(should_preserve(Path::new(".slopchop")));
        assert!(should_preserve(Path::new(".slopchop/stage/worktree")));
    }

    #[test]
    fn test_should_preserve_target() {
        assert!(should_preserve(Path::new("target")));
        assert!(should_preserve(Path::new("target/debug/slopchop")));
    }

    #[test]
    fn test_should_preserve_env() {
        assert!(should_preserve(Path::new(".env")));
        assert!(should_preserve(Path::new(".env.local")));
    }

    #[test]
    fn test_should_not_preserve_src() {
        assert!(!should_preserve(Path::new("src")));
        assert!(!should_preserve(Path::new("src/main.rs")));
        assert!(!should_preserve(Path::new("docs/readme.md")));
    }

    #[test]
    fn test_should_preserve_vscode() {
        assert!(should_preserve(Path::new(".vscode")));
        assert!(should_preserve(Path::new(".vscode/settings.json")));
    }
}
