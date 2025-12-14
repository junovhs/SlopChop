// src/clipboard/utils.rs
use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

/// Pipes input text to a command's stdin.
///
/// # Errors
/// Returns error if spawning, writing, or waiting fails.
pub fn pipe_to_cmd(cmd: &str, args: &[&str], input: &str) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to spawn {cmd}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .with_context(|| format!("Failed to write to {cmd}"))?;
    }
    child
        .wait()
        .with_context(|| format!("Failed to wait for {cmd}"))?;
    Ok(())
}