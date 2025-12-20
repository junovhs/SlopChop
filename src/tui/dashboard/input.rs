use super::apply;
use super::state::{DashboardApp, Tab};
use crate::discovery;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

pub fn handle_input(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut DashboardApp,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) {
    if handle_global_navigation(code, modifiers, app) {
        return;
    }

    if handle_actions(code, modifiers, app, terminal) {
        return;
    }

    match app.active_tab {
        Tab::Config => handle_config_input(code, app),
        Tab::Logs | Tab::Dashboard => {}
    }
}

fn handle_global_navigation(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut DashboardApp,
) -> bool {
    if matches!((modifiers, code), (_, KeyCode::Char('q'))) {
        app.should_quit = true;
        return true;
    }

    if code == KeyCode::Tab {
        app.cycle_tab(false);
        return true;
    }

    handle_view_switch(code, app)
}

fn handle_view_switch(code: KeyCode, app: &mut DashboardApp) -> bool {
    match code {
        KeyCode::Char('1') => { app.active_tab = Tab::Dashboard; true }
        KeyCode::Char('2') => { app.active_tab = Tab::Config; true }
        KeyCode::Char('3') => { app.active_tab = Tab::Logs; true }
        _ => false,
    }
}

fn handle_actions(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut DashboardApp,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> bool {
    match (modifiers, code) {
        (_, KeyCode::Char('r')) => {
            refresh_app(app);
            true
        }
        (KeyModifiers::CONTROL, KeyCode::Enter) | (_, KeyCode::Char('a')) => {
            if app.pending_payload.is_some() {
                apply::handle_interactive_apply(app, terminal);
            }
            true
        }
        (_, KeyCode::Esc) => {
            if app.pending_payload.is_some() {
                app.pending_payload = None;
                app.log("Payload dismissed");
            }
            true
        }
        _ => false,
    }
}

fn handle_config_input(code: KeyCode, app: &mut DashboardApp) {
    app.config_editor.handle_input(code);
}

fn refresh_app(app: &mut DashboardApp) {
    let files = match discovery::discover(app.config) {
        Ok(f) => f,
        Err(e) => {
            app.log(&format!("Scan failed: {e}"));
            return;
        }
    };
    let engine = crate::analysis::RuleEngine::new(app.config.clone());
    app.scan_report = Some(engine.scan(files));
    app.log("Refreshed");
}