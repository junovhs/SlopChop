// tests/unit_project.rs
use anyhow::Result;
use slopchop_core::project::ProjectType;
use std::fs::File;
use tempfile::TempDir;

#[test]
fn test_detect_rust() -> Result<()> {
    let temp = TempDir::new()?;
    File::create(temp.path().join("Cargo.toml"))?;
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Rust);
    Ok(())
}

#[test]
fn test_detect_node() -> Result<()> {
    let temp = TempDir::new()?;
    File::create(temp.path().join("package.json"))?;
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Node);
    Ok(())
}

#[test]
fn test_detect_python() -> Result<()> {
    let temp = TempDir::new()?;
    File::create(temp.path().join("requirements.txt"))?;
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Python);
    Ok(())
}

#[test]
fn test_detect_go() -> Result<()> {
    let temp = TempDir::new()?;
    File::create(temp.path().join("go.mod"))?;
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Go);
    Ok(())
}

#[test]
fn test_detect_unknown() -> Result<()> {
    let temp = TempDir::new()?;
    assert_eq!(ProjectType::detect_in(temp.path()), ProjectType::Unknown);
    Ok(())
}