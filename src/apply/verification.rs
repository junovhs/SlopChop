// src/apply/verification.rs
use crate::apply::process_runner::CommandRunner;
use crate::apply::types::ApplyContext;
use crate::cli::locality;
use crate::config::Config;
use crate::discovery;
use crate::analysis::RuleEngine;
use crate::events::{EventKind, EventLogger};
use crate::reporting;
use crate::spinner::Spinner;
use crate::types::{CheckReport, CommandResult, ScanReport};
use anyhow::Result;
use colored::Colorize;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Runs the full verification pipeline: External Commands -> Scan -> Locality.
pub fn run_verification_pipeline<P: AsRef<Path>>(
    ctx: &ApplyContext,
    cwd: P,
) -> Result<CheckReport> {
    let logger = EventLogger::new(&ctx.repo_root);
    logger.log(EventKind::CheckStarted);

    if !ctx.silent {
        println!("{}", "\n  Verifying changes...".blue().bold());
    }

    let working_dir = cwd.as_ref();
    let runner = CommandRunner::new(ctx.silent);

    let (mut command_results, mut passed) = run_external_checks(ctx, working_dir, &runner, &logger)?;

    let scan_report = run_internal_scan(working_dir, ctx.silent)?;
    if scan_report.has_errors() {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }

    let locality_result = run_locality_scan(working_dir, ctx.silent)?;
    if locality_result.exit_code != 0 {
        passed = false;
        logger.log(EventKind::CheckFailed { exit_code: 1 });
    }
    command_results.push(locality_result);

    if !ctx.silent {
        crate::apply::advisory::maybe_print_edit_advisory(&ctx.repo_root);
    }

    if passed {
        logger.log(EventKind::CheckPassed);
    }

    // Use extracted writer to save tokens in this file
    crate::apply::report_writer::write_check_report(&scan_report, &command_results, passed, &ctx.repo_root)?;

    Ok(CheckReport {
        scan: scan_report,
        commands: command_results,
        passed,
    })
}

fn run_external_checks(
    ctx: &ApplyContext,
    cwd: &Path,
    runner: &CommandRunner,
    logger: &EventLogger
) -> Result<(Vec<CommandResult>, bool)> {
    let mut results = Vec::new();
    let mut passed = true;

    if let Some(commands) = ctx.config.commands.get("check") {
        for cmd in commands {
            let result = runner.run(cmd, cwd)?;
            if result.exit_code != 0 {
                passed = false;
                logger.log(EventKind::CheckFailed { exit_code: result.exit_code });
            }
            results.push(result);
        }
    }
    Ok((results, passed))
}

fn run_internal_scan(cwd: &Path, silent: bool) -> Result<ScanReport> {
    let sp = if silent { None } else { Some(Spinner::start("slopchop scan")) };
    let start = Instant::now();

    let original_cwd = env::current_dir()?;
    env::set_current_dir(cwd)?;

    let config = Config::load();
    let files = discovery::discover(&config)?;
    let engine = RuleEngine::new(config);
    let total_files = files.len();

    let counter = AtomicUsize::new(0);
    
    // NO THROTTLING: Update every file for maximum "stream" feel.
    // Spinner render loop (80ms) handles visual throttling.
    let report = engine.scan_with_progress(&files, &|path| {
        if let Some(s) = &sp {
            let i = counter.fetch_add(1, Ordering::Relaxed) + 1;
            s.step_progress(i, total_files, format!("Scanning {}", path.display()));
        }
    });
    
    let mut final_report = report;
    final_report.duration_ms = start.elapsed().as_millis();

    env::set_current_dir(original_cwd)?;

    let success = !final_report.has_errors();
    if let Some(s) = sp { s.stop(success); }

    if !success && !silent {
        reporting::print_report(&final_report)?;
    }

    Ok(final_report)
}

fn run_locality_scan(cwd: &Path, silent: bool) -> Result<CommandResult> {
    let config = Config::load();

    if !config.rules.locality.is_enabled() || !config.rules.locality.is_error_mode() {
        return Ok(CommandResult {
            command: "slopchop scan --locality".to_string(),
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration_ms: 0,
        });
    }

    let start = Instant::now();
    let sp = if silent { None } else { Some(Spinner::start("slopchop scan --locality")) };

    let passed: bool;
    let output: String;

    if silent {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(cwd)?;
        let (p, v) = locality::check_locality_silent(cwd)?;
        env::set_current_dir(original_cwd)?;
        passed = p;
        if passed {
            output = String::new();
        } else {
            output = format!("Locality check failed with {v} violations.");
        }
    } else {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(cwd)?;
        let res = locality::run_locality_check(cwd)?;
        env::set_current_dir(original_cwd)?;
        passed = res.passed;
        if passed {
            output = String::new();
        } else {
            output = format!("Locality check failed with {} violations.", res.violations);
        }
    }

    let duration = start.elapsed();
    if let Some(s) = sp { s.stop(passed); }

    #[allow(clippy::cast_possible_truncation)]
    Ok(CommandResult {
        command: "slopchop scan --locality".to_string(),
        exit_code: i32::from(!passed),
        stdout: output,
        stderr: String::new(),
        duration_ms: duration.as_millis() as u64,
    })
}