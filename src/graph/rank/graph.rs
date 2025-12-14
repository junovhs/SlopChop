// src/graph/rank/graph.rs
//! The dependency graph structure and builder.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::pagerank;
use super::tags::{Tag, TagKind};
use crate::graph::defs;
use crate::graph::imports;

/// Extracted tags with their lookup maps.
pub struct ExtractedTags {
    pub tags: Vec<Tag>,
    pub defines: HashMap<String, HashSet<PathBuf>>,
    pub references: HashMap<String, Vec<PathBuf>>,
}

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
        let extracted = extract_all_tags(files);
        let edges = build_edges(&extracted.defines, &extracted.references);
        let all_files = collect_all_files(&edges);
        let ranks = pagerank::compute(&edges, &all_files, None);

        Self {
            tags: extracted.tags,
            defines: extracted.defines,
            references: extracted.references,
            ranks,
        }
    }

    /// Re-ranks with a specific anchor file.
    pub fn focus_on(&mut self, anchor: &Path) {
        let edges = build_edges(&self.defines, &self.references);
        let all_files = collect_all_files(&edges);
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

fn extract_all_tags(files: &[(PathBuf, String)]) -> ExtractedTags {
    let mut tags = Vec::new();
    let mut defines: HashMap<String, HashSet<PathBuf>> = HashMap::new();
    let mut references: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for (path, content) in files {
        extract_defs(path, content, &mut tags, &mut defines);
        extract_refs(path, content, &mut tags, &mut references);
    }

    ExtractedTags {
        tags,
        defines,
        references,
    }
}

fn extract_defs(
    path: &Path,
    content: &str,
    tags: &mut Vec<Tag>,
    defines: &mut HashMap<String, HashSet<PathBuf>>,
) {
    for def in defs::extract(path, content) {
        defines
            .entry(def.name.clone())
            .or_default()
            .insert(path.to_path_buf());
        tags.push(Tag {
            file: path.to_path_buf(),
            name: def.name,
            kind: TagKind::Def,
            line: def.line,
            signature: Some(def.signature),
        });
    }
}

fn extract_refs(
    path: &Path,
    content: &str,
    tags: &mut Vec<Tag>,
    references: &mut HashMap<String, Vec<PathBuf>>,
) {
    for ref_name in imports::extract(path, content) {
        let symbol = ref_name.split("::").last().unwrap_or(&ref_name).to_string();
        references
            .entry(symbol.clone())
            .or_default()
            .push(path.to_path_buf());
        tags.push(Tag {
            file: path.to_path_buf(),
            name: symbol,
            kind: TagKind::Ref,
            line: 0,
            signature: None,
        });
    }
}

fn build_edges(
    defines: &HashMap<String, HashSet<PathBuf>>,
    references: &HashMap<String, Vec<PathBuf>>,
) -> HashMap<PathBuf, HashMap<PathBuf, usize>> {
    let mut edges: HashMap<PathBuf, HashMap<PathBuf, usize>> = HashMap::new();

    for symbol in defines.keys().filter(|k| references.contains_key(*k)) {
        add_symbol_edges(symbol, defines, references, &mut edges);
    }

    edges
}

fn add_symbol_edges(
    symbol: &str,
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, Vec<PathBuf>>,
    edges: &mut HashMap<PathBuf, HashMap<PathBuf, usize>>,
) {
    let Some(def_files) = def_map.get(symbol) else {
        return;
    };
    let Some(ref_files) = ref_map.get(symbol) else {
        return;
    };

    for ref_file in ref_files {
        for def_file in def_files {
            if ref_file != def_file {
                *edges
                    .entry(ref_file.clone())
                    .or_default()
                    .entry(def_file.clone())
                    .or_default() += 1;
            }
        }
    }
}

fn collect_all_files(edges: &HashMap<PathBuf, HashMap<PathBuf, usize>>) -> HashSet<PathBuf> {
    let mut files = HashSet::new();
    for (src, targets) in edges {
        files.insert(src.clone());
        files.extend(targets.keys().cloned());
    }
    files
}

fn collect_related(
    def_map: &HashMap<String, HashSet<PathBuf>>,
    ref_map: &HashMap<String, Vec<PathBuf>>,
    anchor: &PathBuf,
    direction: Direction,
    result: &mut HashSet<PathBuf>,
) {
    match direction {
        Direction::Dependent => {
            // Who imports me?
            // Symbols I define -> Who references them?
            for (symbol, def_files) in def_map {
                if !def_files.contains(anchor) {
                    continue;
                }
                if let Some(refs) = ref_map.get(symbol) {
                    add_files(refs, anchor, result);
                }
            }
        }
        Direction::Dependency => {
            // What do I import?
            // Symbols I reference -> Who defines them?
            for (symbol, ref_files) in ref_map {
                if !ref_files.contains(anchor) {
                    continue;
                }
                if let Some(defs) = def_map.get(symbol) {
                    add_files(defs, anchor, result);
                }
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