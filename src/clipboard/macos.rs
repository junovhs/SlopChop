// src/clipboard/macos.rs
use crate::clipboard::utils;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Copies the file at the given path to the clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn copy_file_handle(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    let script = format!("set the clipboard to POSIX file \"{path_str}\"");

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("Failed to set clipboard via osascript")?;
    Ok(())
}

/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    utils::pipe_to_cmd("pbcopy", &[], text)
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_read() -> Result<String> {
    let output = Command::new("pbpaste").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}