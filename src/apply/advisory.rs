// src/apply/advisory.rs
use std::path::Path;
use colored::Colorize;

/// Threshold for triggering the high edit volume advisory.
const NAG_THRESHOLD: usize = 3;

/// Prints an advisory if many files have been modified.
/// With stage system removed, this now checks git status.
pub fn maybe_print_edit_advisory(repo_root: &Path) {
    let modified_count = count_git_changes(repo_root);

    if modified_count > NAG_THRESHOLD {
        println!();
        println!("{}", "━".repeat(60).yellow());
        println!("{}", "[ADVISORY] High Edit Volume Detected".yellow().bold());
        println!("  {modified_count} files modified.");
        println!("  Consider committing soon to maintain high-integrity checkpoints.");
        println!("  Run: {} to commit changes.", "git commit".cyan());
        println!("{}", "━".repeat(60).yellow());
    }
}

/// Counts modified files using git status.
fn count_git_changes(repo_root: &Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_root)
        .output();

    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).lines().count(),
        Err(_) => 0,
    }
}
