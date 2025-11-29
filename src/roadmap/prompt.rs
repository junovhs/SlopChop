use crate::roadmap::types::{Roadmap, Section, TaskStatus};
use std::fmt::Write;

/// Options for prompt generation
#[derive(Debug, Clone, Default)]
pub struct PromptOptions {
    pub full: bool,
    pub examples: bool,
    pub project_name: Option<String>,
}

/// Generate the teaching prompt for AI
#[must_use]
pub fn generate_prompt(roadmap: &Roadmap, options: &PromptOptions) -> String {
    let project_name = options
        .project_name
        .clone()
        .unwrap_or_else(|| roadmap.title.clone());

    let mut prompt = String::new();

    let _ = writeln!(prompt, "# Roadmap Commands for: {project_name}\n");

    let stats = roadmap.stats();
    let pct = if stats.total > 0 {
        #[allow(clippy::cast_precision_loss)]
        {
            (stats.complete as f64 / stats.total as f64) * 100.0
        }
    } else {
        0.0
    };

    let _ = writeln!(
        prompt,
        "Progress: {}/{} tasks complete ({:.0}%)\n",
        stats.complete, stats.total, pct
    );

    prompt.push_str("## Commands\n\n");
    prompt.push_str("Wrap commands in `===ROADMAP===` and `===END===` markers.\n\n");
    prompt.push_str("```\n");
    prompt.push_str("CHECK <path>              # Mark task complete\n");
    prompt.push_str("UNCHECK <path>            # Mark task incomplete\n");
    prompt.push_str("ADD <section> \"<text>\"    # Add new task to section\n");
    prompt.push_str("ADD <section> \"<text>\" AFTER <task>  # Add after specific task\n");
    prompt.push_str("DELETE <path>             # Remove task\n");
    prompt.push_str("UPDATE <path> \"<text>\"    # Change task description\n");
    prompt.push_str("NOTE <path> \"<text>\"      # Add note under task\n");
    prompt.push_str("```\n\n");

    prompt.push_str("## Task Paths\n\n");
    prompt.push_str("Paths are: `section-slug/task-slug`\n");
    prompt.push_str("Example: `v0-5-0-bulletproof-apply/truncation-detection`\n\n");
    prompt.push_str("You can use partial matches - just the task slug often works.\n\n");

    if options.examples {
        prompt.push_str("## Examples\n\n");
        prompt.push_str("```\n");
        prompt.push_str("===ROADMAP===\n");
        prompt.push_str("CHECK truncation-detection\n");
        prompt.push_str("ADD v0-5-0 \"Improve error messages\" AFTER truncation-detection\n");
        prompt.push_str("NOTE path-safety \"Implemented using std::path::Path\"\n");
        prompt.push_str("===END===\n");
        prompt.push_str("```\n\n");
    }

    prompt.push_str("---\n\n");
    prompt.push_str("## Current Roadmap State\n\n");

    if options.full {
        prompt.push_str("```markdown\n");
        prompt.push_str(&roadmap.raw);
        prompt.push_str("\n```\n");
    } else {
        prompt.push_str(&generate_compact_state(roadmap));
    }

    prompt
}

fn generate_compact_state(roadmap: &Roadmap) -> String {
    let mut out = String::new();
    for section in &roadmap.sections {
        if section.tasks.is_empty() && section.subsections.is_empty() {
            continue;
        }
        out.push_str(&format_section_tree(section, 0));
    }
    out
}

fn format_section_tree(section: &Section, depth: usize) -> String {
    let mut out = String::new();
    let indent = "  ".repeat(depth);
    let (complete, total) = count_tasks_recursive(section);

    if total > 0 {
        let progress = format!("[{complete}/{total}]");
        let status_icon = if complete == total {
            "✓"
        } else if complete > 0 {
            "◐"
        } else {
            "○"
        };
        let _ = writeln!(out, "{indent}{status_icon} {} {progress}", section.heading);

        for task in &section.tasks {
            let icon = match task.status {
                TaskStatus::Complete => "  ✓",
                TaskStatus::Pending => "  ○",
            };
            let _ = writeln!(out, "{indent}{icon}  {} (id: {})", task.text, task.id);
        }
    }

    for sub in &section.subsections {
        out.push_str(&format_section_tree(sub, depth + 1));
    }
    out
}

fn count_tasks_recursive(section: &Section) -> (usize, usize) {
    let mut complete = 0;
    let mut total = 0;

    for task in &section.tasks {
        total += 1;
        if task.status == TaskStatus::Complete {
            complete += 1;
        }
    }

    for sub in &section.subsections {
        let (c, t) = count_tasks_recursive(sub);
        complete += c;
        total += t;
    }

    (complete, total)
}
