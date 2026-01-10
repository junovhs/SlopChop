// src/analysis/mod.rs
//! Core analysis logic (The "Rule Engine").

pub mod ast;
pub mod checks;
pub mod metrics;
pub mod safety;
pub mod v2;
pub mod file_analysis;
pub mod logic;
mod engine;

pub use engine::RuleEngine;