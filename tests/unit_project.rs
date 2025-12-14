// tests/unit_project.rs
use anyhow::Result;
use slopchop_core::project::ProjectType;
use std::fs::File;
use tempfile::TempDir;

#[test]
fn test_detection_cases() -> Result<()> {
    let cases = vec![
        (Some("Cargo.toml"), ProjectType::Rust),
        (Some("package.json"), ProjectType::Node),
        (Some("requirements.txt"), ProjectType::Python),
        (Some("go.mod"), ProjectType::Go),
        (None, ProjectType::Unknown),
    ];

    for (file, expected) in cases {
        let temp = TempDir::new()?;
        if let Some(f) = file {
            File::create(temp.path().join(f))?;
        }
        assert_eq!(
            ProjectType::detect_in(temp.path()),
            expected,
            "Failed to detect {expected:?} from file {file:?}"
        );
    }
    Ok(())
}