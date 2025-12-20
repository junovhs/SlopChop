use crate::config::Config;
use crate::tui::config::state::ConfigApp;
use crate::types::ScanReport;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Config,
    Logs,
}

pub struct DashboardApp<'a> {
    pub config: &'a mut Config,
    pub active_tab: Tab,
    pub scan_report: Option<ScanReport>,
    pub config_editor: ConfigApp,
    pub last_scan: Option<Instant>,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub pending_payload: Option<String>,
}

impl<'a> DashboardApp<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        Self {
            config,
            active_tab: Tab::Dashboard,
            scan_report: None,
            config_editor: ConfigApp::new(),
            last_scan: None,
            logs: vec!["Dashboard initialized".to_string()],
            should_quit: false,
            pending_payload: None,
        }
    }

    pub fn log(&mut self, message: &str) {
        self.logs.push(message.to_string());
        if self.logs.len() > 100 { self.logs.remove(0); }
    }

    pub fn cycle_tab(&mut self, reverse: bool) {
        let tabs = [Tab::Dashboard, Tab::Config, Tab::Logs];
        let current = tabs.iter().position(|&t| t == self.active_tab).unwrap_or(0);
        let next = if reverse { (current + 2) % 3 } else { (current + 1) % 3 };
        self.active_tab = tabs[next];
    }
}