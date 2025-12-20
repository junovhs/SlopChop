pub mod apply;
pub mod input;
pub mod state;
pub mod ui;

use crate::analysis::RuleEngine;
use crate::config::Config;
use crate::discovery;
use crate::tui::runner;
use crate::tui::watcher::{self, WatcherEvent};
use anyhow::Result;
use crossterm::event::{self};
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
    run_scan(&mut app);

    let (tx, rx) = mpsc::channel();
    watcher::spawn_watcher(tx);
    app.log("Integrity Watcher Engaged");

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Ok(event) = rx.try_recv() {
            handle_watcher_event(&mut app, event);
        }

        if event::poll(Duration::from_millis(100))? {
            if let event::Event::Key(key) = event::read()? {
                input::handle_input(key.code, key.modifiers, &mut app, terminal);
            }
        }

        if app.should_quit { break; }
    }
    Ok(())
}

fn handle_watcher_event(app: &mut DashboardApp, event: WatcherEvent) {
    match event {
        WatcherEvent::PayloadDetected(content) => {
            app.pending_payload = Some(content);
            app.log("?? XSC7XSC Payload Received. Press 'a' to apply.");
        }
    }
}

fn run_scan(app: &mut DashboardApp) {
    if let Ok(files) = discovery::discover(app.config) {
        let engine = RuleEngine::new(app.config.clone());
        app.scan_report = Some(engine.scan(files));
    }
}