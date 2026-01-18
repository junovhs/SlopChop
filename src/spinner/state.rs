// src/spinner/state.rs
use std::collections::VecDeque;
use std::time::Instant;

pub const ATOMIC_LINES: usize = 3;

pub struct HudState {
    title: String,
    micro_status: String,
    atomic_buffer: VecDeque<String>,
    start_time: Instant,
    final_success: Option<bool>,
    progress: Option<(usize, usize)>,
}

impl HudState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            micro_status: "Starting...".to_string(),
            atomic_buffer: VecDeque::with_capacity(ATOMIC_LINES),
            start_time: Instant::now(),
            final_success: None,
            progress: None,
        }
    }

    pub fn push_log(&mut self, line: &str) {
        if self.atomic_buffer.len() >= ATOMIC_LINES {
            self.atomic_buffer.pop_front();
        }
        self.atomic_buffer.push_back(line.to_string());

        if self.progress.is_none() {
            if let Some(status) = extract_micro_status(line) {
                self.micro_status = status;
            }
        }
    }

    pub fn update_progress(&mut self, current: usize, total: usize, message: String) {
        self.progress = Some((current, total));
        self.micro_status.clone_from(&message);
        
        if self.atomic_buffer.len() >= ATOMIC_LINES {
            self.atomic_buffer.pop_front();
        }
        self.atomic_buffer.push_back(message);
    }

    pub fn set_finished(&mut self, success: bool) {
        self.final_success = Some(success);
    }

    pub fn completion_info(&self) -> (bool, &str, Instant) {
        (
            self.final_success.unwrap_or(false),
            &self.title,
            self.start_time,
        )
    }

    pub fn snapshot(&self) -> (String, String, VecDeque<String>, Instant, Option<(usize, usize)>) {
        (
            self.title.clone(),
            self.micro_status.clone(),
            self.atomic_buffer.clone(),
            self.start_time,
            self.progress,
        )
    }
}

fn extract_micro_status(line: &str) -> Option<String> {
    let trimmed = line.trim();
    
    if trimmed.starts_with("Compiling") 
        || trimmed.starts_with("Checking")
        || trimmed.starts_with("Downloading")
        || trimmed.starts_with("Finished")
        || trimmed.starts_with("Building")
        || trimmed.starts_with("Bundling") {
        return Some(trimmed.to_string());
    }

    if trimmed.starts_with("Running") || trimmed.starts_with("Executing") {
        return Some(trimmed.to_string());
    }

    if trimmed.contains("| LAW:") {
        if let Some(path) = trimmed.split('|').next() {
            return Some(format!("Scanning {}", path.replace("FILE:", "").trim()));
        }
    }

    None
}