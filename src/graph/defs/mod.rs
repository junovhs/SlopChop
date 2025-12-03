// src/graph/defs/mod.rs
//! Extracts symbol DEFINITIONS from source files using tree-sitter.

mod extract;
mod queries;

pub use extract::{extract, DefKind, Definition};
