// src/bin/slopchop.rs
use clap::Parser;
use colored::Colorize;
use slopchop_core::cli::{self, Cli, Commands};
use slopchop_core::exit::SlopChopExit;
use std::process::{ExitCode, Termination};

fn main() -> ExitCode {
    let args = Cli::parse();

    // --ui flag override
    if args.ui {
        let mut config = slopchop_core::config::Config::load();
        if let Err(e) = slopchop_core::tui::run(&mut config) {
            eprintln!("Error running UI: {e}");
            return SlopChopExit::Error.report();
        }
        return SlopChopExit::Success.report();
    }

    let result = match args.command {
        Some(Commands::Prompt { copy }) => cli::handle_prompt(copy),
        Some(Commands::Scan { verbose }) => cli::handle_scan(verbose),
        Some(Commands::Check) => cli::handle_check(),
        Some(Commands::Fix) => cli::handle_fix(),
        Some(Commands::Apply {
            force,
            dry_run,
            stdin,
            check,
            file,
            reset,
            promote,
        }) => cli::handle_apply(&slopchop_core::cli::args::ApplyArgs {
            force,
            dry_run,
            stdin,
            check,
            file,
            reset,
            promote,
        }),
        Some(Commands::Clean { commit }) => {
            slopchop_core::clean::run(commit).map(|()| SlopChopExit::Success)
        }
        Some(Commands::Config) => {
            slopchop_core::tui::run_config().map(|()| SlopChopExit::Success)
        }
        Some(Commands::Dashboard) => cli::handle_dashboard(),
        Some(Commands::Audit {
            format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        }) => slopchop_core::cli::handle_audit(&slopchop_core::cli::audit::AuditCliOptions {
            format: &format,
            no_dead,
            no_dups,
            no_patterns,
            min_lines,
            max,
            verbose,
        }).map(|()| SlopChopExit::Success),
        Some(Commands::Pack {
            stdout,
            copy,
            noprompt,
            format,
            skeleton,
            code_only,
            verbose,
            target,
            focus,
            depth,
        }) => cli::handle_pack(slopchop_core::cli::PackArgs {
            stdout,
            copy,
            noprompt,
            format,
            skeleton,
            code_only,
            verbose,
            target,
            focus,
            depth,
        }),
        Some(Commands::Trace {
            file,
            depth,
            budget,
        }) => cli::handle_trace(&file, depth, budget),
        Some(Commands::Map { deps }) => cli::handle_map(deps),
        Some(Commands::Signatures { copy, stdout }) => {
            cli::handle_signatures(slopchop_core::signatures::SignatureOptions { copy, stdout })
        }
        None => {
            // Default to UI if no command provided
            let mut config = slopchop_core::config::Config::load();
            slopchop_core::tui::run(&mut config).map(|()| SlopChopExit::Success)
        }
    };

    match result {
        Ok(exit_code) => exit_code.report(),
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            SlopChopExit::Error.report()
        }
    }
}