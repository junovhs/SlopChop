use clap::Parser;
use colored::Colorize;
use slopchop_core::cli::{self, Cli};
use slopchop_core::exit::SlopChopExit;

fn main() -> SlopChopExit {
    let cli = Cli::parse();

    let result = if let Some(cmd) = cli.command {
        cli::dispatch::execute(cmd)
    } else {
        use clap::CommandFactory;
        let _ = Cli::command().print_help();
        Ok(SlopChopExit::Success)
    };

    match result {
        Ok(exit_code) => exit_code,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            SlopChopExit::Error
        }
    }
}