// src/config.rs
use crate::error::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct RuleConfig {
    #[serde(default = "default_max_tokens")]
    pub max_file_tokens: usize,
    #[serde(default = "default_max_complexity")]
    pub max_cyclomatic_complexity: usize,
    #[serde(default = "default_max_depth")]
    pub max_nesting_depth: usize,
    #[serde(default = "default_max_args")]
    pub max_function_args: usize,
    #[serde(default = "default_max_words")]
    pub max_function_words: usize,
    #[serde(default)]
    pub ignore_naming_on: Vec<String>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            max_file_tokens: default_max_tokens(),
            max_cyclomatic_complexity: default_max_complexity(),
            max_nesting_depth: default_max_depth(),
            max_function_args: default_max_args(),
            max_function_words: default_max_words(),
            ignore_naming_on: Vec::new(),
        }
    }
}

const fn default_max_tokens() -> usize {
    2000
}
const fn default_max_complexity() -> usize {
    10
}
const fn default_max_depth() -> usize {
    4
}
const fn default_max_args() -> usize {
    5
}
const fn default_max_words() -> usize {
    3
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct WardenToml {
    #[serde(default)]
    pub rules: RuleConfig,
    #[serde(default)]
    pub commands: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum GitMode {
    Auto,
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub git_mode: GitMode,
    pub include_patterns: Vec<Regex>,
    pub exclude_patterns: Vec<Regex>,
    pub code_only: bool,
    pub verbose: bool,
    pub rules: RuleConfig,
    pub commands: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    #[must_use]
    pub fn new() -> Self {
        Self {
            git_mode: GitMode::Auto,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            code_only: false,
            verbose: false,
            rules: RuleConfig::default(),
            commands: HashMap::new(),
        }
    }

    /// Validates configuration.
    /// # Errors
    /// Returns `Ok`.
    pub fn validate(&self) -> Result<()> {
        Ok(())
    }

    pub fn load_local_config(&mut self) {
        self.load_ignore_file();
        self.load_toml_config();
        self.detect_defaults();
    }

    fn detect_defaults(&mut self) {
        if self.commands.contains_key("check") {
            return;
        }

        match Self::detect_project_type() {
            ProjectType::Rust => {
                self.set_default(
                    "check",
                    "cargo clippy --all-targets -- -D warnings -D clippy::pedantic",
                );
                self.set_default("fix", "cargo fmt");
            }
            ProjectType::Node => {
                self.configure_node_defaults();
            }
            ProjectType::Python => {
                self.set_default("check", "ruff check --fix .");
                self.set_default("fix", "ruff format .");
            }
            ProjectType::Unknown => {}
        }
    }

    #[must_use]
    pub fn detect_project_type() -> ProjectType {
        if Path::new("Cargo.toml").exists() {
            return ProjectType::Rust;
        }
        if Path::new("package.json").exists() {
            return ProjectType::Node;
        }
        if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
            return ProjectType::Python;
        }
        ProjectType::Unknown
    }

    fn configure_node_defaults(&mut self) {
        let npx = Self::npx_cmd();
        // Always default to biome - it will auto-install via npx
        self.set_default("check", &format!("{npx} @biomejs/biome check src/"));
        self.set_default("fix", &format!("{npx} @biomejs/biome check --write src/"));
    }

    #[must_use]
    pub fn npx_cmd() -> &'static str {
        if cfg!(windows) {
            "npx.cmd"
        } else {
            "npx"
        }
    }

    #[must_use]
    pub fn npm_cmd() -> &'static str {
        if cfg!(windows) {
            "npm.cmd"
        } else {
            "npm"
        }
    }

    #[must_use]
    pub fn cargo_cmd() -> &'static str {
        if cfg!(windows) {
            "cargo.exe"
        } else {
            "cargo"
        }
    }

    fn set_default(&mut self, key: &str, val: &str) {
        if !self.commands.contains_key(key) {
            self.commands.insert(key.to_string(), val.to_string());
        }
    }

    fn load_ignore_file(&mut self) {
        if let Ok(content) = fs::read_to_string(".wardenignore") {
            for line in content.lines() {
                self.process_ignore_line(line);
            }
        }
    }

    fn process_ignore_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return;
        }
        if let Ok(re) = Regex::new(trimmed) {
            self.exclude_patterns.push(re);
        }
    }

    fn load_toml_config(&mut self) {
        if Path::new("warden.toml").exists() {
            if let Ok(content) = fs::read_to_string("warden.toml") {
                self.parse_toml(&content);
            }
        }
    }

    fn parse_toml(&mut self, content: &str) {
        if let Ok(parsed) = toml::from_str::<WardenToml>(content) {
            self.rules = parsed.rules;
            self.commands = parsed.commands;
            if self.verbose {
                println!("🔧 Loaded warden.toml");
            }
        }
    }

    /// Generates a warden.toml content string based on detected project type.
    #[must_use]
    pub fn generate_toml_content() -> String {
        let project = Self::detect_project_type();
        let commands_section = match project {
            ProjectType::Rust => {
                r#"[commands]
check = "cargo clippy --all-targets -- -D warnings -D clippy::pedantic"
fix = "cargo fmt""#
            }
            ProjectType::Node => {
                let npx = Self::npx_cmd();
                return format!(
                    r#"# warden.toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4
max_function_args = 5
max_function_words = 3
ignore_naming_on = ["tests", "spec"]

[commands]
check = "{npx} @biomejs/biome check src/"
fix = "{npx} @biomejs/biome check --write src/"
"#
                );
            }
            ProjectType::Python => {
                r#"[commands]
check = "ruff check --fix ."
fix = "ruff format .""#
            }
            ProjectType::Unknown => {
                r#"# No project type detected. Configure commands manually:
# [commands]
# check = "your-lint-command"
# fix = "your-fix-command""#
            }
        };

        format!(
            r#"# warden.toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4
max_function_args = 5
max_function_words = 3
ignore_naming_on = ["tests", "spec"]

{commands_section}
"#
        )
    }
}

pub const PRUNE_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "gen",
    ".venv",
    "venv",
    ".tox",
    "__pycache__",
    "coverage",
    "vendor",
    ".warden_apply_backup",
    "Cargo.lock",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "bun.lockb",
    "go.sum",
    "Gemfile.lock",
    "tests",
    "test",
    "spec",
    "docs",
    "examples",
    "fixtures",
];

pub const BIN_EXT_PATTERN: &str =
    r"(?i)\.(png|jpg|gif|svg|ico|webp|woff2?|ttf|pdf|mp4|zip|gz|tar|exe|dll|so|dylib|class|pyc)$";
pub const SECRET_PATTERN: &str =
    r"(?i)(^\.?env(\..*)?$|/\.?env(\..*)?$|(^|/)(id_rsa|id_ed25519|.*\.(pem|p12|key|pfx))$)";
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(rs|go|py|js|jsx|ts|tsx|java|c|cpp|h|hpp|cs|php|rb|sh|sql|html|css|scss|json|toml|yaml|md)$";
pub const CODE_BARE_PATTERN: &str = r"(?i)(Makefile|Dockerfile|CMakeLists\.txt)$";
