use crate::roadmap::types::{Roadmap, Section, TaskStatus};
use std::fmt::Write;

impl Roadmap {
    /// Generate a compact state representation for AI context
    #[must_use]
    pub fn compact_state(&self) -> String {
        let mut out = String::new();
        let _ = write!(out, "# {}\n\n", self.title);

        for section in &self.sections {
            if section.tasks.is_empty() && section.subsections.is_empty() {
                continue;
            }
            out.push_str(&format_section_compact(section, 0));
        }

        out
    }
}

fn format_section_compact(section: &Section, depth: usize) -> String {
    let mut out = String::new();
    let indent = "  ".repeat(depth);

    let total = section.tasks.len();
    let complete = section
        .tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .count();

    if total > 0 {
        let _ = writeln!(out, "{indent}{} [{complete}/{total}]", section.heading);
        for task in &section.tasks {
            let marker = match task.status {
                TaskStatus::Complete => "✓",
                TaskStatus::Pending => "○",
            };
            let _ = writeln!(out, "{indent}  {marker} {} ({})", task.text, task.path);
        }
    } else if !section.subsections.is_empty() {
        let _ = writeln!(out, "{indent}{}", section.heading);
    }

    for sub in &section.subsections {
        out.push_str(&format_section_compact(sub, depth + 1));
    }

    out
}
