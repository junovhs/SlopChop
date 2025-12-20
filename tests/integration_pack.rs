use anyhow::Result;
use slopchop_core::config::Config;
use slopchop_core::pack::{self, PackOptions};
use std::fs;
use tempfile::tempdir;

const SIGIL: &str = "XSC7XSC";

#[test]
fn test_nabla_delimiters_are_unique() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let file_path = root.join("test.rs");
    fs::write(&file_path, "fn main() {}")?;

    let config = Config::default();
    let opts = PackOptions { stdout: true, ..Default::default() };

    let content = pack::generate_content(&[file_path], &opts, &config)?;

    assert!(content.contains(&format!("{SIGIL} FILE {SIGIL}")));
    assert!(content.contains(&format!("{SIGIL} END {SIGIL}")));
    Ok(())
}

#[test]
fn test_nabla_format_structure() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let file_path = root.join("src/main.rs");
    fs::create_dir_all(root.join("src"))?;
    fs::write(&file_path, "code")?;

    let config = Config::default();
    let opts = PackOptions::default();

    let content = pack::generate_content(std::slice::from_ref(&file_path), &opts, &config)?;

    let p_str = file_path.to_string_lossy().replace('\\', "/");
    let header = format!("{SIGIL} FILE {SIGIL} {p_str}");

    assert!(content.contains(&header));
    assert!(content.contains("code"));
    assert!(content.contains(&format!("{SIGIL} END {SIGIL}")));
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
        "Files MUST be < 2000 tokens",
        "Complexity <= 8",
        "OUTPUT FORMAT (MANDATORY)",
        "XSC7XSC FILE XSC7XSC",
        "XSC7XSC MANIFEST XSC7XSC",
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
    assert!(reminder.contains(SIGIL));
    Ok(())
}

#[test]
fn test_pack_skeleton_integration() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let file_path = root.join("test.rs");
    // Ensure sufficient context for tree-sitter
    fs::write(&file_path, "fn main() { body }")?;

    let config = Config::default();
    let opts = PackOptions { skeleton: true, ..Default::default() };

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
    let opts = PackOptions { target: Some(target.clone()), ..Default::default() };

    let content = pack::generate_content(&[target, other], &opts, &config)?;
    assert!(content.contains("fn target() { body }"));
    assert!(content.contains("fn other() { ... }"));
    Ok(())
}