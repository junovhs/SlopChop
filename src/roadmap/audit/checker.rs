// src/roadmap/audit/checker.rs
//! Task verification logic.

use super::types::ViolationReason;
use crate::roadmap::slugify;
use crate::roadmap::types::Task;
use regex::Regex;
use std::fs;
use std::path::Path;

pub fn check_task(
    task: &Task,
    root: &Path,
    scanned: &[String],
    strict: bool,
) -> Option<ViolationReason> {
    if !task.tests.is_empty() {
        return check_explicit_anchors(task, root, strict);
    }
    check_heuristic(task, scanned)
}

fn check_explicit_anchors(task: &Task, root: &Path, strict: bool) -> Option<ViolationReason> {
    for anchor in &task.tests {
        if let Some(r) = verify_anchor(anchor, root) {
            return Some(r);
        }
        if strict {
            if let Some(r) = check_naming(task, anchor) {
                return Some(r);
            }
        }
    }
    None
}

fn check_heuristic(task: &Task, scanned: &[String]) -> Option<ViolationReason> {
    let slug = slugify(&task.text).replace('-', "_");
    let id_slug = task.id.replace('-', "_");

    let found = scanned
        .iter()
        .any(|f| f.contains(&slug) || f.contains(&id_slug));
    if found {
        None
    } else {
        Some(ViolationReason::NoTraceability)
    }
}

fn check_naming(task: &Task, anchor: &str) -> Option<ViolationReason> {
    let func_name = anchor.split("::").nth(1)?;
    let func_clean = func_name.trim();

    let func_base = func_clean.strip_prefix("test_").unwrap_or(func_clean);
    let id_normalized = task.id.replace('-', "_");
    let id_base = id_normalized
        .strip_prefix("test_")
        .unwrap_or(&id_normalized);

    if func_base != id_base {
        return Some(ViolationReason::NamingConventionMismatch {
            expected: format!("test_{id_base}"),
            actual: func_clean.to_string(),
        });
    }
    None
}

fn verify_anchor(anchor: &str, root: &Path) -> Option<ViolationReason> {
    let (file_part, fn_part) = anchor
        .split_once("::")
        .map_or((anchor, None), |(f, n)| (f, Some(n)));
    let path = root.join(file_part.trim());

    if !path.exists() || !path.is_file() {
        return Some(ViolationReason::MissingTestFile(
            file_part.trim().to_string(),
        ));
    }

    if let Some(func) = fn_part {
        if let Ok(content) = fs::read_to_string(&path) {
            if !has_definition(&path, &content, func.trim()) {
                return Some(ViolationReason::MissingTestFunction {
                    file: file_part.trim().to_string(),
                    function: func.trim().to_string(),
                });
            }
        }
    }
    None
}

fn has_definition(path: &Path, content: &str, name: &str) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let pattern = def_pattern(ext, name);

    let Ok(re) = Regex::new(&pattern) else {
        return content.contains(name);
    };

    let found = re
        .find_iter(content)
        .any(|m| !is_commented(content, m.start(), ext));
    found
}

fn def_pattern(ext: &str, name: &str) -> String {
    match ext {
        "rs" => format!(r"fn\s+{name}\b"),
        "py" => format!(r"def\s+{name}\b"),
        "go" => format!(r"func\s+{name}\b"),
        "js" | "ts" | "jsx" | "tsx" => format!(r"(function\s+{name}\b|const\s+{name}\s*=)"),
        _ => name.to_string(),
    }
}

fn is_commented(content: &str, idx: usize, ext: &str) -> bool {
    let line_start = content[..idx].rfind('\n').map_or(0, |i| i + 1);
    let prefix = content[line_start..idx].trim();
    match ext {
        "py" => prefix.starts_with('#'),
        _ => prefix.starts_with("//") || prefix.starts_with('*'),
    }
}
