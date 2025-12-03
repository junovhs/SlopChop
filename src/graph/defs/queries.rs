// src/graph/defs/queries.rs
//! Tree-sitter query definitions for extracting symbol definitions.

use std::sync::LazyLock;
use tree_sitter::{Language, Query};

pub static EXTRACTOR: LazyLock<DefExtractor> = LazyLock::new(DefExtractor::new);

pub struct DefExtractor {
    pub rust: Query,
    pub python: Query,
    pub typescript: Query,
}

impl DefExtractor {
    fn new() -> Self {
        Self {
            rust: compile_query(tree_sitter_rust::language(), RUST_QUERY),
            python: compile_query(tree_sitter_python::language(), PYTHON_QUERY),
            typescript: compile_query(tree_sitter_typescript::language_typescript(), TS_QUERY),
        }
    }

    #[must_use]
    pub fn get_config(&self, ext: &str) -> Option<(Language, &Query)> {
        match ext {
            "rs" => Some((tree_sitter_rust::language(), &self.rust)),
            "py" => Some((tree_sitter_python::language(), &self.python)),
            "ts" | "tsx" | "js" | "jsx" => Some((
                tree_sitter_typescript::language_typescript(),
                &self.typescript,
            )),
            _ => None,
        }
    }
}

const RUST_QUERY: &str = r"
(function_item name: (identifier) @name) @sig
(struct_item name: (type_identifier) @name) @sig
(enum_item name: (type_identifier) @name) @sig
(trait_item name: (type_identifier) @name) @sig
(impl_item type: (type_identifier) @name) @sig
(const_item name: (identifier) @name) @sig
(static_item name: (identifier) @name) @sig
(type_item name: (type_identifier) @name) @sig
";

const PYTHON_QUERY: &str = r"
(function_definition name: (identifier) @name) @sig
(class_definition name: (identifier) @name) @sig
";

const TS_QUERY: &str = r"
(function_declaration name: (identifier) @name) @sig
(class_declaration name: (type_identifier) @name) @sig
(interface_declaration name: (type_identifier) @name) @sig
(type_alias_declaration name: (type_identifier) @name) @sig
";

fn compile_query(lang: Language, pattern: &str) -> Query {
    Query::new(lang, pattern).unwrap_or_else(|e| panic!("Invalid query: {e}"))
}

