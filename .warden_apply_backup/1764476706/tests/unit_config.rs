use std::fs;
use tempfile::tempdir;
use warden_core::config::{Config, RuleConfig};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_load_toml() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    let toml = r#"
[rules]
max_file_tokens = 1000
max_cyclomatic_complexity = 5

[commands]
check = "echo checking"
"#;
    fs::write("warden.toml", toml)?;

    let mut config = Config::new();
    config.load_local_config();

    assert_eq!(config.rules.max_file_tokens, 1000);
    assert_eq!(config.rules.max_cyclomatic_complexity, 5);
    Ok(())
}

#[test]
fn test_defaults() {
    let config = RuleConfig::default();
    assert_eq!(config.max_file_tokens, 2000);
    assert_eq!(config.max_cyclomatic_complexity, 5);
}

#[test]
fn test_command_single() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;
    
    fs::write("warden.toml", r#"[commands]
check = "test""#)?;
    
    let mut config = Config::new();
    config.load_local_config();
    
    let cmds = config.commands.get("check").ok_or("Missing check")?;
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0], "test");
    Ok(())
}

#[test]
fn test_command_list() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;
    
    fs::write("warden.toml", r#"[commands]
check = ["one", "two"]"#)?;
    
    let mut config = Config::new();
    config.load_local_config();
    
    let cmds = config.commands.get("check").ok_or("Missing check")?;
    assert_eq!(cmds.len(), 2);
    assert_eq!(cmds[0], "one");
    assert_eq!(cmds[1], "two");
    Ok(())
}

#[test]
fn test_wardenignore() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    fs::write(".wardenignore", "target/\n*.log")?;
    
    let mut config = Config::new();
    config.load_local_config();
    
    assert!(!config.exclude_patterns.is_empty());
    // Basic verification that patterns loaded
    let any_matches = config.exclude_patterns.iter().any(|r| r.as_str().contains("target/"));
    assert!(any_matches);
    Ok(())
}

#[test]
fn test_ignore_tokens_on() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    let toml = r#"
[rules]
ignore_tokens_on = ["docs/", "legacy.rs"]
"#;
    fs::write("warden.toml", toml)?;
    
    let mut config = Config::new();
    config.load_local_config();
    
    assert!(config.rules.ignore_tokens_on.contains(&"docs/".to_string()));
    assert!(config.rules.ignore_tokens_on.contains(&"legacy.rs".to_string()));
    Ok(())
}

#[test]
fn test_ignore_naming_on() -> TestResult {
    let dir = tempdir()?;
    let _in_dir = ScopedDir::new(dir.path())?;

    let toml = r#"
[rules]
ignore_naming_on = ["tests"]
"#;
    fs::write("warden.toml", toml)?;
    
    let mut config = Config::new();
    config.load_local_config();
    
    assert!(config.rules.ignore_naming_on.contains(&"tests".to_string()));
    Ok(())
}

// --- Helper ---
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