// src/trace/options.rs
//! Options for the trace command.

use std::path::PathBuf;

/// Options for the trace command.
#[derive(Default)]
pub struct TraceOptions {
    pub anchor: PathBuf,
    pub depth: usize,
    pub budget: usize,
    pub stdout: bool,
}

impl TraceOptions {
    #[must_use]
    pub fn new(anchor: PathBuf) -> Self {
        Self {
            anchor,
            depth: 2,
            budget: 4000,
            stdout: false,
        }
    }

    #[must_use]
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    #[must_use]
    pub fn with_budget(mut self, budget: usize) -> Self {
        self.budget = budget;
        self
    }
}

