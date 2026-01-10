use super::items::ConfigItem;
use super::render;
use crate::config::Config;
use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::Print,
    terminal::{self, Clear, ClearType},
};
use std::io::{stdout, Write};
use super::editor::{ConfigEditor, EventResult, EditResult};

#[must_use]
pub fn run_editor(editor: &mut ConfigEditor) -> Result<Option<Config>> {
    terminal::enable_raw_mode()?;
    let result = event_loop(editor);
    terminal::disable_raw_mode()?;
    result
}

#[must_use]
fn event_loop(editor: &mut ConfigEditor) -> Result<Option<Config>> {
     loop {
        render::draw(editor.items(), editor.selected(), editor.config())?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            
            match handle_key_event(editor, key.code)? {
                EventResult::Continue => {},
                EventResult::Exit => return Ok(None),
                EventResult::Save(config) => return Ok(Some(*config)),
            }
        }
    }
}

#[must_use]
fn handle_key_event(editor: &mut ConfigEditor, code: KeyCode) -> Result<EventResult> {
     match code {
        KeyCode::Up => {
            move_selection(editor, -1);
            Ok(EventResult::Continue)
        }
        KeyCode::Down => {
            move_selection(editor, 1);
            Ok(EventResult::Continue)
        }
        KeyCode::Enter => {
            edit_current(editor)?;
            Ok(EventResult::Continue)
        }
        KeyCode::Char('s' | 'S') => Ok(EventResult::Save(Box::new(editor.config().clone()))),
        KeyCode::Esc | KeyCode::Char('q') => Ok(EventResult::Exit),
        _ => Ok(EventResult::Continue)
    }
}

pub fn move_selection(editor: &mut ConfigEditor, delta: isize) {
    let len = editor.items().len() as isize;
    let new_pos = editor.selected() as isize + delta;
    if new_pos >= 0 && new_pos < len {
        #[allow(clippy::cast_sign_loss)]
        {
            editor.set_selected(new_pos as usize);
        }
    }
}

#[must_use]
fn edit_current(editor: &mut ConfigEditor) -> Result<()> {
     let selected = editor.selected();
    let item = editor.items()[selected];
    
    if item.is_boolean() {
        item.toggle_boolean(editor.config_mut());
        editor.set_modified(true);
    } else if item.is_enum() {
        item.cycle_enum(editor.config_mut());
        editor.set_modified(true);
    } else if let Some(new_val) = edit_number(editor, item)? {
        item.set_number(editor.config_mut(), new_val);
        editor.set_modified(true);
    }
    Ok(())
}

#[must_use]
fn edit_number(editor: &ConfigEditor, item: ConfigItem) -> Result<Option<usize>> {
     let mut value = item.get_number(editor.config());
    
    loop {
        render_number_editor(editor.selected(), value)?;
        
        match handle_number_input(&mut value)? {
            EditResult::Continue => {}
            EditResult::Commit(val) => return Ok(Some(val)),
            EditResult::Cancel => return Ok(None),
        }
    }
}

#[must_use]
fn render_number_editor(selected: usize, value: usize) -> Result<()> {
     execute!(
        stdout(),
        cursor::MoveTo(40, u16::try_from(selected).unwrap_or(0) + 1),
        Clear(ClearType::UntilNewLine),
        Print(format!("[{value}] \u{2190}\u{2192}"))
    )?;
    stdout().flush()?;
    Ok(())
}

#[must_use]
fn handle_number_input(value: &mut usize) -> Result<EditResult> {
      let Event::Key(key) = event::read()? else {
        return Ok(EditResult::Continue);
    };
    
    if key.kind != KeyEventKind::Press {
        return Ok(EditResult::Continue);
    }

    Ok(process_number_key(key.code, value))
}

fn process_number_key(code: KeyCode, value: &mut usize) -> EditResult {
    match code {
        KeyCode::Left if *value > 1 => {
            *value -= 1;
            EditResult::Continue
        }
        KeyCode::Right => {
            *value += 1;
            EditResult::Continue
        }
        KeyCode::Enter => EditResult::Commit(*value),
        KeyCode::Esc => EditResult::Cancel,
        _ => EditResult::Continue
    }
}
