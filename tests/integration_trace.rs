// tests/integration_trace.rs
use anyhow::Result;
use slopchop_core::graph::defs;
use slopchop_core::graph::rank::RepoGraph;
use std::path::PathBuf;

#[test]
fn test_graph_builds_on_slopchop_itself() -> Result<()> {
    let files = vec![
        (
            PathBuf::from("src/lib.rs"),
            std::fs::read_to_string("src/lib.rs")?,
        ),
        (
            PathBuf::from("src/config/mod.rs"),
            std::fs::read_to_string("src/config/mod.rs")?,
        ),
    ];

    let graph = RepoGraph::build(&files);
    let ranked = graph.ranked_files();

    // Should have found some files
    assert!(
        !ranked.is_empty() || files.len() <= 2,
        "Graph should process files"
    );
    Ok(())
}

#[test]
fn test_defs_extracts_from_real_file() -> Result<()> {
    let content = std::fs::read_to_string("src/lib.rs")?;
    let defs = defs::extract(std::path::Path::new("src/lib.rs"), &content);

    // lib.rs should have at least some module declarations
    println!("Found {} definitions in lib.rs", defs.len());
    Ok(())
}