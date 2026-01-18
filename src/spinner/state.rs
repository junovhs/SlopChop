// src/spinner/state.rs
use std::collections::VecDeque;
use std::time::Instant;

pub const ATOMIC_LINES: usize = 5;

// Encapsulate fields to fix AHF violation
pub struct HudState {
    pipeline_step: Option<(usize, usize)>,
    pipeline_name: String,
    micro_status: String,
    micro_progress: Option<(usize, usize)>,
    atomic_buffer: VecDeque<String>,
    start_time: Instant,
    final_success: Option<bool>,
}

// DTO for rendering
pub struct HudSnapshot {
    pub pipeline_step: Option<(usize, usize)>,
    pub pipeline_name: String,
    pub micro_status: String,
    pub micro_progress: Option<(usize, usize)>,
    pub atomic_buffer: VecDeque<String>,
    pub start_time: Instant,
}

impl HudState {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            pipeline_step: None,
            pipeline_name: title.into(),
            micro_status: "Initializing...".to_string(),
            micro_progress: None,
            atomic_buffer: VecDeque::with_capacity(ATOMIC_LINES),
            start_time: Instant::now(),
            final_success: None,
        }
    }

    pub fn set_macro_step(&mut self, current: usize, total: usize, name: String) {
        self.pipeline_step = Some((current, total));
        self.pipeline_name = name;
        self.micro_progress = None;
        self.micro_status = "Starting...".to_string();
    }

    pub fn set_micro_status(&mut self, status: String) {
        self.micro_status = status;
        self.micro_progress = None;
    }

    pub fn step_micro_progress(&mut self, current: usize, total: usize, status: String) {
        self.micro_progress = Some((current, total));
        self.micro_status = status;
    }

    pub fn push_log(&mut self, line: &str) {
        if self.atomic_buffer.len() >= ATOMIC_LINES {
            self.atomic_buffer.pop_front();
        }
        self.atomic_buffer.push_back(line.to_string());

        if self.micro_progress.is_none() {
            if let Some(extracted) = extract_micro_status(line) {
                self.micro_status = extracted;
            }
        }
    }

    pub fn set_finished(&mut self, success: bool) {
        self.final_success = Some(success);
    }

    pub fn completion_info(&self) -> (bool, &str, Instant) {
        (
            self.final_success.unwrap_or(false),
            &self.pipeline_name,
            self.start_time,
        )
    }

    pub fn snapshot(&self) -> HudSnapshot {
        HudSnapshot {
            pipeline_step: self.pipeline_step,
            pipeline_name: self.pipeline_name.clone(),
            micro_status: self.micro_status.clone(),
            micro_progress: self.micro_progress,
            atomic_buffer: self.atomic_buffer.clone(),
            start_time: self.start_time,
        }
    }
}

fn extract_micro_status(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() { return None; }

    if trimmed.starts_with("Compiling") || trimmed.starts_with("Checking") 
       || trimmed.starts_with("Finished") || trimmed.starts_with("Downloading") {
        return Some(trimmed.to_string());
    }
    
    if trimmed.starts_with("Scanning") || trimmed.starts_with("Analyzing") {
        return Some(trimmed.to_string());
    }

    None
}