// src/clipboard/linux.rs
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return true;
    }
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        let v = version.to_lowercase();
        return v.contains("microsoft") || v.contains("wsl");
    }
    false
}

/// Copies the file at the given path to the clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn copy_file_handle(path: &Path) -> Result<()> {
    if is_wsl() {
        return copy_file_handle_wsl(path);
    }
    copy_file_handle_native(path)
}

fn copy_file_handle_native(path: &Path) -> Result<()> {
    let uri = format!("file://{}", path.to_string_lossy());

    if try_pipe_to_cmd("wl-copy", &["--type", "text/uri-list"], &uri).is_ok() {
        return Ok(());
    }
    try_pipe_to_cmd(
        "xclip",
        &["-selection", "clipboard", "-t", "text/uri-list", "-i"],
        &uri,
    )
}

fn copy_file_handle_wsl(path: &Path) -> Result<()> {
    let output = Command::new("wslpath")
        .arg("-w")
        .arg(path)
        .output()
        .context("Failed to run wslpath")?;

    let win_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if win_path.is_empty() {
        return Err(anyhow::anyhow!("wslpath returned empty string"));
    }

    let escaped = win_path.replace('\'', "''");
    let cmd = format!("Set-Clipboard -Path '{escaped}'");

    Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &cmd])
        .output()
        .context("Failed to set clipboard via powershell.exe in WSL")?;
    Ok(())
}

/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    if is_wsl() {
        return perform_copy_wsl(text);
    }
    perform_copy_native(text)
}

fn perform_copy_wsl(text: &str) -> Result<()> {
    if try_pipe_to_cmd("clip.exe", &[], text).is_ok() {
        return Ok(());
    }
    try_pipe_to_cmd(
        "powershell.exe",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$Input | Set-Clipboard",
        ],
        text,
    )
}

fn perform_copy_native(text: &str) -> Result<()> {
    if try_pipe_to_cmd("xclip", &["-selection", "clipboard", "-in"], text).is_ok() {
        return Ok(());
    }
    try_pipe_to_cmd("wl-copy", &[], text)
}

fn try_pipe_to_cmd(cmd: &str, args: &[&str], input: &str) -> Result<()> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .context(format!("Failed to spawn {cmd}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .context(format!("Failed to write to {cmd}"))?;
    }
    child.wait().context(format!("Failed to wait for {cmd}"))?;
    Ok(())
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_read() -> Result<String> {
    if is_wsl() {
        return perform_read_wsl();
    }
    perform_read_native()
}

fn perform_read_wsl() -> Result<String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard"])
        .output()
        .context("Failed to run Get-Clipboard via powershell.exe")?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

fn perform_read_native() -> Result<String> {
    if let Ok(output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-out"])
        .output()
    {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let output = Command::new("wl-paste").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}