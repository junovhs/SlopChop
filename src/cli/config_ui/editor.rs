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

pub struct ConfigEditor {
    config: Config,
    selected: usize,
    items: Vec<ConfigItem>,
    modified: bool,
}

impl ConfigEditor {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected: 0,
            items: ConfigItem::all().to_vec(),
            modified: false,
        }
    }
    
    /// Runs the interactive editor.
    ///
    /// # Errors
    /// Returns error if terminal manipulation fails.
    pub fn run(&mut self) -> Result<Option<Config>> {
        terminal::enable_raw_mode()?;
        let result = self.event_loop();
        terminal::disable_raw_mode()?;
        result
    }
    
    fn event_loop(&mut self) -> Result<Option<Config>> {
        loop {
            render::draw(&self.items, self.selected, &self.config)?;
            
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                
                match self.handle_key_event(key.code)? {
                    EventResult::Continue => {},
                    EventResult::Exit => return Ok(None),
                    EventResult::Save(config) => return Ok(Some(*config)),
                }
            }
        }
    }

    fn handle_key_event(&mut self, code: KeyCode) -> Result<EventResult> {
        match code {
            KeyCode::Up => {
                self.move_selection(-1);
                Ok(EventResult::Continue)
            }
            KeyCode::Down => {
                self.move_selection(1);
                Ok(EventResult::Continue)
            }
            KeyCode::Enter => {
                self.edit_current()?;
                Ok(EventResult::Continue)
            }
            KeyCode::Char('s' | 'S') => Ok(EventResult::Save(Box::new(self.config.clone()))),
            KeyCode::Esc | KeyCode::Char('q') => Ok(EventResult::Exit),
            _ => Ok(EventResult::Continue)
        }
    }

    #[allow(clippy::cast_possible_wrap)]
    fn move_selection(&mut self, delta: isize) {
        let len = self.items.len() as isize;
        let new_pos = self.selected as isize + delta;
        if new_pos >= 0 && new_pos < len {
            #[allow(clippy::cast_sign_loss)]
            {
                self.selected = new_pos as usize;
            }
        }
    }
    
    fn edit_current(&mut self) -> Result<()> {
        let item = self.items[self.selected];
        
        if item.is_boolean() {
            self.toggle_boolean(item);
            self.modified = true;
        } else if item.is_enum() {
            self.cycle_enum(item);
            self.modified = true;
        } else if let Some(new_val) = self.edit_number(item)? {
            self.set_number(item, new_val);
            self.modified = true;
        }
        Ok(())
    }

    fn toggle_boolean(&mut self, item: ConfigItem) {
        match item {
            ConfigItem::AutoCopy => self.config.preferences.auto_copy = !self.config.preferences.auto_copy,
            ConfigItem::WriteFixPacket => self.config.preferences.write_fix_packet = !self.config.preferences.write_fix_packet,
            ConfigItem::RequirePlan => self.config.preferences.require_plan = !self.config.preferences.require_plan,
            ConfigItem::AutoPromote => self.config.preferences.auto_promote = !self.config.preferences.auto_promote,
            _ => {}
        }
    }
    
    fn cycle_enum(&mut self, item: ConfigItem) {
        if item == ConfigItem::LocalityMode {
            self.config.rules.locality.mode = match self.config.rules.locality.mode.as_str() {
                "warn" => "error".to_string(),
                "error" => "off".to_string(),
                _ => "warn".to_string(),
            };
        }
    }
}

enum EventResult {
    Continue,
    Exit,
    Save(Box<Config>),
}

enum EditResult {
    Continue,
    Commit(usize),
    Cancel,
}

impl ConfigEditor {
    fn edit_number(&self, item: ConfigItem) -> Result<Option<usize>> {
        let mut value = self.get_number(item);
        
        loop {
            self.render_number_editor(value)?;
            
            match Self::handle_number_input(&mut value)? {
                EditResult::Continue => {}
                EditResult::Commit(val) => return Ok(Some(val)),
                EditResult::Cancel => return Ok(None),
            }
        }
    }

    fn render_number_editor(&self, value: usize) -> Result<()> {
        execute!(
            stdout(),
            cursor::MoveTo(40, u16::try_from(self.selected).unwrap_or(0) + 1),
            Clear(ClearType::UntilNewLine),
            Print(format!("[{value}] ←→"))
        )?;
        stdout().flush()?;
        Ok(())
    }

    fn handle_number_input(value: &mut usize) -> Result<EditResult> {
        let Event::Key(key) = event::read()? else {
            return Ok(EditResult::Continue);
        };
        
        if key.kind != KeyEventKind::Press {
            return Ok(EditResult::Continue);
        }

        Ok(Self::process_number_key(key.code, value))
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

    fn get_number(&self, item: ConfigItem) -> usize {
        match item {
            ConfigItem::MaxTokens => self.config.rules.max_file_tokens,
            ConfigItem::MaxComplexity => self.config.rules.max_cyclomatic_complexity,
            ConfigItem::MaxNesting => self.config.rules.max_nesting_depth,
            ConfigItem::MaxArgs => self.config.rules.max_function_args,
            ConfigItem::LocalityMaxDistance => self.config.rules.locality.max_distance,
            _ => 0,
        }
    }
    
    fn set_number(&mut self, item: ConfigItem, value: usize) {
        match item {
            ConfigItem::MaxTokens => self.config.rules.max_file_tokens = value,
            ConfigItem::MaxComplexity => self.config.rules.max_cyclomatic_complexity = value,
            ConfigItem::MaxNesting => self.config.rules.max_nesting_depth = value,
            ConfigItem::MaxArgs => self.config.rules.max_function_args = value,
            ConfigItem::LocalityMaxDistance => self.config.rules.locality.max_distance = value,
            _ => {}
        }
    }
}

/// Entry point for the config command
///
/// # Errors
/// Returns error if loading config, running editor, or saving config fails.
pub fn run_config_editor() -> Result<()> {
    let config = Config::load();
    let mut editor = ConfigEditor::new(config);
    
    if let Some(new_config) = editor.run()? {
        new_config.save()?;
        println!("Configuration saved.");
    } else {
        println!("Configuration unchanged.");
    }
    
    Ok(())
}
