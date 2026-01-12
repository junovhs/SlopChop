pub mod advisory;
pub mod backup;
mod blocks;
pub mod executor;
pub mod manifest;
pub mod messages;
pub mod parser;
pub mod process_runner;
pub mod processor;
pub mod types;
pub mod validator;
pub mod verification;
pub mod writer;

use crate::clipboard;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Read};
use types::{ApplyContext, ApplyInput, ApplyOutcome};

/// Executes the apply operation based on user input.
///
/// # Errors
/// Returns error if input reading or processing fails.
pub fn run_apply(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    if ctx.reset_stage {
        println!("{}", "Stage reset is deprecated. Use git branches instead.".yellow());
        return Ok(ApplyOutcome::StageReset);
    }
    let content = read_input(&ctx.input)?;
    processor::process_input(&content, ctx)
}

fn read_input(input: &ApplyInput) -> Result<String> {
    match input {
        ApplyInput::Clipboard => clipboard::read_clipboard().context("Failed to read clipboard"),
        ApplyInput::Stdin => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).context("Failed to read stdin")?;
            Ok(buf)
        }
        ApplyInput::File(path) => std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display())),
    }
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

/// Validates and applies a string payload containing a plan, manifest and files.
///
/// # Errors
/// Returns error if extraction, confirmation or writing fails.
pub fn process_input(content: &str, ctx: &ApplyContext) -> Result<ApplyOutcome> {
    processor::process_input(content, ctx)
}

/// Promotes staged changes to the real workspace.
///
/// # Errors
/// Returns error if promotion fails.
pub fn run_promote(ctx: &ApplyContext) -> Result<ApplyOutcome> {
    processor::run_promote_standalone(ctx)
}