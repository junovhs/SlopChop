// src/cli/stage_handlers.rs
//! Handlers for stage-related CLI commands.

use crate::exit::SlopChopExit;
use crate::stage;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Handles the stage command.
///
/// # Errors
/// Returns error if stage operations fail.
pub fn handle_stage(force: bool) -> Result<SlopChopExit> {
    let repo_root = super::handlers::get_repo_root();
    let mut stage_mgr = stage::StageManager::new(&repo_root);

    if stage_mgr.exists() && !force {
        println!("{}", "Stage already exists.".yellow());
        if !crate::apply::executor::confirm("Wipe and recreate?")? {
            return Ok(SlopChopExit::Success);
        }
    }

    if stage_mgr.exists() {
        stage_mgr.reset()?;
    }

    let result = stage_mgr.create_stage()?;
    if let stage::EnsureResult::Created(stats) = result {
        println!("{}", "Stage created.".green().bold());
        println!("  {}", stats.summary());
    }

    Ok(SlopChopExit::Success)
}

/// Handles the sync operation (nuclear mirror from stage to workspace).
///
/// # Errors
/// Returns error if sync operations fail.
pub fn handle_sync(repo_root: &Path) -> Result<SlopChopExit> {
    let mut stage_mgr = stage::StageManager::new(repo_root);

    if !stage_mgr.exists() {
        println!("{}", "No stage to sync.".yellow());
        return Ok(SlopChopExit::Error);
    }

    let worktree = stage_mgr.worktree();
    let result = stage::mirror_stage_to_workspace(repo_root, &worktree)?;

    println!("{}", "[SYNC] Stage mirrored to workspace.".green().bold());
    println!("  Written: {}", result.files_written);
    println!("  Deleted: {}", result.files_deleted);
    println!("  Preserved: {}", result.files_preserved);

    // Auto-reset stage after successful sync
    stage_mgr.reset()?;
    println!("  Stage cleared.");

    Ok(SlopChopExit::Success)
}
