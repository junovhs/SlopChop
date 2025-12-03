// src/roadmap/audit/types.rs
//! Audit types and data structures.

#[derive(Debug, Clone, Copy)]
pub struct AuditOptions {
    pub strict: bool,
}

#[derive(Debug)]
pub struct AuditViolation {
    pub task_id: String,
    pub task_text: String,
    pub reason: ViolationReason,
}

#[derive(Debug)]
pub enum ViolationReason {
    MissingTestFile(String),
    MissingTestFunction { file: String, function: String },
    NamingConventionMismatch { expected: String, actual: String },
    NoTraceability,
}

#[derive(Debug, Default)]
pub struct AuditReport {
    pub violations: Vec<AuditViolation>,
    pub total_checked: usize,
}

impl AuditReport {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

