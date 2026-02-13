//! Core analysis logic (The "Rule Engine").

pub mod ast;
pub mod checks;
pub mod cognitive;
pub mod metrics;
pub mod safety;
pub mod structural;
pub mod scope;
pub mod inspector;
pub mod aggregator;
pub mod deep;
pub mod visitor;
pub mod extract;
pub mod extract_impl; // New module
pub mod patterns;
pub mod worker;

mod engine;

pub use engine::Engine;
pub use aggregator::FileAnalysis;
