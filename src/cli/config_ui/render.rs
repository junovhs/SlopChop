use super::items::ConfigItem;
use crate::config::Config;
use anyhow::Result;
use crossterm::{
    cursor, execute,
    style::{Color, Print, SetForegroundColor, ResetColor},
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

/// Renders the configuration UI
///
/// # Errors
/// Returns error if terminal manipulation fails.
pub fn draw(items: &[ConfigItem], selected: usize, config: &Config) -> Result<()> {
    let mut stdout = stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    
    draw_header(&mut stdout)?;
    
    for (i, item) in items.iter().enumerate() {
        draw_item(&mut stdout, *item, i == selected, config)?;
    }
    
    draw_footer(&mut stdout)?;
    
    stdout.flush()?;
    Ok(())
}

fn draw_header(stdout: &mut std::io::Stdout) -> Result<()> {
    execute!(
        stdout,
        SetForegroundColor(Color::Cyan),
        Print("┌─ SlopChop Configuration ──────────────────┐\n"),
        ResetColor
    )?;
    Ok(())
}

fn draw_item(stdout: &mut std::io::Stdout, item: ConfigItem, is_selected: bool, config: &Config) -> Result<()> {
    let prefix = if is_selected { "│ >" } else { "│  " };
    let value = item.get_value(config);
    let label = item.label();
    
    if is_selected {
        execute!(stdout, SetForegroundColor(Color::Yellow))?;
    }
    
    writeln!(stdout, "{prefix} {label:<25} {value}")?;
    
    if is_selected {
        execute!(stdout, ResetColor)?;
    }
    Ok(())
}

fn draw_footer(stdout: &mut std::io::Stdout) -> Result<()> {
    execute!(
        stdout,
        Print("│                                           │\n"),
        Print("│  [S]ave  [Esc] Cancel                     │\n"),
        SetForegroundColor(Color::Cyan),
        Print("└───────────────────────────────────────────┘\n"),
        ResetColor
    )?;
    Ok(())
}
