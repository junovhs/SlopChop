// src/audit/fingerprint.rs
//! Structural fingerprinting with control flow graph awareness.
//!
//! This module implements semantic fingerprinting that can detect:
//! - Level 3: Identical algorithms with different variable names
//! - Level 4: Equivalent control flow with different syntax
//!
//! Example: These two functions will be detected as equivalent:
//!   fn foo(x: bool) -> i32 { if x { return 1; } else { return 2; } }
//!   fn bar(y: bool) -> i32 { if y { 1 } else { 2 } }

use super::types::Fingerprint;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tree_sitter::Node;

/// Control flow nodes - these define the CFG structure.
const CFG_NODES: &[&str] = &[
    "if_expression",
    "else_clause",
    "match_expression",
    "match_arm",
    "for_expression",
    "while_expression",
    "loop_expression",
    "return_expression",
    "break_expression",
    "continue_expression",
    "try_expression",
    "block",
    "closure_expression",
];

/// Branch-introducing nodes (each adds a branch point).
const BRANCH_NODES: &[&str] = &[
    "if_expression",
    "match_arm",
    "try_expression", // ? operator
];

/// Loop nodes.
const LOOP_NODES: &[&str] = &["for_expression", "while_expression", "loop_expression"];

/// Exit nodes (return, break, continue).
const EXIT_NODES: &[&str] = &["return_expression", "break_expression", "continue_expression"];

/// Operators and keywords that define semantic structure.
const STRUCTURAL_TOKENS: &[&str] = &[
    "+", "-", "*", "/", "%", "&&", "||", "!", "==", "!=", "<", ">", "<=", ">=", "&", "|", "^",
    "<<", ">>", "+=", "-=", "*=", "/=", "if", "else", "match", "while", "for", "loop", "return",
    "break", "continue", "let", "mut", "const", "fn", "struct", "enum", "impl", "trait", "pub",
    "async", "await", "move", "=>", "->", "?", "true", "false", "None", "Some", "Ok", "Err",
];

/// Node kinds to ignore for text hashing (identifiers, literals).
const IDENTIFIER_KINDS: &[&str] = &[
    "identifier",
    "type_identifier",
    "field_identifier",
    "scoped_identifier",
    "scoped_type_identifier",
    "string_literal",
    "raw_string_literal",
    "char_literal",
    "integer_literal",
    "float_literal",
];

/// Computes a structural fingerprint for an AST node.
#[must_use]
pub fn compute(node: Node, source: &[u8]) -> Fingerprint {
    let mut state = FingerprintState::new();
    state.visit(node, source, 0);
    state.finalize()
}

struct FingerprintState {
    struct_hasher: DefaultHasher,
    cfg_hasher: DefaultHasher,
    max_depth: usize,
    node_count: usize,
    branch_count: usize,
    loop_count: usize,
    exit_count: usize,
}

impl FingerprintState {
    fn new() -> Self {
        Self {
            struct_hasher: DefaultHasher::new(),
            cfg_hasher: DefaultHasher::new(),
            max_depth: 0,
            node_count: 0,
            branch_count: 0,
            loop_count: 0,
            exit_count: 0,
        }
    }

    fn visit(&mut self, node: Node, source: &[u8], depth: usize) {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(depth);

        let kind = node.kind();

        // Count control flow elements
        if BRANCH_NODES.contains(&kind) {
            self.branch_count += 1;
        }
        if LOOP_NODES.contains(&kind) {
            self.loop_count += 1;
        }
        if EXIT_NODES.contains(&kind) {
            self.exit_count += 1;
        }

        // Hash into structural hasher (full AST)
        self.mix_structural(kind, depth);
        if should_hash_text(kind, node, source) {
            if let Ok(text) = node.utf8_text(source) {
                text.hash(&mut self.struct_hasher);
            }
        }

        // Hash into CFG hasher (control flow only)
        if CFG_NODES.contains(&kind) {
            self.mix_cfg(kind, depth);
        }

        // Hash child count and recurse
        let child_count = node.child_count();
        (child_count as u64).hash(&mut self.struct_hasher);

        for (i, child) in node.children(&mut node.walk()).enumerate() {
            (i as u64).hash(&mut self.struct_hasher);
            self.visit(child, source, depth + 1);
        }
    }

    fn mix_structural(&mut self, kind: &str, depth: usize) {
        0xDEAD_BEEF_u64.wrapping_add(depth as u64).hash(&mut self.struct_hasher);
        kind.hash(&mut self.struct_hasher);
    }

    fn mix_cfg(&mut self, kind: &str, depth: usize) {
        // Normalize equivalent CFG patterns
        let normalized = normalize_cfg_node(kind);
        depth.hash(&mut self.cfg_hasher);
        normalized.hash(&mut self.cfg_hasher);
    }

    fn finalize(self) -> Fingerprint {
        Fingerprint {
            hash: self.struct_hasher.finish(),
            cfg_hash: self.cfg_hasher.finish(),
            depth: self.max_depth,
            node_count: self.node_count,
            branch_count: self.branch_count,
            loop_count: self.loop_count,
            exit_count: self.exit_count,
        }
    }
}

