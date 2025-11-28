// src/heuristics.rs
use crate::config::{CODE_BARE_PATTERN, CODE_EXT_PATTERN};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const MIN_TEXT_ENTROPY: f64 = 3.5;
const MAX_TEXT_ENTROPY: f64 = 5.5;

const BUILD_SYSTEM_MARKERS: &[&str] = &[
    "find_package",
    "add_executable",
    "target_link_libraries",
    "cmake_minimum_required",
    "project(",
    "add-apt-repository",
    "conanfile.py",
    "dependency",
    "require",
    "include",
    "import",
    "version",
    "dependencies",
];

/// Pre-compiled regex for known code file extensions.
static CODE_EXT_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(CODE_EXT_PATTERN)
        .map_err(|e| eprintln!("Warning: Invalid CODE_EXT_PATTERN: {e}"))
        .ok()
});

/// Pre-compiled regex for known code files without extensions.
static CODE_BARE_RE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(CODE_BARE_PATTERN)
        .map_err(|e| eprintln!("Warning: Invalid CODE_BARE_PATTERN: {e}"))
        .ok()
});

pub struct HeuristicFilter;

impl Default for HeuristicFilter {
    fn default() -> Self {
        Self::new()
    }
}

impl HeuristicFilter {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn filter(&self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        files
            .into_iter()
            .filter(|path| Self::should_keep(path))
            .collect()
    }

    fn should_keep(path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Fast path: known code files always pass
        if Self::is_known_code_file(&path_str) {
            return true;
        }

        // Check entropy for unknown files
        let Ok(entropy) = calculate_entropy(path) else {
            return false;
        };

        if !(MIN_TEXT_ENTROPY..=MAX_TEXT_ENTROPY).contains(&entropy) {
            return false;
        }

        // Check for build system markers
        if Self::has_build_markers(path) {
            return true;
        }

        true
    }

    fn is_known_code_file(path_str: &str) -> bool {
        let ext_match = CODE_EXT_RE.as_ref().is_some_and(|re| re.is_match(path_str));

        let bare_match = CODE_BARE_RE
            .as_ref()
            .is_some_and(|re| re.is_match(path_str));

        ext_match || bare_match
    }

    fn has_build_markers(path: &Path) -> bool {
        let Ok(content) = fs::read_to_string(path) else {
            return false;
        };

        let lower_content = content.to_lowercase();
        BUILD_SYSTEM_MARKERS
            .iter()
            .any(|marker| lower_content.contains(marker))
    }
}

fn calculate_entropy(path: &Path) -> std::io::Result<f64> {
    let bytes = fs::read(path)?;
    if bytes.is_empty() {
        return Ok(0.0);
    }

    let mut freq_map: HashMap<u8, u32> = HashMap::new();
    for &byte in &bytes {
        *freq_map.entry(byte).or_insert(0) += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    let len = bytes.len() as f64;

    let entropy = freq_map.values().fold(0.0, |acc, &count| {
        let probability = f64::from(count) / len;
        acc - probability * probability.log2()
    });

    Ok(entropy)
}
