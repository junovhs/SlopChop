// src/trace/runner.rs
//! Trace command runner.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use colored::Colorize;

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

struct FileStats {
    size_kb: f64,
    tokens: usize,
}

/// Runs the trace command.
///
/// # Errors
/// Returns error if anchor doesn't exist or file operations fail.
pub fn run(opts: &TraceOptions) -> Result<String> {
    if !opts.anchor.exists() {
        anyhow::bail!("Anchor file not found: {}", opts.anchor.display());
    }

    let config = load_config();
    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);

    let file_vec: Vec<_> = contents
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();

    let mut graph = RepoGraph::build(&file_vec);
    graph.focus_on(&opts.anchor);

    let direct = graph.neighbors(&opts.anchor);
    let indirect = collect_indirect(&graph, &opts.anchor, &direct);

    Ok(output::render(&opts.anchor, &direct, &indirect, &contents))
}

/// Shows repository structure map.
///
/// # Errors
/// Returns error if discovery fails.
pub fn map(show_deps: bool) -> Result<String> {
    let config = load_config();
    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);

    let mut graph = None;
    if show_deps {
        let file_vec: Vec<_> = contents
            .iter()
            .map(|(p, c)| (p.clone(), c.clone()))
            .collect();
        graph = Some(RepoGraph::build(&file_vec));
    }

    let mut out = String::from("# Repository Map\n\n");
    let mut dirs = group_by_directory(&files);

    // Sort files within each directory for deterministic output
    for files in dirs.values_mut() {
        files.sort();
    }

    for (dir, dir_files) in &dirs {
        write_dir_section(&mut out, dir, dir_files, &contents, graph.as_ref());
    }

    Ok(out)
}

fn load_config() -> Config {
    let mut config = Config::new();
    config.load_local_config();
    config
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

fn group_by_directory(files: &[PathBuf]) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut dirs: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        dirs.entry(dir).or_default().push(file.clone());
    }
    dirs
}

fn write_dir_section(
    out: &mut String,
    dir: &Path,
    files: &[PathBuf],
    contents: &HashMap<PathBuf, String>,
    graph: Option<&RepoGraph>,
) {
    let _ = writeln!(out, "{}/", dir.display().to_string().blue().bold());
    
    for f in files {
        let name = f.file_name().unwrap_or_default().to_string_lossy();
        let stats = get_file_stats(f, contents);
        
        let meta = format!(
            "{} KB • {} toks",
            format!("{:.1}", stats.size_kb).yellow(),
            stats.tokens.to_string().cyan()
        );

        let _ = writeln!(out, "  ├── {name:<30} ({meta})");

        if let Some(g) = graph {
            render_dependencies(out, g, f);
        }
    }
    let _ = writeln!(out);
}

fn render_dependencies(out: &mut String, graph: &RepoGraph, file: &Path) {
    let deps = graph.neighbors(file);
    if deps.is_empty() {
        return;
    }
    
    for dep in deps {
        let dep_name = dep.to_string_lossy();
        let _ = writeln!(out, "  │   └── 🔗 {}", dep_name.dimmed());
    }
}

#[allow(clippy::cast_precision_loss)]
fn get_file_stats(
    path: &Path,
    contents: &HashMap<PathBuf, String>,
) -> FileStats {
    let content = contents.get(path).map_or("", String::as_str);
    let tokens = Tokenizer::count(content);
    let size_bytes = content.len();
    
    FileStats {
        size_kb: size_bytes as f64 / 1024.0,
        tokens,
    }
}