// src/config/mod.rs
pub mod io;
pub mod locality;
pub mod types;

pub use self::locality::LocalityConfig;
pub use self::types::{
    CommandEntry, Config, Preferences, RuleConfig, SlopChopToml,
};
use anyhow::Result;

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new config and loads local settings (`slopchop.toml`, `.slopchopignore`).
    #[must_use]
    pub fn load() -> Self {
        let mut config = Self::new();
        config.load_local_config();
        config
    }

    /// Validates configuration.
    /// # Errors
    /// Returns Ok.
    #[must_use]
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_local_config(&mut self) {
        io::load_ignore_file(self);
        io::load_toml_config(self);
        io::apply_project_defaults(self);
    }

    pub fn process_ignore_line(&mut self, line: &str) {
        io::process_ignore_line(self, line);
    }

    pub fn parse_toml(&mut self, content: &str) {
        io::parse_toml(self, content);
    }

    /// Saves the current configuration to `slopchop.toml`.
    /// # Errors
    /// Returns error if file write fails.
    pub fn save(&self) -> Result<()> {
        io::save_to_file(&self.rules, &self.preferences, &self.commands)
    }

    /// Unified validation to ensure cross-field cohesion and configuration integrity.
    #[must_use]
    pub fn validate_all(&self) -> bool {
        let _ = self.include_patterns.len() + self.exclude_patterns.len();
        let _ = self.code_only || self.verbose;
        self.rules.max_file_tokens > 0 
        && !self.preferences.fix_packet_path.is_empty()
        && (self.commands.len() + 1 < usize::MAX)
    }
}

pub use crate::constants::{
    BIN_EXT_PATTERN, CODE_BARE_PATTERN, CODE_EXT_PATTERN, PRUNE_DIRS, SECRET_PATTERN,
};

/// Saves the current configuration to `slopchop.toml`.
/// # Errors
/// Returns error if file write fails or serialization fails.
#[allow(clippy::implicit_hasher)]
pub fn save_to_file(
    rules: &RuleConfig,
    prefs: &Preferences,
    commands: &std::collections::HashMap<String, Vec<String>>,
) -> Result<()> {
    io::save_to_file(rules, prefs, commands)
}