use super::codegen;
use super::diff;
use super::parameterize;
use super::types::{Opportunity, OpportunityKind};
use crate::config::Config;
use crate::lang::Lang;
use tree_sitter::Parser;
use std::fs;

pub fn enhance_opportunities(opportunities: &mut [Opportunity], limit: usize, _config: &Config) {
    let mut enhanced_count = 0;

    for opportunity in opportunities.iter_mut() {
        if enhanced_count >= limit {
            break;
        }

        if opportunity.kind != OpportunityKind::Duplication {
            continue;
        }

        // Only enhance high-confidence duplicates
        if opportunity.impact.confidence < 0.8 {
            continue;
        }

        if let Some(plan) = generate_plan(opportunity) {
            opportunity.refactoring_plan = Some(plan);
            enhanced_count += 1;
        }
    }
}

fn generate_plan(opportunity: &Opportunity) -> Option<String> {
    if opportunity.units.len() < 2 {
        return None;
    }
    
    let unit_a = &opportunity.units[0];
    let unit_b = &opportunity.units[1]; // Compare first two for now

    // 1. Read Files
    let src_a = fs::read_to_string(&unit_a.file).ok()?;
    let src_b = fs::read_to_string(&unit_b.file).ok()?;

    // 2. Parse (Local Parser)
    let ext_a = unit_a.file.extension().and_then(|s| s.to_str())?;
    // Assume same language for duplication
    let lang_a = Lang::from_ext(ext_a)?;
    
    let mut parser = Parser::new();
    if parser.set_language(lang_a.grammar()).is_err() {
        return None;
    }

    let tree_a = parser.parse(&src_a, None)?;
    let tree_b = parser.parse(&src_b, None)?;

    // 3. Find Nodes (approximate by line range)
    let start_point_a = tree_sitter::Point { row: unit_a.start_line.saturating_sub(1), column: 0 };
    let end_point_a = tree_sitter::Point { row: unit_a.end_line.saturating_sub(1), column: 0 };
    
    let start_point_b = tree_sitter::Point { row: unit_b.start_line.saturating_sub(1), column: 0 };
    let end_point_b = tree_sitter::Point { row: unit_b.end_line.saturating_sub(1), column: 0 };

    let node_a = tree_a
        .root_node()
        .named_descendant_for_point_range(start_point_a, end_point_a)?;
        
    let node_b = tree_b
        .root_node()
        .named_descendant_for_point_range(start_point_b, end_point_b)?;

    // 4. Diff
    let model = diff::diff_trees(node_a, src_a.as_bytes(), node_b, src_b.as_bytes())?;

    // 5. Infer Strategies
    let strategies = parameterize::infer_strategies(&model);

    // 6. Generate Plan Text
    let mut full_plan = String::new();
    for strategy in strategies {
        // Use unit_a's file as context for where to put changes (heuristic)
        if let Ok(desc) = codegen::generate_refactor(&strategy, unit_a.file.to_str()?) {
             full_plan.push_str(&desc);
             full_plan.push('\n');
        }
    }

    if full_plan.is_empty() {
        None
    } else {
        Some(full_plan)
    }
}
