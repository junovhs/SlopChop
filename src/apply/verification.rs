// src/apply/verification.rs
use crate::analysis::RuleEngine;
use crate::apply::types::ApplyContext;
use crate::clipboard;
use crate::config::Config;
use crate::discovery;
use crate::events::{EventKind, EventLogger};
use crate::reporting;
use crate::spinner::Spinner;
use crate::stage;
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::env;
use std::path::Path;
use std::process::Command;

/// Runs the full verification pipeline: Check -> Test -> Scan.
/// Stops at the first failure, summarizes output, and copies to clipboard.
///
/// Uses the staged worktree as cwd if provided, otherwise uses `effective_cwd`.
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_pipeline<P: AsRef<Path>>(ctx: &ApplyContext, cwd: P) -> Result<bool> {
    let logger = EventLogger::new(&ctx.repo_root);
    logger.log(EventKind::CheckStarted);

    println!("{}", "\n→ Verifying changes...".blue().bold());

    let working_dir = cwd.as_ref();

    // 1. Run external checks (e.g. clippy, eslint)
    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            if !run_stage_in_dir(cmd, cmd, working_dir)? {
                logger.log(EventKind::CheckFailed { exit_code: 1 });
                return Ok(false);
            }
        }
    }

    // 2. Run SlopChop scan (Structural check) - Internal Call
    // We run this in-process to avoid binary mismatch/recursion/PATH issues with subprocesses.
    if !run_internal_scan(working_dir)? {
        logger.log(EventKind::CheckFailed { exit_code: 1 });
        return Ok(false);
    }

    logger.log(EventKind::CheckPassed);
    Ok(true)
}

/// Runs verification using the effective cwd (stage if exists, else repo root).
///
/// # Errors
/// Returns error if command execution fails.
pub fn run_verification_auto(ctx: &ApplyContext) -> Result<bool> {
    let cwd = stage::effective_cwd(&ctx.repo_root);
    run_verification_pipeline(ctx, &cwd)
}

fn run_stage_in_dir(label: &str, cmd_str: &str, cwd: &Path) -> Result<bool> {
    let sp = Spinner::start(label);

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        sp.stop(true);
        return Ok(true);
    };

    let output = Command::new(prog).args(args).current_dir(cwd).output()?;

    let success = output.status.success();
    sp.stop(success);

    if !success {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");

        let summary = summarize_output(&combined, cmd_str);

        handle_failure(label, &summary);
    }

    Ok(success)
}

fn run_internal_scan(cwd: &Path) -> Result<bool> {
    let sp = Spinner::start("slopchop scan");
    
    // RAII-style CWD guard
    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;
    
    // Run scan logic
    let scan_result = std::panic::catch_unwind(|| -> Result<bool> {
        // Reload config from the target directory to respect local settings
        let config = Config::load();
        let files = discovery::discover(&config)?;
        let engine = RuleEngine::new(config);
        let report = engine.scan(files);
        
        if report.has_errors() {
            Ok(false)
        } else {
            Ok(true)
        }
    });

    // Restore CWD immediately
    env::set_current_dir(original_cwd)?;

    match scan_result {
        Ok(Ok(success)) => {
            sp.stop(success);
            if !success {
                println!("{}", "-".repeat(60).red());
                println!("{} Failed: {}", "[!]".red(), "slopchop scan".bold());
                
                // Re-run to print output to stdout
                env::set_current_dir(cwd)?;
                let config = Config::load();
                let files = discovery::discover(&config)?;
                let engine = RuleEngine::new(config);
                let report = engine.scan(files);
                reporting::print_report(&report)?;
                env::set_current_dir(env::current_dir()?.parent().unwrap_or(Path::new(".")))?; 
                
                println!("{}", "-".repeat(60).red());
                return Ok(false);
            }
            Ok(true)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err(anyhow!("Internal scan panicked")),
    }
}

fn handle_failure(stage_name: &str, summary: &str) {
    println!("{}", "-".repeat(60).red());
    println!("{} Failed: {}", "[!]".red(), stage_name.bold());
    println!("{}", summary.trim());
    println!("{}", "-".repeat(60).red());

    match clipboard::smart_copy(summary) {
        Ok(msg) => println!("{} {}", "[+]".yellow(), msg),
        Err(e) => println!("{} Failed to copy to clipboard: {}", "[!]".yellow(), e),
    }
}

fn summarize_output(output: &str, cmd: &str) -> String {
    let is_test = cmd.contains("test");
    let is_cargo = cmd.contains("cargo");

    output
        .lines()
        .filter(|line| keep_line(line, is_cargo, is_test))
        .take(50) // Limit length for token efficiency
        .collect::<Vec<_>>()
        .join("\n")
}

fn keep_line(line: &str, is_cargo: bool, is_test: bool) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    if is_common_noise(trimmed) {
        return false;
    }

    if is_test && is_test_noise(trimmed) {
        return false;
    }

    if is_cargo && is_cargo_noise(trimmed) {
        return false;
    }

    true
}

fn is_common_noise(line: &str) -> bool {
    line.starts_with("Finished")
        || line.starts_with("Compiling")
        || line.starts_with("Running")
        || line.starts_with("Doc-tests")
        || line.starts_with("Checking")
}

fn is_test_noise(line: &str) -> bool {
    line.starts_with("test result:") || line.starts_with("test ")
}

fn is_cargo_noise(line: &str) -> bool {
    line.contains("warnings emitted") || line.contains("generated")
}