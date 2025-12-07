// src/tui/dashboard/input.rs
use super::apply;
use super::state::{DashboardApp, Tab};
use crate::discovery;
use crate::roadmap_v2::types::TaskStore;
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
        Tab::Roadmap => handle_roadmap_input(code, app),
        Tab::Logs => handle_scrolling(code, app),
        Tab::Dashboard => {}
    }
}

fn handle_global_navigation(
    code: KeyCode,
    modifiers: KeyModifiers,
    app: &mut DashboardApp,
) -> bool {
    if matches!((modifiers, code), (_, KeyCode::Char('q'))) {
        app.quit();
        return true;
    }

    if handle_tab_nav(code, modifiers, app) {
        return true;
    }

    handle_view_switch(code, app)
}

fn handle_tab_nav(code: KeyCode, modifiers: KeyModifiers, app: &mut DashboardApp) -> bool {
    match (modifiers, code) {
        (_, KeyCode::Tab) => {
            app.next_tab();
            true
        }
        (KeyModifiers::SHIFT, KeyCode::BackTab) => {
            app.previous_tab();
            true
        }
        _ => false,
    }
}

fn handle_view_switch(code: KeyCode, app: &mut DashboardApp) -> bool {
    match code {
        KeyCode::Char('1') => {
            app.active_tab = Tab::Dashboard;
            true
        }
        KeyCode::Char('2') => {
            app.active_tab = Tab::Roadmap;
            true
        }
        KeyCode::Char('3') => {
            app.active_tab = Tab::Config;
            true
        }
        KeyCode::Char('4') => {
            app.active_tab = Tab::Logs;
            true
        }
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
        (_, KeyCode::Char('f')) => {
            app.roadmap_filter = app.roadmap_filter.next();
            app.roadmap_unselect();
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
    match code {
        KeyCode::Left | KeyCode::Char('h') => {
            crate::tui::config::helpers::adjust_rule(&mut app.config_editor, false);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            crate::tui::config::helpers::adjust_rule(&mut app.config_editor, true);
        }
        KeyCode::Char('s') | KeyCode::Enter => {
            app.config_editor.save();
            app.log("Config saved");
        }
        _ => {}
    }
}

fn handle_roadmap_input(code: KeyCode, app: &mut DashboardApp) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => app.roadmap_next(),
        KeyCode::Up | KeyCode::Char('k') => app.roadmap_previous(),
        KeyCode::Char(' ') | KeyCode::Enter => app.toggle_roadmap_task(),
        _ => {}
    }
}

fn handle_scrolling(code: KeyCode, app: &mut DashboardApp) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            if app.active_tab == Tab::Logs {
                app.scroll = app.scroll.saturating_add(1);
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.active_tab == Tab::Logs {
                app.scroll = app.scroll.saturating_sub(1);
            }
        }
        _ => {}
    }
}

fn refresh_app(app: &mut DashboardApp) {
    // We can't import run_scan/load_roadmap from mod.rs easily without circular deps
    // so we reimplement the logic here or call helpers.
    // Ideally these should be on DashboardApp or a service module.
    // For now, let's keep it simple:

    // 1. Scan
    let files = match discovery::discover(app.config) {
        Ok(f) => f,
        Err(e) => {
            app.log(&format!("Scan failed: {e}"));
            return;
        }
    };
    let engine = crate::analysis::RuleEngine::new(app.config.clone());
    let report = engine.scan(files);
    app.scan_report = Some(report);
    app.trigger_scan();

    // 2. Roadmap
    if let Ok(store) = TaskStore::load(None) {
        app.roadmap = Some(store);
    } else {
        app.roadmap = None;
    }

    app.log("Refreshed");
}