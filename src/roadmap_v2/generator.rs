// src/roadmap_v2/generator.rs
use super::types::{TaskStore, SectionStatus, TaskStatus};

impl TaskStore {
    /// Generate ROADMAP.md content from the store
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        
        out.push_str(&format!("# {}\n\n", self.meta.title));
        
        if !self.meta.description.is_empty() {
            out.push_str(&self.meta.description);
            out.push_str("\n\n");
        }

        out.push_str("---\n\n");

        for section in &self.sections {
            self.write_section(&mut out, section);
        }

        out
    }

    fn write_section(&self, out: &mut String, section: &super::types::Section) {
        let status_marker = match section.status {
            SectionStatus::Complete => " ?",
            SectionStatus::Current => " ?? CURRENT",
            SectionStatus::Pending => "",
        };

        out.push_str(&format!("## {}{}\n\n", section.title, status_marker));

        let section_tasks: Vec<_> = self.tasks.iter()
            .filter(|t| t.section == section.id)
            .collect();

        let groups = self.collect_groups(&section_tasks);

        for group in groups {
            if let Some(name) = &group {
                out.push_str(&format!("### {name}\n"));
            }

            for task in section_tasks.iter().filter(|t| t.group == *group) {
                self.write_task(out, task);
            }

            out.push('\n');
        }

        out.push_str("---\n\n");
    }

    fn collect_groups(&self, tasks: &[&super::types::Task]) -> Vec<Option<String>> {
        let mut groups: Vec<Option<String>> = Vec::new();
        
        for task in tasks {
            if !groups.contains(&task.group) {
                groups.push(task.group.clone());
            }
        }
        
        groups
    }

    fn write_task(&self, out: &mut String, task: &super::types::Task) {
        let checkbox = match task.status {
            TaskStatus::Done => "[x]",
            TaskStatus::Pending => "[ ]",
            TaskStatus::NoTest => "[x]",
        };

        let test_anchor = match (&task.test, &task.status) {
            (Some(test), _) => format!(" <!-- test: {test} -->"),
            (None, TaskStatus::NoTest) => " [no-test]".to_string(),
            (None, _) => String::new(),
        };

        out.push_str(&format!("- {checkbox} **{}**{test_anchor}\n", task.text));
    }
}