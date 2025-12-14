// src/tui/dashboard/state.rs
use crate::config::Config;
use crate::roadmap_v2::types::{Task, TaskStatus, TaskStore};
use crate::tui::config::state::ConfigApp;
use crate::types::ScanReport;
use ratatui::widgets::ListState;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Roadmap,
    Config,
    Logs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatusFilter {
    All,
    Pending,
    Done,
}

impl TaskStatusFilter {
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::All => Self::Pending,
            Self::Pending => Self::Done,
            Self::Done => Self::All,
        }
    }
}

pub struct DashboardApp<'a> {
    pub config: &'a mut Config,
    pub active_tab: Tab,
    pub scan_report: Option<ScanReport>,
    pub roadmap: Option<TaskStore>,
    pub config_editor: ConfigApp,
    pub last_scan: Option<Instant>,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub scroll: u16,
    // Roadmap state
    pub roadmap_list_state: ListState,
    pub roadmap_filter: TaskStatusFilter,
    /// Pending payload from clipboard watcher, waiting for user confirmation
    pub pending_payload: Option<String>,
}

impl<'a> DashboardApp<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        Self {
            config,
            active_tab: Tab::Dashboard,
            scan_report: None,
            roadmap: None,
            config_editor: ConfigApp::new(),
            last_scan: None,
            logs: vec!["SlopChop Dashboard initialized".to_string()],
            should_quit: false,
            scroll: 0,
            roadmap_list_state: ListState::default(),
            roadmap_filter: TaskStatusFilter::All,
            pending_payload: None,
        }
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = chrono_lite_timestamp();
        self.logs.push(format!("[{timestamp}] {message}"));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn on_tick(&mut self) {
        if self.active_tab == Tab::Dashboard {
            if let Some(last) = self.last_scan {
                if last.elapsed() > Duration::from_secs(30) {
                    // Could auto-refresh here, but let's keep it manual for now
                }
            }
        }
        self.config_editor.check_message_expiry();
    }

    pub fn trigger_scan(&mut self) {
        self.last_scan = Some(Instant::now());
    }

    pub fn cycle_tab(&mut self, reverse: bool) {
        let tabs = [Tab::Dashboard, Tab::Roadmap, Tab::Config, Tab::Logs];
        let current = tabs
            .iter()
            .position(|&t| t == self.active_tab)
            .unwrap_or(0);
        let len = tabs.len();

        let next = if reverse {
            (current + len - 1) % len
        } else {
            (current + 1) % len
        };

        self.active_tab = tabs[next];
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    #[must_use]
    pub fn has_pending_payload(&self) -> bool {
        self.pending_payload.is_some()
    }

    // --- Roadmap Helpers ---

    #[must_use]
    pub fn get_filtered_tasks(&self) -> Vec<&Task> {
        let Some(store) = &self.roadmap else {
            return vec![];
        };

        store
            .tasks
            .iter()
            .filter(|t| match self.roadmap_filter {
                TaskStatusFilter::All => true,
                TaskStatusFilter::Pending => t.status == TaskStatus::Pending,
                TaskStatusFilter::Done => matches!(t.status, TaskStatus::Done | TaskStatus::NoTest),
            })
            .collect()
    }

    pub fn roadmap_next(&mut self) {
        let count = self.get_filtered_tasks().len();
        if count == 0 {
            return;
        }

        let i = match self.roadmap_list_state.selected() {
            Some(i) => (i + 1) % count,
            None => 0,
        };
        self.roadmap_list_state.select(Some(i));
    }

    pub fn roadmap_previous(&mut self) {
        let count = self.get_filtered_tasks().len();
        if count == 0 {
            return;
        }

        let i = match self.roadmap_list_state.selected() {
            Some(i) => (i + count - 1) % count,
            None => count - 1,
        };
        self.roadmap_list_state.select(Some(i));
    }

    pub fn roadmap_unselect(&mut self) {
        self.roadmap_list_state.select(None);
    }

    pub fn toggle_roadmap_task(&mut self) {
        let filtered = self.get_filtered_tasks();
        let Some(selected_idx) = self.roadmap_list_state.selected() else {
            return;
        };
        let Some(task) = filtered.get(selected_idx) else {
            return;
        };
        let task_id = task.id.clone();

        let mut success = false;
        let mut error_msg = None;

        if let Some(store) = &mut self.roadmap {
            if let Some(t) = store.tasks.iter_mut().find(|t| t.id == task_id) {
                t.status = match t.status {
                    TaskStatus::Pending => TaskStatus::Done,
                    TaskStatus::Done | TaskStatus::NoTest => TaskStatus::Pending,
                };
                success = true;

                if let Err(e) = store.save(None) {
                    error_msg = Some(format!("Failed to save roadmap: {e}"));
                }
            }
        }

        if success {
            self.log(&format!("Toggled task: {task_id}"));
        }
        if let Some(msg) = error_msg {
            self.log(&msg);
        }
    }
}

/// Simple timestamp without pulling in chrono crate
fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Just show HH:MM:SS (approximate, good enough for logs)
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{hours:02}:{mins:02}:{secs:02}")
}