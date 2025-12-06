// src/roadmap_v2/cli/mod.rs
mod display;
mod handlers;
mod migrate;

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

const DEFAULT_TASKS: &str = "tasks.toml";
const DEFAULT_ROADMAP: &str = "ROADMAP.md";

#[derive(Subcommand, Debug, Clone)]
pub enum RoadmapV2Command {
    /// Initialize a new tasks.toml
    Init {
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        output: PathBuf,
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Show current roadmap status
    Show {
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        file: PathBuf,
        #[arg(long, default_value = "tree")]
        format: String,
    },
    /// List tasks with filters
    Tasks {
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        file: PathBuf,
        #[arg(long)]
        pending: bool,
        #[arg(long)]
        complete: bool,
    },
    /// Apply commands from clipboard or stdin
    Apply {
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        file: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        stdin: bool,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Generate ROADMAP.md from tasks.toml
    Generate {
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        source: PathBuf,
        #[arg(short, long, default_value = DEFAULT_ROADMAP)]
        output: PathBuf,
    },
    /// Run traceability audit
    Audit {
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        file: PathBuf,
        #[arg(long)]
        strict: bool,
    },
    /// Migrate legacy ROADMAP.md to tasks.toml
    Migrate {
        #[arg(short, long, default_value = DEFAULT_ROADMAP)]
        input: PathBuf,
        #[arg(short, long, default_value = DEFAULT_TASKS)]
        output: PathBuf,
    },
}

/// Entry point for roadmap v2 commands
///
/// # Errors
/// Returns error if IO fails or clipboard access fails
pub fn handle_command(cmd: RoadmapV2Command) -> Result<()> {
    match cmd {
        RoadmapV2Command::Init { output, name } => handlers::run_init(&output, name),
        RoadmapV2Command::Show { file, format } => handlers::run_show(&file, &format),
        RoadmapV2Command::Tasks { file, pending, complete } => {
            handlers::run_tasks(&file, pending, complete)
        }
        RoadmapV2Command::Apply { file, dry_run, stdin, verbose } => {
            handlers::run_apply(&file, dry_run, stdin, verbose)
        }
        RoadmapV2Command::Generate { source, output } => handlers::run_generate(&source, &output),
        RoadmapV2Command::Audit { file, strict } => handlers::run_audit(&file, strict),
        RoadmapV2Command::Migrate { input, output } => migrate::run_migrate(&input, &output),
    }
}