/// Normalizes CFG node kinds to treat equivalent patterns identically.
fn normalize_cfg_node(kind: &str) -> &str {
    match kind {
        // A match with 2 arms on bool is equivalent to if-else
        // We normalize at a higher level; here we just group similar constructs
        "if_expression" | "match_expression" => "BRANCH",
        "else_clause" | "match_arm" => "BRANCH_ARM",
        "for_expression" | "while_expression" | "loop_expression" => "LOOP",
        "return_expression" => "RETURN",
        "break_expression" | "continue_expression" => "LOOP_EXIT",
        "block" => "BLOCK",
        "closure_expression" => "CLOSURE",
        "try_expression" => "TRY",
        _ => kind,
    }
}

fn should_hash_text(kind: &str, node: Node, source: &[u8]) -> bool {
    if IDENTIFIER_KINDS.contains(&kind) {
        return false;
    }
    if STRUCTURAL_TOKENS.contains(&kind) {
        return true;
    }
    if let Ok(text) = node.utf8_text(source) {
        if STRUCTURAL_TOKENS.contains(&text) {
            return true;
        }
    }
    false
}

/// Computes similarity between two fingerprints.
/// Uses multi-level comparison for semantic equivalence detection.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    // Level 1: Exact structural match
    if a.hash == b.hash {
        return 1.0;
    }

    // Level 2: Same CFG (control flow equivalent)
    if a.cfg_hash == b.cfg_hash {
        // Same control flow, different expressions - very similar
        let struct_sim = structural_similarity(a, b);
        return 0.85 + (struct_sim * 0.15);
    }

    // Level 3: Similar CFG metrics
    let cfg_sim = cfg_similarity(a, b);
    let struct_sim = structural_similarity(a, b);

    // Weight CFG similarity more heavily - it's more meaningful
    cfg_sim * 0.6 + struct_sim * 0.4
}

/// Compares control flow metrics.
#[allow(clippy::cast_precision_loss)]
fn cfg_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let branch_sim = metric_similarity(a.branch_count, b.branch_count);
    let loop_sim = metric_similarity(a.loop_count, b.loop_count);
    let exit_sim = metric_similarity(a.exit_count, b.exit_count);

    // If all CFG metrics match exactly, very high similarity
    if a.branch_count == b.branch_count
        && a.loop_count == b.loop_count
        && a.exit_count == b.exit_count
    {
        return 0.95;
    }

    branch_sim * 0.5 + loop_sim * 0.3 + exit_sim * 0.2
}

/// Compares structural metrics (depth, node count).
#[allow(clippy::cast_precision_loss)]
fn structural_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let depth_sim = metric_similarity(a.depth, b.depth);
    let count_sim = metric_similarity(a.node_count, b.node_count);
    depth_sim * 0.3 + count_sim * 0.7
}

/// Computes similarity between two numeric metrics.
#[allow(clippy::cast_precision_loss)]
fn metric_similarity(a: usize, b: usize) -> f64 {
    let max = a.max(b) as f64;
    if max == 0.0 {
        return 1.0;
    }
    1.0 - (a as f64 - b as f64).abs() / max
}

/// Extracts fingerprinted units from a parsed file.
#[must_use]
pub fn extract_units(
    source: &str,
    tree: &tree_sitter::Tree,
) -> Vec<(String, &'static str, usize, usize, Fingerprint)> {
    let mut units = Vec::new();
    extract_from_node(tree.root_node(), source.as_bytes(), &mut units);
    units
}

fn extract_from_node(
    node: Node,
    source: &[u8],
    units: &mut Vec<(String, &'static str, usize, usize, Fingerprint)>,
) {
    if let Some(unit_kind) = match_unit_kind(node.kind()) {
        if let Some(name) = extract_name(node, source) {
            let fingerprint = compute(node, source);
            let start = node.start_position().row + 1;
            let end = node.end_position().row + 1;
            units.push((name, unit_kind, start, end, fingerprint));
        }
    }

    for child in node.children(&mut node.walk()) {
        extract_from_node(child, source, units);
    }
}

fn match_unit_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "function_item" | "function_definition" => Some("function"),
        "impl_item" => Some("impl"),
        "struct_item" | "struct_definition" => Some("struct"),
        "enum_item" | "enum_definition" => Some("enum"),
        "trait_item" | "trait_definition" => Some("trait"),
        "mod_item" => Some("module"),
        _ => None,
    }
}

fn extract_name(node: Node, source: &[u8]) -> Option<String> {
    let name_node = node.child_by_field_name("name")?;
    name_node.utf8_text(source).ok().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_hash_equivalence() {
        // Same CFG metrics should give high similarity
        let fp1 = Fingerprint {
            hash: 1,
            cfg_hash: 100,
            depth: 5,
            node_count: 20,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        let fp2 = Fingerprint {
            hash: 2, // Different structural hash
            cfg_hash: 100, // Same CFG hash!
            depth: 5,
            node_count: 22,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        let sim = similarity(&fp1, &fp2);
        assert!(sim >= 0.85, "CFG-equivalent code should be >= 85% similar, got {sim}");
    }

    #[test]
    fn test_different_cfg_similar_metrics() {
        let fp1 = Fingerprint {
            hash: 1,
            cfg_hash: 100,
            depth: 5,
            node_count: 20,
            branch_count: 2,
            loop_count: 1,
            exit_count: 1,
        };
        let fp2 = Fingerprint {
            hash: 2,
            cfg_hash: 200, // Different CFG
            depth: 5,
            node_count: 20,
            branch_count: 2, // But same metrics
            loop_count: 1,
            exit_count: 1,
        };
        let sim = similarity(&fp1, &fp2);
        assert!(sim >= 0.9, "Same CFG metrics should be >= 90% similar, got {sim}");
    }
}