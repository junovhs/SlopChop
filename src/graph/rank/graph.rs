// src/graph/rank/graph.rs
//! The dependency graph structure and query interface.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::builder;
use super::pagerank;
use super::tags::{Tag, TagKind};

/// The dependency graph and ranker.
#[derive(Clone)]
pub struct RepoGraph {
    tags: Vec<Tag>,
    defines: HashMap<String, HashSet<PathBuf>>,
    references: HashMap<String, Vec<PathBuf>>,
    ranks: HashMap<PathBuf, f64>,
}

#[derive(Clone, Copy)]
enum Direction {
    Dependency, // Fan-out: What I import
    Dependent,  // Fan-in: Who imports me
}

impl RepoGraph {
    /// Builds the graph from files and their contents.
    #[must_use]
    pub fn build(files: &[(PathBuf, String)]) -> Self {
        let data = builder::build_data(files);
        let ranks = pagerank::compute(&data.edges, &data.all_files, None);

        Self {
            tags: data.tags,
            defines: data.defines,
            references: data.references,
            ranks,
        }
    }

    /// Re-ranks with a specific anchor file.
    pub fn focus_on(&mut self, anchor: &Path) {
        let (edges, all_files) = builder::rebuild_topology(&self.defines, &self.references);
        self.ranks = pagerank::compute(&edges, &all_files, Some(&anchor.to_path_buf()));
    }

    /// Returns files ranked by importance.
    #[must_use]
    pub fn ranked_files(&self) -> Vec<(PathBuf, f64)> {
        let mut ranked: Vec<_> = self.ranks.iter().map(|(p, r)| (p.clone(), *r)).collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }

    /// Returns files directly connected to the anchor (both directions).
    #[must_use]
    pub fn neighbors(&self, anchor: &Path) -> Vec<PathBuf> {
        let anchor_path = anchor.to_path_buf();
        let mut result = HashSet::new();

        collect_related(
            &self.defines,
            &self.references,
            &anchor_path,
            Direction::Dependent,
            &mut result,
        );
        collect_related(
            &self.defines,
            &self.references,
            &anchor_path,
            Direction::Dependency,
            &mut result,
        );

        result.into_iter().collect()
    }

    /// Returns files that this file depends on (fan-out / what I import).
    #[must_use]
    pub fn dependencies(&self, anchor: &Path) -> Vec<PathBuf> {
        self.query_direction(anchor, Direction::Dependency)
    }

    /// Returns files that depend on this file (fan-in / who imports me).
    #[must_use]
    pub fn dependents(&self, anchor: &Path) -> Vec<PathBuf> {
        self.query_direction(anchor, Direction::Dependent)
    }

    fn query_direction(&self, anchor: &Path, dir: Direction) -> Vec<PathBuf> {
        let anchor_path = anchor.to_path_buf();
        let mut result = HashSet::new();
        collect_related(
            &self.defines,
            &self.references,
            &anchor_path,
            dir,
            &mut result,
        );
        let mut deps: Vec<_> = result.into_iter().collect();
        deps.sort();
        deps
    }

    /// Returns definition tags only.
    #[must_use]
    pub fn definition_tags(&self) -> Vec<Tag> {
        self.tags
            .iter()
            .filter(|t| t.kind == TagKind::Def)
            .cloned()
            .collect()
    }

    /// Returns true if this file is a hub (high fan-in).
    #[must_use]
    pub fn is_hub(&self, anchor: &Path, threshold: usize) -> bool {
        self.dependents(anchor).len() >= threshold
    }
}

fn collect_related(
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, Vec<PathBuf>>,
    anchor: &PathBuf,
    direction: Direction,
    result: &mut HashSet<PathBuf>,
) {
    match direction {
        Direction::Dependent => collect_dependents(def_map, ref_map, anchor, result),
        Direction::Dependency => collect_dependencies(def_map, ref_map, anchor, result),
    }
}

fn collect_dependents(
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, Vec<PathBuf>>,
    anchor: &PathBuf,
    result: &mut HashSet<PathBuf>,
) {
    // Who imports me?
    // Symbols I define -> Who references them?
    for (symbol, def_files) in def_map {
        if def_files.contains(anchor) {
            if let Some(refs) = ref_map.get(symbol) {
                add_files(refs, anchor, result);
            }
        }
    }
}

fn collect_dependencies(
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, Vec<PathBuf>>,
    anchor: &PathBuf,
    result: &mut HashSet<PathBuf>,
) {
    // What do I import?
    // Symbols I reference -> Who defines them?
    for (symbol, ref_files) in ref_map {
        if ref_files.contains(anchor) {
            if let Some(defs) = def_map.get(symbol) {
                add_files(defs, anchor, result);
            }
        }
    }
}

trait FileCollection {
    fn iter_files(&self) -> impl Iterator<Item = &PathBuf>;
}

impl FileCollection for Vec<PathBuf> {
    fn iter_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.iter()
    }
}

impl FileCollection for HashSet<PathBuf> {
    fn iter_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.iter()
    }
}

fn add_files<C: FileCollection>(collection: &C, anchor: &PathBuf, result: &mut HashSet<PathBuf>) {
    for f in collection.iter_files() {
        if f != anchor {
            result.insert(f.clone());
        }
    }
}