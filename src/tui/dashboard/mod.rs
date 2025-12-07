// src/tui/dashboard/mod.rs
pub mod apply;
pub mod input;
pub mod state;
pub mod ui;

use crate::analysis::RuleEngine;
use crate::config::Config;
use crate::discovery;
use crate::roadmap_v2::types::TaskStore;
use crate::tui::runner;
use crate::tui::watcher::{self, WatcherEvent};
use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use state::DashboardApp;
use std::io;
use std::sync::mpsc;
use std::time::Duration;

/// Runs the TUI dashboard.
///
/// # Errors
/// Returns error if terminal setup fails or during execution.
pub fn run(config: &mut Config) -> Result<()> {
    runner::setup_terminal()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run_app(&mut terminal, config);

    runner::restore_terminal()?;
    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    config: &mut Config,
) -> Result<()> {
    let mut app = DashboardApp::new(config);

    // Initial scan
    run_scan(&mut app);
    load_roadmap(&mut app);

    // Set up watcher channel
    let (tx, rx) = mpsc::channel();
    watcher::spawn_watcher(tx);
    app.log("Clipboard watcher started");

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Check for watcher events (non-blocking)
        if let Ok(event) = rx.try_recv() {
            handle_watcher_event(&mut app, event);
        }

        // Poll for keyboard input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                input::handle_input(key.code, key.modifiers, &mut app, terminal);
            }
        }

        // Periodic tick
        app.on_tick();

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_watcher_event(app: &mut DashboardApp, event: WatcherEvent) {
    match event {
        WatcherEvent::PayloadDetected(content) => {
            let file_count = content.matches("#__SLOPCHOP_FILE__#").count();
            app.pending_payload = Some(content);
            app.log(&format!(
                "📋 Payload detected: {file_count} file(s). Press 'a' or Ctrl+Enter to apply, Esc to dismiss"
            ));
        }
    }
}

fn run_scan(app: &mut DashboardApp) {
    let files = match discovery::discover(app.config) {
        Ok(f) => f,
        Err(e) => {
            app.log(&format!("Scan failed: {e}"));
            return;
        }
    };

    let engine = RuleEngine::new(app.config.clone());
    let report = engine.scan(files);
    app.scan_report = Some(report);
    app.trigger_scan();
}

fn load_roadmap(app: &mut DashboardApp) {
    match TaskStore::load(None) {
        Ok(store) => app.roadmap = Some(store),
        Err(_) => app.roadmap = None,
    }
}