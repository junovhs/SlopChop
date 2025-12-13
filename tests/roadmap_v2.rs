// tests/roadmap_v2.rs
use anyhow::{Context, Result};
use slopchop_core::roadmap_v2::types::{AddCommand, AfterTarget};
use slopchop_core::roadmap_v2::{parse_commands, RoadmapCommand, Task, TaskStatus, TaskStore};
use tempfile::TempDir;

#[test]
fn test_store_load_save() -> Result<()> {
    let dir = TempDir::new().context("Failed to create temp dir")?;
    let path = dir.path().join("tasks.toml");

    let mut store = TaskStore::default();
    store.meta.title = "Test Roadmap".to_string();
    store.sections.push(slopchop_core::roadmap_v2::types::Section {
        id: "v1".to_string(),
        title: "v1.0.0".to_string(),
        status: slopchop_core::roadmap_v2::types::SectionStatus::Current,
        order: 0,
    });

    store.save(Some(&path)).context("Save failed")?;
    assert!(path.exists());

    let loaded = TaskStore::load(Some(&path)).context("Load failed")?;
    assert_eq!(loaded.meta.title, "Test Roadmap");
    assert_eq!(loaded.sections.len(), 1);
    Ok(())
}

#[test]
fn test_store_apply_check() -> Result<()> {
    let mut store = create_test_store();

    let cmd = RoadmapCommand::Check {
        id: "task-1".to_string(),
    };
    store.apply(cmd).context("Apply failed")?;

    let task = store
        .tasks
        .iter()
        .find(|t| t.id == "task-1")
        .context("Task not found")?;
    assert_eq!(task.status, TaskStatus::Done);
    Ok(())
}

#[test]
fn test_store_apply_uncheck() -> Result<()> {
    let mut store = create_test_store();

    store
        .apply(RoadmapCommand::Check {
            id: "task-1".to_string(),
        })
        .context("Check failed")?;

    store
        .apply(RoadmapCommand::Uncheck {
            id: "task-1".to_string(),
        })
        .context("Uncheck failed")?;

    let task = store
        .tasks
        .iter()
        .find(|t| t.id == "task-1")
        .context("Task not found")?;
    assert_eq!(task.status, TaskStatus::Pending);
    Ok(())
}

#[test]
fn test_store_apply_add() -> Result<()> {
    let mut store = create_test_store();

    let new_task = Task {
        id: "new-task".to_string(),
        text: "New Feature".to_string(),
        status: TaskStatus::Pending,
        section: "v1".to_string(),
        group: None,
        test: Some("tests/unit.rs::test_new".to_string()),
        order: 10,
    };

    let add_cmd = AddCommand {
        task: new_task,
        after: AfterTarget::End,
    };

    store
        .apply(RoadmapCommand::Add(add_cmd))
        .context("Add failed")?;

    assert_eq!(store.tasks.len(), 2);
    let added = store
        .tasks
        .iter()
        .find(|t| t.id == "new-task")
        .context("Task not found")?;
    assert_eq!(added.text, "New Feature");
    Ok(())
}

#[test]
fn test_store_apply_delete() -> Result<()> {
    let mut store = create_test_store();

    store
        .apply(RoadmapCommand::Delete {
            id: "task-1".to_string(),
        })
        .context("Delete failed")?;

    assert!(store.tasks.is_empty());
    Ok(())
}

#[test]
fn test_store_apply_update() -> Result<()> {
    let mut store = create_test_store();

    store
        .apply(RoadmapCommand::Update {
            id: "task-1".to_string(),
            fields: slopchop_core::roadmap_v2::types::TaskUpdate {
                text: Some("Updated Text".to_string()),
                test: Some("tests/new.rs::test_fn".to_string()),
                section: None,
                group: None,
            },
        })
        .context("Update failed")?;

    let task = store
        .tasks
        .iter()
        .find(|t| t.id == "task-1")
        .context("Task not found")?;
    assert_eq!(task.text, "Updated Text");
    assert_eq!(task.test, Some("tests/new.rs::test_fn".to_string()));
    Ok(())
}

#[test]
fn test_parse_check_command() -> Result<()> {
    let input = "===ROADMAP===\nCHECK\nid = my-task\n===ROADMAP===";
    let cmds = parse_commands(input).context("Parse failed")?;

    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Check { id } => assert_eq!(id, "my-task"),
        _ => panic!("Expected Check command"),
    }
    Ok(())
}

#[test]
fn test_parse_uncheck_command() -> Result<()> {
    let input = "===ROADMAP===\nUNCHECK\nid = task-abc\n===ROADMAP===";
    let cmds = parse_commands(input).context("Parse failed")?;

    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Uncheck { id } => assert_eq!(id, "task-abc"),
        _ => panic!("Expected Uncheck command"),
    }
    Ok(())
}

#[test]
fn test_parse_add_command() -> Result<()> {
    let input = r"===ROADMAP===
ADD
id = new-feature
text = Support Go Language
section = v0.8.0
group = Lang Support
test = tests/unit.rs::test_go
===ROADMAP===";

    let cmds = parse_commands(input).context("Parse failed")?;

    assert_eq!(cmds.len(), 1);
    match &cmds[0] {
        RoadmapCommand::Add(add_cmd) => {
            assert_eq!(add_cmd.task.id, "new-feature");
            assert_eq!(add_cmd.task.text, "Support Go Language");
            assert_eq!(add_cmd.task.section, "v0.8.0");
            assert_eq!(add_cmd.task.group, Some("Lang Support".to_string()));
            assert_eq!(add_cmd.task.test, Some("tests/unit.rs::test_go".to_string()));
        }
        _ => panic!("Expected Add command"),
    }
    Ok(())
}

#[test]
fn test_parse_multiple_commands() -> Result<()> {
    let input = r"
===ROADMAP===
CHECK
id = task-1
===ROADMAP===

Some text in between

===ROADMAP===
CHECK
id = task-2
===ROADMAP===
";

    let cmds = parse_commands(input).context("Parse failed")?;
    assert_eq!(cmds.len(), 2);
    Ok(())
}

#[test]
fn test_generator_markdown() {
    let store = create_test_store();
    let md = store.to_markdown();

    assert!(md.contains("# Test Roadmap"));
    assert!(md.contains("## v1.0.0"));
    assert!(md.contains("- [ ] **Task One**"));
}

#[test]
fn test_generator_with_done_task() {
    let mut store = create_test_store();
    store.tasks[0].status = TaskStatus::Done;

    let md = store.to_markdown();
    assert!(md.contains("- [x] **Task One**"));
}

#[test]
fn test_generator_with_test_anchor() {
    let mut store = create_test_store();
    store.tasks[0].test = Some("tests/unit.rs::test_fn".to_string());

    let md = store.to_markdown();
    assert!(md.contains("<!-- test: tests/unit.rs::test_fn -->"));
}

fn create_test_store() -> TaskStore {
    use slopchop_core::roadmap_v2::types::{RoadmapMeta, Section, SectionStatus};

    TaskStore {
        meta: RoadmapMeta {
            title: "Test Roadmap".to_string(),
            description: String::new(),
        },
        sections: vec![Section {
            id: "v1".to_string(),
            title: "v1.0.0".to_string(),
            status: SectionStatus::Current,
            order: 0,
        }],
        tasks: vec![Task {
            id: "task-1".to_string(),
            text: "Task One".to_string(),
            status: TaskStatus::Pending,
            section: "v1".to_string(),
            group: None,
            test: None,
            order: 0,
        }],
    }
}