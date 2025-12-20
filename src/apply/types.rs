use crate::config::Config;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Update,
    New,
    Delete,
}

#[derive(Debug, Clone)]
pub struct ManifestEntry {
    pub path: String,
    pub operation: Operation,
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub content: String,
    pub line_count: usize,
}

#[derive(Debug)]
pub enum ApplyOutcome {
    Success {
        written: Vec<String>,
        deleted: Vec<String>,
        backed_up: bool,
    },
    ValidationFailure {
        errors: Vec<String>,
        missing: Vec<String>,
        ai_message: String,
    },
    ParseError(String),
    WriteError(String),
}

#[derive(Debug, Clone, Default)]
pub enum ApplyInput {
    #[default]
    Clipboard,
    Stdin,
    File(PathBuf),
}

pub struct ApplyContext<'a> {
    pub config: &'a Config,
    pub force: bool,
    pub dry_run: bool,
    pub input: ApplyInput,
}

impl<'a> ApplyContext<'a> {
    #[must_use]
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            force: false,
            dry_run: false,
            input: ApplyInput::default(),
        }
    }
}

pub type Manifest = Vec<ManifestEntry>;
pub type ExtractedFiles = HashMap<String, FileContent>;