use crate::clipboard;
use crate::roadmap::{
    apply_commands, generate_prompt, CommandBatch, PromptOptions, Roadmap, TaskStatus,
};
use anyhow::{anyhow, Context, Result};
use clap::Subcommand;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Subcommand, Debug, Clone)]
pub enum RoadmapCommand {
    Init {
        #[arg(short, long, default_value = "ROADMAP.md")]
        output: PathBuf,
        #[arg(short, long)]
        name: Option<String>,
    },
    Prompt {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long)]
        full: bool,
        #[arg(long)]
        examples: bool,
        #[arg(long)]
        stdout: bool,
    },
    Apply {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        stdin: bool,
        #[arg(short, long)]
        verbose: bool,
    },
    Show {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long, default_value = "tree")]
        format: String,
    },
    Tasks {
        #[arg(short, long, default_value = "ROADMAP.md")]
        file: PathBuf,
        #[arg(long)]
        pending: bool,
        #[arg(long)]
        complete: bool,
    },
}

/// Entry point for roadmap commands
/// # Errors
/// Returns error if IO fails or clipboard access fails
pub fn handle_command(cmd: RoadmapCommand) -> Result<()> {
    match cmd {
        RoadmapCommand::Init { output, name } => run_init(&output, name),
        RoadmapCommand::Prompt {
            file,
            full,
            examples,
            stdout,
        } => run_prompt(&file, full, examples, stdout),
        RoadmapCommand::Apply {
            file,
            dry_run,
            stdin,
            verbose,
        } => run_apply(&file, dry_run, stdin, verbose),
        RoadmapCommand::Show { file, format } => run_show(&file, &format),
        RoadmapCommand::Tasks {
            file,
            pending,
            complete,
        } => run_tasks(&file, pending, complete),
    }
}

fn run_init(output: &Path, name: Option<String>) -> Result<()> {
    if output.exists() {
        return Err(anyhow!(
            "{} already exists. Use --output to specify a different file",
            output.display()
        ));
    }

    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|s| s.to_string_lossy().to_string()))
            .unwrap_or_else(|| "Project".to_string())
    });

    std::fs::write(output, generate_template(&project_name))?;
    println!("✓ Created {}", output.display());
    Ok(())
}

fn run_prompt(file: &Path, full: bool, examples: bool, stdout: bool) -> Result<()> {
    let roadmap = load_roadmap(file)?;
    let options = PromptOptions {
        full,
        examples,
        project_name: None,
    };

    let prompt = generate_prompt(&roadmap, &options);

    if stdout {
        println!("{prompt}");
    } else {
        match clipboard::smart_copy(&prompt) {
            Ok(msg) => {
                println!("✓ Copied to clipboard");
                println!("  ({msg})");
            }
            Err(e) => eprintln!("Clipboard error: {e}. Try --stdout."),
        }
    }
    Ok(())
}

fn run_apply(file: &Path, dry_run: bool, stdin: bool, verbose: bool) -> Result<()> {
    let mut roadmap = load_roadmap(file)?;

    let input = if stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    } else {
        clipboard::read_clipboard().context("Failed to read clipboard")?
    };

    let batch = CommandBatch::parse(&input);

    if batch.commands.is_empty() {
        if !batch.errors.is_empty() {
            eprintln!("Parse errors:");
            for err in &batch.errors {
                eprintln!("  {err}");
            }
        }
        return Err(anyhow!(
            "No commands found in input. Expected '===ROADMAP===' block."
        ));
    }

    println!(
        "Found {} commands: {}",
        batch.commands.len(),
        batch.summary()
    );

    if !batch.errors.is_empty() && verbose {
        eprintln!("Parse warnings:");
        for err in &batch.errors {
            eprintln!("  {err}");
        }
    }

    if dry_run {
        println!("\n[DRY RUN] Would apply:");
        for cmd in &batch.commands {
            println!("  {cmd:?}");
        }
        return Ok(());
    }

    let results = apply_commands(&mut roadmap, &batch);
    let mut success = 0;

    println!("\nResults:");
    for result in &results {
        println!("  {result}");
        if matches!(result, crate::roadmap::ApplyResult::Success(_)) {
            success += 1;
        }
    }

    if success > 0 {
        roadmap.save(file)?;
        println!("\n✓ Saved {} ({} changes applied)", file.display(), success);
    }

    Ok(())
}

fn run_show(file: &Path, format: &str) -> Result<()> {
    let roadmap = load_roadmap(file)?;
    match format {
        "stats" => {
            let stats = roadmap.stats();
            println!("{}", roadmap.title);
            println!("  Total:    {}", stats.total);
            println!("  Complete: {}", stats.complete);
            println!("  Pending:  {}", stats.pending);
            let pct = if stats.total > 0 {
                #[allow(clippy::cast_precision_loss)]
                {
                    (stats.complete as f64 / stats.total as f64) * 100.0
                }
            } else {
                0.0
            };
            println!("  Progress: {pct:.1}%");
        }
        _ => println!("{}", roadmap.compact_state()),
    }
    Ok(())
}

fn run_tasks(file: &Path, pending: bool, complete: bool) -> Result<()> {
    let roadmap = load_roadmap(file)?;
    let tasks = roadmap.all_tasks();

    let filter_pending = pending && !complete;
    let filter_complete = complete && !pending;

    for task in tasks {
        let include = match (filter_pending, filter_complete) {
            (true, _) => task.status == TaskStatus::Pending,
            (_, true) => task.status == TaskStatus::Complete,
            _ => true,
        };

        if include {
            let status = match task.status {
                TaskStatus::Complete => "[x]",
                TaskStatus::Pending => "[ ]",
            };
            println!("{} {} - {}", status, task.path, task.text);
        }
    }
    Ok(())
}

fn load_roadmap(path: &Path) -> Result<Roadmap> {
    Roadmap::from_file(path).context("Failed to load roadmap file. Run `warden roadmap init`?")
}

fn generate_template(name: &str) -> String {
    format!(
        r"# {name} Roadmap

## Current State

- [ ] Initial setup

## v0.1.0

**Theme:** Foundation

- [ ] Core feature
"
    )
}
