use std::fs;
use tempfile::tempdir;
use warden_core::project::ProjectType;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_detect_rust() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;
    
    fs::write("Cargo.toml", "[package]")?;
    assert_eq!(ProjectType::detect(), ProjectType::Rust);
    Ok(())
}

#[test]
fn test_detect_node() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    fs::write("package.json", "{}")?;
    assert_eq!(ProjectType::detect(), ProjectType::Node);
    Ok(())
}

#[test]
fn test_detect_python() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    fs::write("requirements.txt", "flask")?;
    assert_eq!(ProjectType::detect(), ProjectType::Python);
    Ok(())
}

#[test]
fn test_detect_go() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    fs::write("go.mod", "module test")?;
    assert_eq!(ProjectType::detect(), ProjectType::Go);
    Ok(())
}

#[test]
fn test_detect_unknown() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;
    
    // Empty directory
    assert_eq!(ProjectType::detect(), ProjectType::Unknown);
    Ok(())
}

// --- Helper for changing CWD strictly for tests ---
struct ScopedDir {
    original: std::path::PathBuf,
}

impl ScopedDir {
    fn new(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let original = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { original })
    }
}

impl Drop for ScopedDir {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}