// tests/cli_map.rs
use anyhow::Result;
use std::fs;
use std::sync::{LazyLock, Mutex, PoisonError};
use tempfile::tempdir;
use warden_core::trace;

// Protect CWD changes with a global mutex
static CWD_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn test_map_basic() -> Result<()> {
    // Handle poisoned lock by recovering (we just need serialization)
    let _lock = CWD_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    
    let temp = tempdir()?;
    let _guard = TestDirectoryGuard::new(temp.path());
    
    // Use .rs extension so discovery heuristics accept it
    fs::write("main.rs", "fn main() {}")?;

    let result = trace::map(false)?;
    assert!(result.contains("main.rs"));
    Ok(())
}

#[test]
fn test_map_tree() -> Result<()> {
    let _lock = CWD_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    
    let temp = tempdir()?;
    let _guard = TestDirectoryGuard::new(temp.path());
    fs::create_dir("src")?;
    fs::write("src/lib.rs", "fn lib() {}")?;

    let result = trace::map(false)?;
    assert!(result.contains("src/"));
    assert!(result.contains("lib.rs"));
    Ok(())
}

#[test]
fn test_map_deps() -> Result<()> {
    let _lock = CWD_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
    
    let temp = tempdir()?;
    let _guard = TestDirectoryGuard::new(temp.path());
    fs::create_dir("src")?;
    
    // Create an explicit dependency link
    // lib.rs refers to 'Helper'
    fs::write("src/lib.rs", "use crate::utils::Helper;")?;
    
    // utils.rs defines 'Helper'
    fs::write("src/utils.rs", "pub struct Helper;")?;

    // Enable dependency visualization
    let result = trace::map(true)?;
    
    assert!(result.contains("src/"));
    assert!(result.contains("lib.rs"));
    // Should show link from lib.rs -> utils.rs
    assert!(result.contains("🔗"));
    assert!(result.contains("utils.rs"));
    Ok(())
}

// Helper to change directory for test duration
struct TestDirectoryGuard {
    original: std::path::PathBuf,
}

impl TestDirectoryGuard {
    fn new(path: &std::path::Path) -> Self {
        let original = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self { original }
    }
}

impl Drop for TestDirectoryGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}