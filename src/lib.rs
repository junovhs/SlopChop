// src/lib.rs
pub mod analysis;
pub mod apply;
pub mod clean;
pub mod clipboard;
pub mod config;
pub mod constants;
pub mod discovery;
pub mod error;
pub mod graph;
pub mod pack;
pub mod project;
pub mod prompt;
pub mod reporting;
pub mod roadmap;
pub mod skeleton;
pub mod tokens;
pub mod trace;
pub mod tui;
pub mod types;
pub mod wizard;

// Legacy/Test compatibility aliases
pub use analysis as rules;
