// src/graph/rank/mod.rs
//! Builds a dependency graph and ranks files using `PageRank`.

pub mod builder;
mod graph;
mod pagerank;
mod tags;

pub use graph::RepoGraph;
pub use tags::{Tag, TagKind};