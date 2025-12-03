// src/trace/runner.rs
//! Trace command runner.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::options::TraceOptions;
use super::output;
use crate::config::Config;
use crate::discovery;
use crate::graph::rank::RepoGraph;
use crate::tokens::Tokenizer;

/// Result of tracing dependencies.
pub struct TraceResult {
    pub anchor: PathBuf,
    pub direct: Vec<PathBuf>,
    pub indirect: Vec<PathBuf>,
    pub output: String,
    pub tokens: usize,
}

/// Runs the trace command.
///
/// # Errors
/// Returns error if anchor doesn't exist or file operations fail.
pub fn run(options: &TraceOptions) -> Result<TraceResult> {
    if !options.anchor.exists() {
        anyhow::bail!("Anchor file not found: {}", options.anchor.display());
    }

    println!("🔍 Tracing from {}...", options.anchor.display());

    let config = Config::new();
    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);
    let file_vec: Vec<_> = contents
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();

    let mut graph = RepoGraph::build(&file_vec);
    graph.focus_on(&options.anchor);

    let direct = graph.neighbors(&options.anchor);
    let indirect = collect_indirect(&graph, &options.anchor, &direct);

    let output_text = output::render(&options.anchor, &direct, &indirect, &contents);
    let tokens = Tokenizer::count(&output_text);

    write_output(&output_text, options)?;
    print_summary(&direct, &indirect, tokens);

    Ok(TraceResult {
        anchor: options.anchor.clone(),
        direct,
        indirect,
        output: output_text,
        tokens,
    })
}

fn read_all_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn collect_indirect(graph: &RepoGraph, anchor: &Path, direct: &[PathBuf]) -> Vec<PathBuf> {
    graph
        .ranked_files()
        .into_iter()
        .filter(|(p, _)| p != anchor && !direct.contains(p))
        .take(10)
        .map(|(p, _)| p)
        .collect()
}

fn write_output(content: &str, options: &TraceOptions) -> Result<()> {
    if options.stdout {
        println!("{content}");
    } else {
        let path = PathBuf::from("trace_context.txt");
        fs::write(&path, content).context("Failed to write trace output")?;
        println!("📦 Trace written to {}", path.display());
    }
    Ok(())
}

fn print_summary(direct: &[PathBuf], indirect: &[PathBuf], tokens: usize) {
    let total = 1 + direct.len() + indirect.len();
    println!("   {total} files, {tokens} tokens");
}

/// Quick map - shows just the file tree.
///
/// # Errors
/// Returns error if discovery fails.
pub fn quick_map(config: &Config) -> Result<String> {
    let files = discovery::discover(config)?;
    let mut output = String::from("# Repository Map\n\n");

    let dirs = group_by_directory(&files);
    for (dir, dir_files) in &dirs {
        write_dir_section(&mut output, dir, dir_files);
    }

    Ok(output)
}

fn group_by_directory(files: &[PathBuf]) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut dirs: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        dirs.entry(dir).or_default().push(file.clone());
    }
    dirs
}

fn write_dir_section(out: &mut String, dir: &Path, files: &[PathBuf]) {
    let _ = writeln!(out, "{}/ ({} files)", dir.display(), files.len());

    for f in files.iter().take(5) {
        let name = f.file_name().unwrap_or_default().to_string_lossy();
        let _ = writeln!(out, "  └── {name}");
    }

    if files.len() > 5 {
        let _ = writeln!(out, "  └── ... and {} more", files.len() - 5);
    }
}
