pub mod backup;
pub mod extractor;
// git module removed per "Lean & Mean" directive
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod verification;
pub mod writer;

use crate::clipboard;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Read, Write};
use types::{ApplyContext, ApplyInput, ApplyOutcome};

/// Executes the apply operation based on user input.
///
/// # Errors
/// Returns error if input reading or processing fails.
pub fn run_apply(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let content = read_input(&ctx.input)?;
    process_input(&content, ctx)
}

fn read_input(input: &ApplyInput) -> Result<String> {
    match input {
        ApplyInput::Clipboard => clipboard::read_clipboard().context("Failed to read clipboard"),
        ApplyInput::Stdin => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).context("Failed to read stdin")?;
            Ok(buf)
        }
        ApplyInput::File(path) => std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display())),
    }
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

/// Validates and applies a string payload containing atechnical plan, manifest and files.
///
/// # Errors
/// Returns error if extraction, confirmation or writing fails.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Input is empty".to_string()));
    }

    let plan_opt = extractor::extract_plan(content);
    if !check_plan_requirement(plan_opt.as_deref(), ctx)? {
        return Ok(ApplyOutcome::ParseError("Operation cancelled.".to_string()));
    }

    let validation = validate_payload(content);
    if !matches!(validation, ApplyOutcome::Success { .. }) {
        return Ok(validation);
    }

    apply_to_filesystem(content, ctx)
}

fn check_plan_requirement(plan: Option<&str>, ctx: &ApplyContext) -> Result<bool> {
    if let Some(p) = plan {
        println!("{}", "[PLAN]:".cyan().bold());
        println!("{}", p.trim());
        if !ctx.force && !ctx.dry_run { return confirm("Apply these changes?"); }
        return Ok(true);
    }
    if ctx.config.preferences.require_plan {
        println!("{}", "[X] REJECTED: Missing PLAN block.".red());
        Ok(false)
    } else {
        if !ctx.force && !ctx.dry_run { return confirm("Apply without a plan?"); }
        Ok(true)
    }
}

fn validate_payload(content: &str) -> ApplyOutcome {
    let manifest = match manifest::parse_manifest(content) {
        Ok(Some(m)) => m,
        Ok(None) => Vec::new(),
        Err(e) => return ApplyOutcome::ParseError(format!("Manifest Error: {e}")),
    };
    let extracted = match extractor::extract_files(content) {
        Ok(e) => e,
        Err(e) => return ApplyOutcome::ParseError(format!("Extraction Error: {e}")),
    };
    validator::validate(&manifest, &extracted)
}

fn apply_to_filesystem(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    let extracted = extractor::extract_files(content)?;
    let manifest = manifest::parse_manifest(content)?.unwrap_or_default();

    if ctx.dry_run {
        return Ok(ApplyOutcome::Success {
            written: vec!["(Dry Run) Verified".to_string()],
            deleted: vec![],
            backed_up: false,
        });
    }

    let retention = ctx.config.preferences.backup_retention;
    writer::write_files(&manifest, &extracted, None, retention)
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}