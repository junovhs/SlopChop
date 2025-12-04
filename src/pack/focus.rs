// src/pack/focus.rs
//! Focus mode computation for foveal/peripheral packing.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use crate::graph::rank::RepoGraph;

/// Computes foveal (full) and peripheral (skeleton) file sets.
#[must_use]
pub fn compute_sets(
    all_files: &[PathBuf],
    focus: &[PathBuf],
    depth: usize,
) -> (HashSet<PathBuf>, HashSet<PathBuf>) {
    let contents = read_files(all_files);
    let graph = build_graph(&contents);
    let all_set: HashSet<_> = all_files.iter().cloned().collect();

    let foveal = collect_foveal(focus, &all_set);
    let peripheral = collect_peripheral(&foveal, &graph, &all_set, depth);

    (foveal, peripheral)
}

fn read_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn build_graph(contents: &HashMap<PathBuf, String>) -> RepoGraph {
    let file_vec: Vec<_> = contents
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();
    RepoGraph::build(&file_vec)
}

fn collect_foveal(focus: &[PathBuf], all_set: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    focus
        .iter()
        .filter(|f| all_set.contains(*f))
        .cloned()
        .collect()
}

fn collect_peripheral(
    foveal: &HashSet<PathBuf>,
    graph: &RepoGraph,
    all_set: &HashSet<PathBuf>,
    depth: usize,
) -> HashSet<PathBuf> {
    let mut peripheral = HashSet::new();
    let mut frontier = foveal.clone();

    for _ in 0..depth {
        let next = expand_frontier(&frontier, foveal, &peripheral, graph, all_set);
        peripheral.extend(next.iter().cloned());
        frontier = next;
    }

    peripheral
}

fn expand_frontier(
    frontier: &HashSet<PathBuf>,
    foveal: &HashSet<PathBuf>,
    peripheral: &HashSet<PathBuf>,
    graph: &RepoGraph,
    all_set: &HashSet<PathBuf>,
) -> HashSet<PathBuf> {
    let mut next = HashSet::new();

    for anchor in frontier {
        for neighbor in graph.neighbors(anchor) {
            if should_include(&neighbor, foveal, peripheral, all_set) {
                next.insert(neighbor);
            }
        }
    }

    next
}

fn should_include(
    path: &PathBuf,
    foveal: &HashSet<PathBuf>,
    peripheral: &HashSet<PathBuf>,
    all_set: &HashSet<PathBuf>,
) -> bool {
    !foveal.contains(path) && !peripheral.contains(path) && all_set.contains(path)
}
