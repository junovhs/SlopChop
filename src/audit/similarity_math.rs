// src/audit/similarity_math.rs
//! Math and scoring logic for similarity detection.

use super::types::{CodeUnit, CodeUnitKind, Fingerprint};
use std::collections::HashSet;

pub const SIMILARITY_THRESHOLD: f64 = 0.92;
pub const TRIVIAL_SIMILARITY_THRESHOLD: f64 = 0.97;
pub const MIN_FINGERPRINT_SIMILARITY: f64 = 0.6;

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_fingerprint_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    if a.hash == b.hash {
        return 1.0;
    }
    calculate_fuzzy_similarity(a, b)
}

#[allow(clippy::cast_precision_loss)]
fn calculate_fuzzy_similarity(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let base_score = structural_score(a, b);

    if a.cfg_hash == b.cfg_hash {
        return 0.85 + (base_score * 0.15);
    }

    if cfg_metrics_match(a, b) {
        return 0.95;
    }

    weighted_cfg_score(a, b, base_score)
}

fn weighted_cfg_score(a: &Fingerprint, b: &Fingerprint, base_score: f64) -> f64 {
    let cfg_val = cfg_score(a, b);
    cfg_val * 0.6 + base_score * 0.4
}

fn cfg_metrics_match(a: &Fingerprint, b: &Fingerprint) -> bool {
    a.branch_count == b.branch_count
        && a.loop_count == b.loop_count
        && a.exit_count == b.exit_count
}

#[allow(clippy::cast_precision_loss)]
fn cfg_score(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let branch = diff_ratio(a.branch_count, b.branch_count);
    let loops = diff_ratio(a.loop_count, b.loop_count);
    let exits = diff_ratio(a.exit_count, b.exit_count);
    branch * 0.5 + loops * 0.3 + exits * 0.2
}

#[allow(clippy::cast_precision_loss)]
fn structural_score(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let depth = diff_ratio(a.depth, b.depth);
    let nodes = diff_ratio(a.node_count, b.node_count);
    depth * 0.3 + nodes * 0.7
}

#[allow(clippy::cast_precision_loss)]
fn diff_ratio(a: usize, b: usize) -> f64 {
    let max = a.max(b) as f64;
    if max == 0.0 {
        1.0
    } else {
        1.0 - (a as f64 - b as f64).abs() / max
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn calculate_unit_similarity(a: &CodeUnit, b: &CodeUnit, fp_sim: f64) -> f64 {
    let line_sim = diff_ratio(a.line_count(), b.line_count());
    let tok_sim = diff_ratio(a.tokens, b.tokens);
    line_sim * 0.1 + tok_sim * 0.2 + fp_sim * 0.7
}

#[must_use]
pub fn are_units_similar(a: &CodeUnit, b: &CodeUnit) -> bool {
    if !preliminary_checks(a, b) {
        return false;
    }

    let fp_sim = calculate_fingerprint_similarity(&a.fingerprint, &b.fingerprint);
    if fp_sim < MIN_FINGERPRINT_SIMILARITY {
        return false;
    }

    let total_sim = calculate_unit_similarity(a, b, fp_sim);
    total_sim >= get_threshold(a, b)
}

fn preliminary_checks(a: &CodeUnit, b: &CodeUnit) -> bool {
    a.kind == b.kind && passes_enum_gate(a, b)
}

fn passes_enum_gate(a: &CodeUnit, b: &CodeUnit) -> bool {
    if a.kind != CodeUnitKind::Enum || b.kind != CodeUnitKind::Enum {
        return true;
    }
    check_enum_overlap(&a.signature, &b.signature)
}

fn check_enum_overlap(sig_a: &[String], sig_b: &[String]) -> bool {
    let (intersection, min_len) = compute_intersection_stats(sig_a, sig_b);
    evaluate_overlap_threshold(intersection, min_len)
}

fn compute_intersection_stats(sig_a: &[String], sig_b: &[String]) -> (usize, usize) {
    let set_a = to_normalized_set(sig_a);
    let set_b = to_normalized_set(sig_b);
    let intersection = set_a.intersection(&set_b).count();
    let min_len = set_a.len().min(set_b.len());
    (intersection, min_len)
}

fn to_normalized_set(sig: &[String]) -> HashSet<String> {
    sig.iter().map(|s| s.to_lowercase()).collect()
}

#[allow(clippy::cast_precision_loss)]
fn evaluate_overlap_threshold(intersection: usize, min_len: usize) -> bool {
    if min_len == 0 {
        return false;
    }
    if min_len <= 3 {
        return check_small_enum_overlap(intersection, min_len);
    }
    (intersection as f64 / min_len as f64) >= 0.5
}

fn check_small_enum_overlap(intersection: usize, min_len: usize) -> bool {
    if min_len <= 2 {
        intersection == min_len
    } else {
        intersection >= 2
    }
}

fn get_threshold(a: &CodeUnit, b: &CodeUnit) -> f64 {
    if is_complex(a) && is_complex(b) {
        SIMILARITY_THRESHOLD
    } else {
        TRIVIAL_SIMILARITY_THRESHOLD
    }
}

fn is_complex(unit: &CodeUnit) -> bool {
    let fp = &unit.fingerprint;
    fp.branch_count > 0 || fp.loop_count > 0 || fp.exit_count > 0
}