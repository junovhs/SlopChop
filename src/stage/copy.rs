// src/stage/copy.rs
//! Directory copy logic with exclusions for stage creation.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Directories to always exclude from stage copy.
const EXCLUDED_DIRS: &[&str] = &[
    ".slopchop",
    ".git",
    "node_modules",
    "target",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "vendor",
];

/// Files to always exclude from stage copy.
const EXCLUDED_FILES: &[&str] = &[".DS_Store", "Thumbs.db", "desktop.ini"];

/// Copies the source directory to destination, excluding heavy/ignored paths.
///
/// # Errors
/// Returns error if filesystem operations fail.
pub fn copy_repo_to_stage(src: &Path, dest: &Path) -> Result<CopyStats> {
    let mut stats = CopyStats::default();

    fs::create_dir_all(dest)
        .with_context(|| format!("Failed to create stage dir: {}", dest.display()))?;

    for entry in WalkDir::new(src).min_depth(1) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                stats.errors += 1;
                eprintln!("Warning: Failed to read entry: {e}");
                continue;
            }
        };

        let Ok(rel_path) = entry.path().strip_prefix(src) else {
            continue;
        };

        if should_exclude(rel_path) {
            update_skipped_stats(&mut stats, &entry);
            continue;
        }

        let dest_path = dest.join(rel_path);
        process_entry_copy(&entry, &dest_path, &mut stats);
    }

    Ok(stats)
}

fn update_skipped_stats(stats: &mut CopyStats, entry: &walkdir::DirEntry) {
    if entry.file_type().is_dir() {
        stats.dirs_skipped += 1;
    } else {
        stats.files_skipped += 1;
    }
}

fn process_entry_copy(entry: &walkdir::DirEntry, dest_path: &Path, stats: &mut CopyStats) {
    if entry.file_type().is_dir() {
        if let Err(e) = create_dir_safe(dest_path) {
            stats.errors += 1;
            eprintln!("Warning: Failed to create dir {}: {e}", dest_path.display());
        } else {
            stats.dirs_copied += 1;
        }
    } else if entry.file_type().is_file() {
        if let Err(e) = copy_file_safe(entry.path(), dest_path) {
            stats.errors += 1;
            eprintln!("Warning: Failed to copy {}: {e}", entry.path().display());
        } else {
            stats.files_copied += 1;
        }
    } else if entry.file_type().is_symlink() {
        stats.symlinks_skipped += 1;
    }
}

/// Statistics from a copy operation.
#[derive(Debug, Default)]
pub struct CopyStats {
    pub files_copied: usize,
    pub dirs_copied: usize,
    pub files_skipped: usize,
    pub dirs_skipped: usize,
    pub symlinks_skipped: usize,
    pub errors: usize,
}

impl CopyStats {
    /// Returns true if the copy completed without errors.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors == 0
    }

    /// Returns a human-readable summary.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Copied {} files, {} dirs. Skipped {} files, {} dirs, {} symlinks.",
            self.files_copied,
            self.dirs_copied,
            self.files_skipped,
            self.dirs_skipped,
            self.symlinks_skipped
        )
    }
}

/// Checks if a relative path should be excluded from copying.
fn should_exclude(rel_path: &Path) -> bool {
    rel_path.components().any(|c| {
        if let std::path::Component::Normal(name) = c {
            let s = name.to_string_lossy();
            EXCLUDED_DIRS.contains(&s.as_ref()) || EXCLUDED_FILES.contains(&s.as_ref())
        } else {
            false
        }
    })
}

fn create_dir_safe(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
    }
    Ok(())
}

fn copy_file_safe(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        create_dir_safe(parent)?;
    }
    fs::copy(src, dest)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dest.display()))?;
    Ok(())
}

/// Removes a directory and all its contents.
///
/// # Errors
/// Returns error if removal fails.
pub fn remove_stage(stage_dir: &Path) -> Result<()> {
    if stage_dir.exists() {
        fs::remove_dir_all(stage_dir)
            .with_context(|| format!("Failed to remove stage: {}", stage_dir.display()))?;
    }
    Ok(())
}