use crate::roadmap::parser::slugify;
use crate::roadmap::types::{
    ApplyResult, Command, CommandBatch, MovePosition, Roadmap, TaskStatus,
};

pub fn apply_commands(roadmap: &mut Roadmap, batch: &CommandBatch) -> Vec<ApplyResult> {
    batch
        .commands
        .iter()
        .map(|cmd| run_cmd(roadmap, cmd))
        .collect()
}

fn run_cmd(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Check { path } => set_status(roadmap, path, TaskStatus::Complete),
        Command::Uncheck { path } => set_status(roadmap, path, TaskStatus::Pending),
        Command::Delete { path } => run_delete(roadmap, path),
        Command::Add { .. } | Command::Update { .. } | Command::Note { .. } => {
            run_content_cmd(roadmap, cmd)
        }
        _ => ApplyResult::Error("Command not supported".into()),
    }
}

fn run_content_cmd(roadmap: &mut Roadmap, cmd: &Command) -> ApplyResult {
    match cmd {
        Command::Add {
            parent,
            text,
            after,
        } => run_add(roadmap, parent, text, after.as_deref()),
        Command::Update { path, text } => run_update(roadmap, path, text),
        Command::Note { path, note } => run_note(roadmap, path, note),
        _ => unreachable!(),
    }
}

fn set_status(roadmap: &mut Roadmap, path: &str, status: TaskStatus) -> ApplyResult {
    if let Some(task) = roadmap.find_task(path) {
        if update_line_status(roadmap, task.line, status) {
            return ok_res(status, path);
        }
    }

    if let Some(idx) = find_line_idx(roadmap, path) {
        if update_line_status(roadmap, idx, status) {
            return ok_res(status, path);
        }
    }
    ApplyResult::NotFound(path.into())
}

fn run_add(roadmap: &mut Roadmap, parent: &str, text: &str, after: Option<&str>) -> ApplyResult {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    if let Some(idx) = scan_insertion_point(&lines, parent, after) {
        insert_raw(roadmap, idx, format!("- [ ] **{text}**"));
        ApplyResult::Success(format!("Added: {text}"))
    } else {
        ApplyResult::NotFound(format!("Section: {parent}"))
    }
}

fn run_delete(roadmap: &mut Roadmap, path: &str) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        remove_raw(roadmap, idx);
        ApplyResult::Success(format!("Deleted: {path}"))
    } else {
        ApplyResult::NotFound(path.into())
    }
}

fn run_update(roadmap: &mut Roadmap, path: &str, text: &str) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        let line = roadmap.raw.lines().nth(idx).unwrap_or("");
        let indent = &line[..line.len() - line.trim_start().len()];
        let mark = if line.to_uppercase().contains("[X]") {
            "[x]"
        } else {
            "[ ]"
        };

        replace_raw(roadmap, idx, format!("{indent}- {mark} **{text}**"));
        ApplyResult::Success(format!("Updated: {path}"))
    } else {
        ApplyResult::NotFound(path.into())
    }
}

fn run_note(roadmap: &mut Roadmap, path: &str, note: &str) -> ApplyResult {
    if let Some(idx) = find_line_idx(roadmap, path) {
        let line = roadmap.raw.lines().nth(idx).unwrap_or("");
        let len = line.len() - line.trim_start().len();
        let prefix = " ".repeat(len + 2);

        insert_raw(roadmap, idx + 1, format!("{prefix}*{note}*"));
        ApplyResult::Success(format!("Added note: {path}"))
    } else {
        ApplyResult::NotFound(path.into())
    }
}

#[allow(dead_code)] // Stub for future use
fn apply_move(_: &mut Roadmap, path: &str, _: &MovePosition) -> ApplyResult {
    ApplyResult::Error(format!("MOVE not implemented: {path}"))
}

#[allow(dead_code)] // Stub for future use
fn apply_section_replace(_: &mut Roadmap, id: &str, _: &str) -> ApplyResult {
    ApplyResult::Error(format!("SECTION not implemented: {id}"))
}

// --- Helpers ---

fn scan_insertion_point(lines: &[&str], parent: &str, after: Option<&str>) -> Option<usize> {
    let p_slug = slugify(parent);
    let mut in_sec = false;
    let mut last_task = None;
    let mut sec_start = None;

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("##") {
            if check_section_entry(line, &p_slug) {
                in_sec = true;
                sec_start = Some(i + 1);
            } else if in_sec {
                break;
            }
            continue;
        }

        if in_sec && is_task(line) {
            last_task = Some(i);
            if check_after_match(line, after) {
                return Some(i + 1);
            }
        }
    }
    last_task.map(|i| i + 1).or(sec_start)
}

fn check_section_entry(line: &str, parent_slug: &str) -> bool {
    slugify(line).contains(parent_slug)
}

fn check_after_match(line: &str, after: Option<&str>) -> bool {
    after.is_some_and(|tgt| slugify(line).contains(&slugify(tgt)))
}

fn find_line_idx(roadmap: &Roadmap, path: &str) -> Option<usize> {
    let search = path.split('/').next_back().unwrap_or(path);
    let s_slug = slugify(search);
    roadmap
        .raw
        .lines()
        .position(|l| is_task(l) && slugify(l).contains(&s_slug))
}

fn update_line_status(roadmap: &mut Roadmap, idx: usize, status: TaskStatus) -> bool {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    if idx >= lines.len() {
        return false;
    }

    let line = lines[idx];
    let new = match status {
        TaskStatus::Complete => line.replace("- [ ]", "- [x]"),
        TaskStatus::Pending => line.replace("- [x]", "- [ ]").replace("- [X]", "- [ ]"),
    };
    replace_raw(roadmap, idx, new);
    true
}

fn replace_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, |lines| {
        if idx < lines.len() {
            lines[idx] = line;
        }
    });
}

fn insert_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, |lines| {
        if idx <= lines.len() {
            lines.insert(idx, line);
        }
    });
}

fn remove_raw(roadmap: &mut Roadmap, idx: usize) {
    modify_lines(roadmap, |lines| {
        if idx < lines.len() {
            lines.remove(idx);
        }
    });
}

fn modify_lines<F>(roadmap: &mut Roadmap, f: F)
where
    F: FnOnce(&mut Vec<String>),
{
    let mut lines: Vec<String> = roadmap.raw.lines().map(ToString::to_string).collect();
    f(&mut lines);
    roadmap.raw = lines.join("\n");
}

fn is_task(line: &str) -> bool {
    line.trim().starts_with("- [")
}

fn ok_res(status: TaskStatus, path: &str) -> ApplyResult {
    let act = if status == TaskStatus::Complete {
        "Checked"
    } else {
        "Unchecked"
    };
    ApplyResult::Success(format!("{act}: {path}"))
}
