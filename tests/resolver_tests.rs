// tests/resolver_tests.rs
//! Integration tests for import resolution.

use anyhow::Result;
use slopchop_core::graph::resolver::resolve;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_rust_crate_path() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();

    fs::create_dir_all(root.join("src/config"))?;
    fs::write(root.join("src/lib.rs"), "")?;
    fs::write(root.join("src/config/types.rs"), "")?;

    let current = root.join("src/lib.rs");
    let resolved = resolve(root, &current, "crate::config::types");

    assert_eq!(resolved, Some(root.join("src/config/types.rs")));
    Ok(())
}

#[test]
fn test_rust_sibling_module() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();

    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src/main.rs"), "")?;
    fs::write(root.join("src/util.rs"), "")?;

    let current = root.join("src/main.rs");
    let resolved = resolve(root, &current, "util");

    assert_eq!(resolved, Some(root.join("src/util.rs")));
    Ok(())
}

#[test]
fn test_rust_mod_index() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();

    fs::create_dir_all(root.join("src/utils"))?;
    fs::write(root.join("src/main.rs"), "")?;
    fs::write(root.join("src/utils/mod.rs"), "")?;

    let current = root.join("src/main.rs");
    let resolved = resolve(root, &current, "utils");

    assert_eq!(resolved, Some(root.join("src/utils/mod.rs")));
    Ok(())
}

#[test]
fn test_ts_relative_import() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();

    fs::write(root.join("app.ts"), "")?;
    fs::write(root.join("cmp.tsx"), "")?;

    let current = root.join("app.ts");
    let resolved = resolve(root, &current, "./cmp");

    assert_eq!(resolved, Some(root.join("cmp.tsx")));
    Ok(())
}

#[test]
fn test_ts_path_alias() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();

    fs::create_dir_all(root.join("src/components"))?;
    fs::write(root.join("src/components/Button.tsx"), "")?;
    fs::write(root.join("src/app.ts"), "")?;

    let tsconfig = r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["src/*"]}}}"#;
    fs::write(root.join("tsconfig.json"), tsconfig)?;

    let current = root.join("src/app.ts");
    let resolved = resolve(root, &current, "@/components/Button");

    assert!(resolved.is_some());
    assert!(resolved.as_ref().is_some_and(|p| p.ends_with("Button.tsx")));
    Ok(())
}

#[test]
fn test_node_module_skipped() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();

    fs::write(root.join("app.ts"), "")?;

    let current = root.join("app.ts");

    // These should return None (external packages)
    assert!(resolve(root, &current, "react").is_none());
    assert!(resolve(root, &current, "@types/node").is_none());
    assert!(resolve(root, &current, "lodash").is_none());

    Ok(())
}