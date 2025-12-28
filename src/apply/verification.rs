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

/// Runs the full verification pipeline: External Commands -> Scan -> Locality.
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

    // 2. Run SlopChop structural scan
    if !run_internal_scan(working_dir)? {
        logger.log(EventKind::CheckFailed { exit_code: 1 });
        return Ok(false);
    }

    // 3. Run locality scan (if enabled)
    if !run_locality_scan(working_dir)? {
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
        let combined = collect_output(&output.stdout, &output.stderr);
        let summary = summarize_output(&combined, cmd_str);
        handle_failure(label, &summary);
    }

    Ok(success)
}

fn collect_output(stdout: &[u8], stderr: &[u8]) -> String {
    let out = String::from_utf8_lossy(stdout);
    let err = String::from_utf8_lossy(stderr);
    format!("{out}\n{err}")
}

fn run_internal_scan(cwd: &Path) -> Result<bool> {
    let sp = Spinner::start("slopchop scan");
    
    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;
    
    let scan_result = execute_scan();
    
    env::set_current_dir(&original_cwd)?;

    match scan_result {
        Ok(true) => { sp.stop(true); Ok(true) }
        Ok(false) => {
            sp.stop(false);
            print_scan_failure(cwd, &original_cwd)?;
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

fn execute_scan() -> Result<bool> {
    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);
    Ok(!report.has_errors())
}

fn print_scan_failure(cwd: &Path, original: &Path) -> Result<()> {
    println!("{}", "-".repeat(60).red());
    println!("{} Failed: {}", "[!]".red(), "slopchop scan".bold());
    
    env::set_current_dir(cwd)?;
    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);
    reporting::print_report(&report)?;
    env::set_current_dir(original)?;
    
    println!("{}", "-".repeat(60).red());
    Ok(())
}

fn run_locality_scan(cwd: &Path) -> Result<bool> {
    let config = Config::load();
    
    if !config.rules.locality.is_enabled() {
        return Ok(true);
    }

    let is_blocking = config.rules.locality.is_error_mode();
    let label = if is_blocking { "slopchop scan --locality" } else { "slopchop scan --locality (warn)" };
    
    let sp = Spinner::start(label);
    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;
    
    let result = locality::check_locality_silent(cwd);
    
    env::set_current_dir(&original_cwd)?;

    handle_locality_result(result, sp, cwd, &original_cwd)
}

fn handle_locality_result(
    result: Result<(bool, usize)>,
    sp: Spinner,
    cwd: &Path,
    original: &Path,
) -> Result<bool> {
    let (passed, violations) = result?;
    
    if violations == 0 {
        sp.stop(true);
        return Ok(true);
    }
    
    if passed {
        sp.stop(true);
        print_locality_warnings(cwd, original, violations)?;
    } else {
        sp.stop(false);
        print_locality_failure(cwd, original)?;
    }
    
    Ok(passed)
}

fn print_locality_warnings(cwd: &Path, original: &Path, count: usize) -> Result<()> {
    println!("{}", "-".repeat(60).yellow());
    println!("{} {} locality violation(s) (non-blocking)", "[!]".yellow(), count);
    
    env::set_current_dir(cwd)?;
    let _ = locality::run_locality_check(cwd);
    env::set_current_dir(original)?;
    
    println!("{}", "-".repeat(60).yellow());
    Ok(())
}

fn print_locality_failure(cwd: &Path, original: &Path) -> Result<()> {
    println!("{}", "-".repeat(60).red());
    println!("{} Failed: {}", "[!]".red(), "slopchop scan --locality".bold());
    
    env::set_current_dir(cwd)?;
    let _ = locality::run_locality_check(cwd);
    env::set_current_dir(original)?;
    
    println!("{}", "-".repeat(60).red());
    Ok(())
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
    let dominated_by_cargo = cmd.contains("cargo");
    let dominated_by_test = cmd.contains("test");

    output
        .lines()
        .filter(|line| is_relevant_line(line, dominated_by_cargo, dominated_by_test))
        .take(50)
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_relevant_line(line: &str, is_cargo: bool, is_test: bool) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() { return false; }
    if is_cargo && is_cargo_noise(trimmed) { return false; }
    if is_test && trimmed.starts_with("running ") { return false; }
    true
}

fn is_cargo_noise(line: &str) -> bool {
    line.contains("Compiling") || line.contains("Finished") || 
    line.contains("Fresh") || line.contains("Downloading")
}