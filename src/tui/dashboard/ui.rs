use crate::tui::dashboard::state::{DashboardApp, Tab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    match app.active_tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[1]),
        Tab::Config => draw_config(f, app, chunks[1]),
        Tab::Logs => draw_logs(f, app, chunks[1]),
    }
    draw_footer(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let titles: Vec<Line> = vec!["[1] Dashboard", "[2] Config", "[3] Logs"]
        .into_iter()
        .map(|t| Line::from(Span::styled(t, Style::default().fg(Color::Green))))
        .collect();
    
    let tabs = Tabs::new(titles).block(Block::default().borders(Borders::ALL).title("SlopChop"))
        .select(app.active_tab as usize)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));
    f.render_widget(tabs, area);
}

fn draw_dashboard(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let mut msg = "System Nominal. Ready for payloads.".to_string();
    if app.pending_payload.is_some() {
        msg = "\n?? XSC7XSC PAYLOAD DETECTED\nPress 'a' to apply.".to_string();
    }
    let status = Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, area);
}

fn draw_config(f: &mut Frame, app: &mut DashboardApp, area: Rect) {
    crate::tui::config::view::draw_embed(f, &app.config_editor, area);
}

fn draw_logs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let logs: Vec<ListItem> = app.logs.iter().rev().map(|s| ListItem::new(s.as_str())).collect();
    let list = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, _app: &DashboardApp, area: Rect) {
    let text = "q: Quit | TAB: Switch | a: APPLY PAYLOAD";
    let p = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}