//! Command execution and output capture.

use std::fmt::Write;
use std::path::Path;
use std::process::Command;

use super::VerificationReport;

/// Runs a list of commands and captures combined output.
#[must_use]
pub fn run_commands(repo_root: &Path, commands: &[String]) -> VerificationReport {
    let mut passed = true;
    let mut output = String::new();

    for cmd_str in commands {
        let _ = writeln!(output, "$ {cmd_str}");

        match run_single_command(repo_root, cmd_str) {
            Ok(cmd_output) => {
                output.push_str(&cmd_output);
                output.push('\n');
            }
            Err(e) => {
                passed = false;
                let _ = writeln!(output, "ERROR: {e}");
                output.push('\n');
            }
        }
    }

    VerificationReport::new(passed, output)
}

/// Runs a single command string.
fn run_single_command(repo_root: &Path, cmd_str: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();

    let Some(&program) = parts.first() else {
        return Ok("(empty command)".to_string());
    };

    let args = parts.get(1..).unwrap_or(&[]);

    let output = Command::new(program)
        .args(args)
        .current_dir(repo_root)
        .output()?;

    let mut result = String::new();

    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        let _ = writeln!(
            result,
            "[exit code: {}]",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(result)
}
