// src/analysis/ast.rs
use super::checks::{self, CheckContext};
use crate::config::RuleConfig;
use crate::lang::Lang;
use crate::types::Violation;
use anyhow::{anyhow, Result};
use tree_sitter::{Language, Parser, Query};

pub struct Analyzer;

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn analyze(
        &self,
        ext: &str,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let Some(lang) = Lang::from_ext(ext) else {
            return vec![];
        };
        Self::run_analysis(lang, filename, content, config)
    }

    fn run_analysis(
        lang: Lang,
        filename: &str,
        content: &str,
        config: &RuleConfig,
    ) -> Vec<Violation> {
        let grammar = lang.grammar();
        let mut parser = Parser::new();
        if parser.set_language(grammar).is_err() {
            return vec![];
        }

        let Some(tree) = parser.parse(content, None) else {
            return vec![];
        };

        // Fallible compilation steps
        let Ok(q_naming) = compile_query(grammar, lang.q_naming()) else {
            return vec![];
        };

        let Ok(q_complexity) = compile_query(grammar, lang.q_complexity()) else {
            return vec![];
        };

        // Banned query is optional and fallible
        let q_banned = lang
            .q_banned()
            .and_then(|q| compile_query(grammar, q).ok());

        let mut violations = Vec::new();
        let ctx = CheckContext {
            root: tree.root_node(),
            source: content,
            filename,
            config,
        };

        checks::check_naming(&ctx, &q_naming, &mut violations);
        checks::check_metrics(&ctx, &q_complexity, &mut violations);

        if let Some(banned) = q_banned {
            checks::check_banned(&ctx, &banned, &mut violations);
        }

        violations
    }
}

fn compile_query(lang: Language, pattern: &str) -> Result<Query> {
    Query::new(lang, pattern).map_err(|e| anyhow!("Invalid tree-sitter query: {e}"))
}