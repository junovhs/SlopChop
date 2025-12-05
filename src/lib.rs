// src/lib.rs
//! SlopChop - Code quality guardian for AI-assisted development.

// Allow noisy pedantic lints that are pure style, not correctness.
// Keeps pedantic enabled for the valuable lints while reducing churn.
#![allow(clippy::doc_markdown)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::format_push_string)]

pub mod analysis;
pub mod apply;
pub mod clean;
pub mod cli;
pub mod clipboard;
pub mod config;
pub mod constants;
pub mod context;
pub mod discovery;
pub mod error;
pub mod graph;
pub mod pack;
pub mod project;
pub mod prompt;
pub mod reporting;
pub mod roadmap;
pub mod skeleton;
pub mod spinner;
pub mod tokens;
pub mod trace;
pub mod tui;
pub mod types;
pub mod wizard;

// Legacy/Test compatibility aliases
pub use analysis as rules;