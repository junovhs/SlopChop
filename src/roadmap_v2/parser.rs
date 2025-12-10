// src/roadmap_v2/parser.rs
use super::types::{RoadmapCommand, Task, TaskStatus, TaskUpdate};
use anyhow::{anyhow, bail, Result};

/// Parses roadmap commands from the ===ROADMAP=== block(s) in AI output.
///
/// # Errors
/// Returns error if command syntax is invalid.
pub fn parse_commands(input: &str) -> Result<Vec<RoadmapCommand>> {
    let blocks = extract_roadmap_blocks(input);
    if blocks.is_empty() {
        return Ok(vec![]);
    }

    let mut commands = Vec::new();
    for block in blocks {
        let mut block_cmds = parse_block_content(&block)?;
        commands.append(&mut block_cmds);
    }

    Ok(commands)
}

fn extract_roadmap_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut state = BlockState::default();

    for line in input.lines() {
        process_line_for_blocks(line, &mut state, &mut blocks);
    }

    blocks
}

#[derive(Default)]
struct BlockState {
    capturing: bool,
    current_block: String,
}

fn process_line_for_blocks(line: &str, state: &mut BlockState, blocks: &mut Vec<String>) {
    let marker = "===ROADMAP===";
    let trimmed = line.trim();

    if trimmed == marker {
        if state.capturing {
            if !state.current_block.trim().is_empty() {
                blocks.push(state.current_block.clone());
            }
            state.current_block.clear();
            state.capturing = false;
        } else {
            state.capturing = true;
        }
    } else if state.capturing {
        state.current_block.push_str(line);
        state.current_block.push('\n');
    }
}

fn parse_block_content(block: &str) -> Result<Vec<RoadmapCommand>> {
    let mut commands = Vec::new();
    let mut current_block = String::new();

    for line in block.lines() {
        let trimmed = clean_line(line);
        if trimmed.is_empty() {
            continue;
        }

        if is_command_keyword(trimmed) {
            if !current_block.is_empty() {
                commands.push(parse_single_command(&current_block)?);
            }
            current_block = trimmed.to_string();
        } else {
            if !current_block.is_empty() {
                current_block.push('\n');
            }
            current_block.push_str(line);
        }
    }

    if !current_block.is_empty() {
        commands.push(parse_single_command(&current_block)?);
    }

    Ok(commands)
}

fn clean_line(line: &str) -> &str {
    line.split('#').next().unwrap_or("").trim()
}

fn is_command_keyword(line: &str) -> bool {
    let upper = clean_line(line).to_uppercase();
    matches!(
        upper.as_str(),
        "CHECK" | "UNCHECK" | "ADD" | "UPDATE" | "DELETE"
    )
}

fn parse_single_command(block: &str) -> Result<RoadmapCommand> {
    let lines: Vec<&str> = block.lines().collect();
    let keyword_line = lines.first().copied().unwrap_or_default();
    let keyword = clean_line(keyword_line).to_uppercase();

    match keyword.as_str() {
        "CHECK" => parse_check(&lines),
        "UNCHECK" => parse_uncheck(&lines),
        "ADD" => parse_add(&lines),
        "UPDATE" => parse_update(&lines),
        "DELETE" => parse_delete(&lines),
        "" => bail!("Empty command block"),
        other => bail!("Unknown roadmap command: {other}"),
    }
}

fn parse_check(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Check { id })
}

fn parse_uncheck(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Uncheck { id })
}

fn parse_delete(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Delete { id })
}

fn parse_add(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    let text = require_field(lines, "text")?;
    let section = require_field(lines, "section")?;

    Ok(RoadmapCommand::Add(Task {
        id,
        text,
        status: TaskStatus::Pending,
        section,
        test: get_field(lines, "test"),
        group: get_field(lines, "group"),
        order: 0,
    }))
}

fn parse_update(lines: &[&str]) -> Result<RoadmapCommand> {
    let id = require_field(lines, "id")?;
    let fields = TaskUpdate {
        text: get_field(lines, "text"),
        test: get_field(lines, "test"),
        section: get_field(lines, "section"),
        group: get_field(lines, "group"),
    };
    Ok(RoadmapCommand::Update { id, fields })
}

fn require_field(lines: &[&str], key: &str) -> Result<String> {
    let value = get_field(lines, key).ok_or_else(|| anyhow!("Missing required field: {key}"))?;
    if value.trim().is_empty() {
        bail!("Field '{key}' cannot be empty");
    }
    Ok(value)
}

fn get_field(lines: &[&str], key: &str) -> Option<String> {
    for line in lines {
        let trimmed = clean_line(line);
        let Some((k, v)) = trimmed.split_once('=') else {
            continue;
        };
        if k.trim() != key {
            continue;
        }
        let value = v.trim();
        if value.is_empty() {
            return None;
        }
        return Some(value.to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_block(content: &str) -> String {
        format!("\n{}\n{}\n{}\n", "===ROADMAP===", content, "===ROADMAP===")
    }

    #[test]
    fn test_parse_check() -> Result<()> {
        let input = make_block("CHECK\nid = my-task");
        let cmds = parse_commands(&input)?;
        assert!(matches!(cmds[0], RoadmapCommand::Check { ref id } if id == "my-task"));
        Ok(())
    }

    #[test]
    fn test_parse_add() -> Result<()> {
        let input = make_block("ADD\nid = t\ntext = Do X\nsection = v1");
        let cmds = parse_commands(&input)?;
        assert!(matches!(cmds[0], RoadmapCommand::Add(_)));
        Ok(())
    }

    #[test]
    fn test_multiple_commands() -> Result<()> {
        let input = make_block("CHECK\nid = task-1\nCHECK\nid = task-2");
        let cmds = parse_commands(&input)?;
        assert_eq!(cmds.len(), 2);
        Ok(())
    }

    #[test]
    fn test_ignores_inline_markers() -> Result<()> {
        let input = "Fix the ===ROADMAP=== issue.";
        let cmds = parse_commands(input)?;
        assert!(cmds.is_empty());
        Ok(())
    }

    #[test]
    fn test_rejects_empty_id() {
        let input = make_block("CHECK\nid = ");
        assert!(parse_commands(&input).is_err());
    }

    #[test]
    fn test_rejects_empty_text() {
        let input = make_block("ADD\nid = t\ntext = \nsection = v1");
        assert!(parse_commands(&input).is_err());
    }
}