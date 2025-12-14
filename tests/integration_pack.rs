use anyhow::Result;
use slopchop_core::config::Config;
use slopchop_core::pack::{self, PackOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_nabla_delimiters_are_unique() -> Result<()> {
    // Legacy name for roadmap compatibility. Tests SlopChop Protocol delimiters.
    let temp = tempdir()?;
    let root = temp.path();
    let file_path = root.join("test.rs");
    fs::write(&file_path, "fn main() {}")?;

    let config = Config::default();
    let opts = PackOptions {
        stdout: true,
        ..Default::default()
    };

    let content = pack::generate_content(&[file_path], &opts, &config)?;

    assert!(content.contains("#__SLOPCHOP_FILE__#"));
    assert!(content.contains("#__SLOPCHOP_END__#"));
    // Verify old unicode symbols are gone
    assert!(!content.contains("???"));
    Ok(())
}

#[test]
fn test_nabla_format_structure() -> Result<()> {
    // Legacy name for roadmap compatibility. Tests SlopChop Protocol format.
    let temp = tempdir()?;
    let root = temp.path();
    let file_path = root.join("src/main.rs");
    fs::create_dir_all(root.join("src"))?;
    fs::write(&file_path, "code")?;

    let config = Config::default();
    let opts = PackOptions::default();

    let content = pack::generate_content(std::slice::from_ref(&file_path), &opts, &config)?;

    // Normalize path for test consistency
    let p_str = file_path.to_string_lossy().replace('\\', "/");
    let header = format!("#__SLOPCHOP_FILE__# {p_str}");

    assert!(content.contains(&header));
    assert!(content.contains("code"));
    assert!(content.contains("#__SLOPCHOP_END__#"));
    Ok(())
}

#[test]
fn test_prompt_content() -> Result<()> {
    let config = Config::default();
    let generator = slopchop_core::prompt::PromptGenerator::new(config.rules);
    let prompt = generator.generate()?;

    let required = [
        "THE 3 LAWS",
        "LAW OF ATOMICITY",
        "Files: MUST be < 2000 tokens",
        "Complexity: MUST be <= 8",
        "OUTPUT FORMAT (MANDATORY)",
        "#__SLOPCHOP_FILE__#",
        "#__SLOPCHOP_MANIFEST__#",
    ];

    for req in required {
        assert!(prompt.contains(req), "Prompt missing: {req}");
    }
    Ok(())
}

#[test]
fn test_reminder_is_concise() -> Result<()> {
    let config = Config::default();
    let generator = slopchop_core::prompt::PromptGenerator::new(config.rules);
    let reminder = generator.generate_reminder()?;

    assert!(reminder.contains("SLOPCHOP CONSTRAINTS"));
    assert!(reminder.contains("#__SLOPCHOP_FILE__#"));
    Ok(())
}

#[test]
fn test_pack_skeleton_integration() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let file_path = root.join("test.rs");
    fs::write(&file_path, "fn main() { body }")?;

    let config = Config::default();
    let opts = PackOptions {
        skeleton: true,
        ..Default::default()
    };

    let content = pack::generate_content(&[file_path], &opts, &config)?;
    assert!(content.contains("fn main() { ... }"));
    Ok(())
}

#[test]
fn test_smart_context_focus_mode() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let target = root.join("target.rs");
    let other = root.join("other.rs");

    fs::write(&target, "fn target() { body }")?;
    fs::write(&other, "fn other() { body }")?;

    let config = Config::default();
    let opts = PackOptions {
        target: Some(target.clone()),
        ..Default::default()
    };

    let content = pack::generate_content(&[target, other], &opts, &config)?;
    assert!(content.contains("fn target() { body }"));
    assert!(content.contains("fn other() { ... }"));
    Ok(())
}