// src/graph/resolver.rs
use std::path::{Path, PathBuf};

/// Resolves an import string to a likely file path on disk.
///
/// # Arguments
/// * `project_root` - The root of the repository.
/// * `current_file` - The path of the file containing the import.
/// * `import_str` - The raw import string (e.g., "`crate::foo`", "./utils").
///
/// # Returns
/// `Option<PathBuf>` if a matching local file is found.
#[must_use]
pub fn resolve(project_root: &Path, current_file: &Path, import_str: &str) -> Option<PathBuf> {
    let ext = current_file.extension().and_then(|s| s.to_str())?;

    match ext {
        "rs" => resolve_rust(project_root, current_file, import_str),
        "ts" | "tsx" | "js" | "jsx" => resolve_js(project_root, current_file, import_str),
        "py" => resolve_python(project_root, current_file, import_str),
        _ => None,
    }
}

fn resolve_rust(root: &Path, current: &Path, import: &str) -> Option<PathBuf> {
    if let Some(rest) = import.strip_prefix("crate::") {
        return resolve_crate_path(root, rest);
    }

    if import.starts_with("super::") {
        return resolve_super_path(current, import);
    }

    if import.starts_with("self::") {
        return resolve_self_path(current, import);
    }

    if !import.contains("::") {
        return resolve_sibling_path(current, import);
    }

    None
}

fn resolve_crate_path(root: &Path, rest: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = rest.split("::").collect();
    let base = root.join("src");
    check_variations(&base, &parts, "rs")
}

fn resolve_super_path(current: &Path, import: &str) -> Option<PathBuf> {
    let mut parts: Vec<&str> = import.split("::").collect();
    let mut dir = current.parent()?;

    // Consume super segments
    while let Some(&"super") = parts.first() {
        parts.remove(0);
        dir = dir.parent()?;
    }

    if parts.is_empty() {
        return None;
    }

    check_variations(dir, &parts, "rs")
}

fn resolve_self_path(current: &Path, import: &str) -> Option<PathBuf> {
    let rest = import.strip_prefix("self::")?;
    let parts: Vec<&str> = rest.split("::").collect();
    let dir = current.parent()?;
    check_variations(dir, &parts, "rs")
}

fn resolve_sibling_path(current: &Path, import: &str) -> Option<PathBuf> {
    let parent = current.parent()?;
    let parts = vec![import];
    check_variations(parent, &parts, "rs")
}

fn resolve_js(_root: &Path, current: &Path, import: &str) -> Option<PathBuf> {
    if !import.starts_with('.') {
        return None;
    }

    let parent = current.parent()?;
    let path = parent.join(import);

    if let Some(p) = check_js_file(&path) {
        return Some(p);
    }
    check_js_directory(&path)
}

fn check_js_file(path: &Path) -> Option<PathBuf> {
    if path.exists() && path.is_file() {
        return Some(path.to_path_buf());
    }

    let extensions = ["ts", "tsx", "js", "jsx", "json"];
    for ext in extensions {
        let p = path.with_extension(ext);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn check_js_directory(path: &Path) -> Option<PathBuf> {
    if !path.is_dir() {
        return None;
    }

    let extensions = ["ts", "tsx", "js", "jsx", "json"];
    for ext in extensions {
        let p = path.join(format!("index.{ext}"));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn resolve_python(root: &Path, _current: &Path, import: &str) -> Option<PathBuf> {
    // 1. Handle Relative "from . import foo" -> "."
    if import.starts_with('.') {
        return None; // Simplified: assuming simple relative import for now
    }

    // 2. Absolute (from root)
    let parts: Vec<&str> = import.split('.').collect();
    check_variations(root, &parts, "py")
}

fn check_variations(base: &Path, parts: &[&str], ext: &str) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    for part in parts {
        current.push(part);
    }

    // Variation A: path.ext
    let file_path = current.with_extension(ext);
    if file_path.exists() {
        return Some(file_path);
    }

    // Variation B: path/mod.rs or path/__init__.py
    let index_name = match ext {
        "rs" => "mod.rs",
        "py" => "__init__.py",
        _ => return None,
    };

    let index_path = current.join(index_name);
    if index_path.exists() {
        return Some(index_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs;
    use tempfile::tempdir;

    struct ResolverTestCase<'a> {
        files: Vec<(&'a str, &'a str)>, // (path, content)
        current_file: &'a str,
        import_str: &'a str,
        expected_path: &'a str,
    }

    #[test]
    fn test_resolution_scenarios() -> Result<()> {
        let cases = vec![
            // 1. Relative Module
            ResolverTestCase {
                files: vec![
                    ("src/main.rs", "mod util;"),
                    ("src/util.rs", "// util"),
                ],
                current_file: "src/main.rs",
                import_str: "util",
                expected_path: "src/util.rs",
            },
            // 2. Crate Path
            ResolverTestCase {
                files: vec![
                    ("src/lib.rs", "use crate::config::types;"),
                    ("src/config/types.rs", "// types"),
                ],
                current_file: "src/lib.rs",
                import_str: "crate::config::types",
                expected_path: "src/config/types.rs",
            },
            // 3. Mod Index (mod.rs)
            ResolverTestCase {
                files: vec![
                    ("src/main.rs", "mod utils;"),
                    ("src/utils/mod.rs", "// mod.rs"),
                ],
                current_file: "src/main.rs",
                import_str: "utils",
                expected_path: "src/utils/mod.rs",
            },
            // 4. Super Path
            ResolverTestCase {
                files: vec![
                    ("src/lib.rs", "// lib"),
                    ("src/parent/child.rs", "use super::lib;"),
                ],
                current_file: "src/parent/child.rs",
                import_str: "super::lib",
                expected_path: "src/lib.rs",
            },
            // 5. Self Path
            ResolverTestCase {
                files: vec![
                    ("src/main.rs", ""),
                    ("src/util.rs", ""),
                ],
                current_file: "src/main.rs",
                import_str: "self::util",
                expected_path: "src/util.rs",
            },
            // 6. JS Relative
            ResolverTestCase {
                files: vec![
                    ("app.ts", ""),
                    ("cmp.tsx", ""),
                ],
                current_file: "app.ts",
                import_str: "./cmp",
                expected_path: "cmp.tsx",
            },
        ];

        for case in cases {
            let temp = tempdir()?;
            let root = temp.path();

            for (rel_path, content) in case.files {
                let p = root.join(rel_path);
                if let Some(parent) = p.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&p, content)?;
            }

            let current = root.join(case.current_file);
            let expected = root.join(case.expected_path);

            let resolved = resolve(root, &current, case.import_str);
            assert_eq!(resolved, Some(expected), "Failed for import '{}'", case.import_str);
        }
        Ok(())
    }
}