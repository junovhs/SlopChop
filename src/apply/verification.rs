// src/apply/verification.rs
use crate::analysis::RuleEngine;
use crate::apply::types::ApplyContext;
use crate::cli::locality;
use crate::clipboard;
use crate::config::Config;
use crate::discovery;
use crate::events::{EventKind, EventLogger};
use crate::reporting;
use crate::spinner::Spinner;
use crate::stage;
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::path::Path;
use std::process::Command;

pub struct VerificationResult {
    pub passed: bool,
    pub failed_checks: Vec<(String, String)>,
}

/// Runs the full verification pipeline: External Commands -> Scan -> Locality.
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_pipeline<P: AsRef<Path>>(ctx: &ApplyContext, cwd: P) -> Result<VerificationResult> {
    let logger = EventLogger::new(&ctx.repo_root);
    logger.log(EventKind::CheckStarted);

    println!("{}", "\n  Verifying changes...".blue().bold());

    let working_dir = cwd.as_ref();
    let mut failed_checks = Vec::new();

    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            if let Some(error) = run_stage_in_dir(cmd, cmd, working_dir)? {
                logger.log(EventKind::CheckFailed { exit_code: 1 });
                failed_checks.push((cmd.clone(), error));
                return Ok(VerificationResult { passed: false, failed_checks });
            }
        }
    }

    if let Some(error) = run_internal_scan(working_dir)? {
        logger.log(EventKind::CheckFailed { exit_code: 1 });
        failed_checks.push(("slopchop scan".to_string(), error));
        return Ok(VerificationResult { passed: false, failed_checks });
    }

    if let Some(error) = run_locality_scan(working_dir)? {
        logger.log(EventKind::CheckFailed { exit_code: 1 });
        failed_checks.push(("slopchop scan --locality".to_string(), error));
        return Ok(VerificationResult { passed: false, failed_checks });
    }

    logger.log(EventKind::CheckPassed);
    Ok(VerificationResult { passed: true, failed_checks })
}

#[must_use]
pub fn generate_ai_feedback(
    failed_commands: &[(String, String)],
    modified_files: &[String],
) -> String {
    use std::fmt::Write;
    let mut msg = String::from("VERIFICATION FAILED\n\n");
    
    msg.push_str("The following checks failed:\n\n");
    for (cmd, stderr) in failed_commands {
        let _ = writeln!(msg, "COMMAND: {cmd}");
        msg.push_str("OUTPUT:\n");
        // Truncate to ~1000 chars to avoid clipboard bloat
        let truncated = if stderr.len() > 1000 {
            format!("{}...\n[truncated]", &stderr[..1000])
        } else {
            stderr.clone()
        };
        msg.push_str(&truncated);
        msg.push_str("\n\n");
    }
    
    if !modified_files.is_empty() {
        msg.push_str("FILES MODIFIED IN THIS APPLY:\n");
        for f in modified_files {
            let _ = writeln!(msg, "- {f}");
        }
    }
    
    msg.push_str("\nPlease fix the issues and provide corrected files.");
    msg
}

/// Runs verification using the effective cwd (stage if exists, else repo root).
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_auto(ctx: &ApplyContext) -> Result<bool> {
    let cwd = stage::effective_cwd(&ctx.repo_root);
    let result = run_verification_pipeline(ctx, &cwd)?;
    Ok(result.passed)
}

fn run_stage_in_dir(label: &str, cmd_str: &str, cwd: &Path) -> Result<Option<String>> {
    let sp = Spinner::start(label);

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        sp.stop(true);
        return Ok(None);
    };

    let output = Command::new(prog).args(args).current_dir(cwd).output()?;
    let success = output.status.success();
    sp.stop(success);

    if !success {
        let combined = collect_output(&output.stdout, &output.stderr);
        let summary = summarize_output(&combined, cmd_str);
        handle_failure(label, &summary);
        return Ok(Some(combined));
    }

    Ok(None)
}

fn collect_output(stdout: &[u8], stderr: &[u8]) -> String {
    let out = String::from_utf8_lossy(stdout);
    let err = String::from_utf8_lossy(stderr);
    format!("{out}\n{err}")
}

fn run_internal_scan(cwd: &Path) -> Result<Option<String>> {
    let sp = Spinner::start("slopchop scan");
    
    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;
    
    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);
    
    env::set_current_dir(original_cwd)?;
    
    let success = !report.has_errors();
    sp.stop(success);

    if !success {
        // We capture output by forcing the report to a string is tricky because print_report writes to stdout
        // But for internal scan we can just say "Scan violations found"
        reporting::print_report(&report)?;
        return Ok(Some("SlopChop internal scan failed. See output above.".to_string()));
    }

    Ok(None)
}

fn run_locality_scan(cwd: &Path) -> Result<Option<String>> {
    let config = Config::load();
    
    if !config.rules.locality.is_enabled() || !config.rules.locality.is_error_mode() {
        return Ok(None);
    }
    
    let sp = Spinner::start("slopchop scan --locality");

    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;

    let result = locality::run_locality_check(cwd);

    env::set_current_dir(original_cwd)?;

    let success = result.as_ref().is_ok_and(|r| r.passed);
    sp.stop(success);

    if let Ok(ref res) = result {
        if !res.passed {
            println!("{} locality violations found", res.violations);
            return Ok(Some(format!("Locality check failed: {} violations", res.violations)));
        }
    }

    Ok(None)
}

fn summarize_output(output: &str, cmd: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let max_lines = 20;
    
    if lines.len() <= max_lines {
        return output.to_string();
    }
    
    let summary: String = lines.iter().take(max_lines).copied().collect::<Vec<_>>().join("\n");
    format!("{summary}\n... ({} more lines, run '{cmd}' for full output)", lines.len() - max_lines)
}

fn handle_failure(label: &str, summary: &str) {
    println!("{}", "-".repeat(60));
    println!("{} {label}", "[!] Failed:".red().bold());
    println!("{summary}");
    println!("{}", "-".repeat(60));

    if let Err(e) = clipboard::copy_to_clipboard(summary) {
        eprintln!("Could not copy to clipboard: {e}");
    } else {
        println!("{}", "[+] Text copied to clipboard".dimmed());
    }
}
