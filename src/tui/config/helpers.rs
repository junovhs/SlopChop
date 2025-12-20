// src/tui/config/helpers.rs
use super::state::ConfigApp;
use crate::config::Theme;

#[derive(Clone, Copy)]
struct Preset {
    tokens: usize,
    complexity: usize,
    depth: usize,
}

const PRESETS: [Preset; 3] = [
    Preset { tokens: 1500, complexity: 4, depth: 2 }, // Strict
    Preset { tokens: 2000, complexity: 8, depth: 3 }, // Standard
    Preset { tokens: 3000, complexity: 12, depth: 4 }, // Relaxed
];

pub fn adjust_rule(app: &mut ConfigApp, increase: bool) {
    match app.selected_field {
        1 => adjust_int(&mut app.rules.max_file_tokens, 100, 100, increase),
        2 => adjust_int(&mut app.rules.max_cyclomatic_complexity, 1, 1, increase),
        3 => adjust_int(&mut app.rules.max_nesting_depth, 1, 1, increase),
        4 => adjust_int(&mut app.rules.max_function_args, 1, 1, increase),
        5 => adjust_int(&mut app.rules.max_function_words, 1, 1, increase),
        _ => {}
    }
}

pub fn adjust_pref(app: &mut ConfigApp, increase: bool) {
    // Indices shifted due to removal of auto_commit/push/prefix
    // 6: Auto-Copy, 7: Auto-Format, 8: Theme, 9: Progress, 10: RequirePlan
    if matches!(app.selected_field, 6 | 7 | 9 | 10) {
        toggle_pref(app);
    } else {
        cycle_pref(app, increase);
    }
}

fn toggle_pref(app: &mut ConfigApp) {
    match app.selected_field {
        6 => app.preferences.auto_copy = !app.preferences.auto_copy,
        7 => app.preferences.auto_format = !app.preferences.auto_format,
        9 => app.preferences.progress_bars = !app.preferences.progress_bars,
        10 => app.preferences.require_plan = !app.preferences.require_plan,
        _ => {}
    }
}

fn cycle_pref(app: &mut ConfigApp, increase: bool) {
    if app.selected_field == 8 {
        cycle_theme(app, increase);
    }
}

fn adjust_int(val: &mut usize, step: usize, min: usize, increase: bool) {
    if increase {
        *val = val.saturating_add(step);
    } else {
        *val = val.saturating_sub(step).max(min);
    }
}

fn cycle_theme(app: &mut ConfigApp, forward: bool) {
    let themes = [Theme::Cyberpunk, Theme::Nasa, Theme::Corporate];
    let current = themes
        .iter()
        .position(|t| *t == app.preferences.theme)
        .unwrap_or(0);
    let next = if forward {
        (current + 1) % 3
    } else {
        (current + 2) % 3
    };
    app.preferences.theme = themes[next];
}

pub fn cycle_preset(app: &mut ConfigApp, forward: bool) {
    let current = detect_preset_index(app);
    let next_idx = if forward {
        (current + 1) % 3
    } else {
        (current + 2) % 3
    };

    let p = PRESETS[next_idx];
    app.rules.max_file_tokens = p.tokens;
    app.rules.max_cyclomatic_complexity = p.complexity;
    app.rules.max_nesting_depth = p.depth;
}

fn detect_preset_index(app: &ConfigApp) -> usize {
    if app.rules.max_file_tokens <= 1500 {
        0 // Strict
    } else if app.rules.max_file_tokens <= 2000 {
        1 // Standard
    } else {
        2 // Relaxed
    }
}

#[derive(Clone, Copy)]
struct FieldInfo {
    label: &'static str,
    desc: &'static str,
}

const FIELD_INFOS: [FieldInfo; 11] = [
    FieldInfo { label: "GLOBAL PROTOCOL", desc: "Select a predefined security clearance level.\n\nStrict: Greenfield/Critical systems.\nStandard: Recommended balance.\nRelaxed: Legacy containment." },
    FieldInfo { label: "LAW OF ATOMICITY", desc: "Limits file size. Large files confuse AI context windows and make verification impossible. \n\nGoal: Modular, atomic units." },
    FieldInfo { label: "LAW OF COMPLEXITY", desc: "Limits control flow paths. High complexity increases hallucination rates and makes code untestable.\n\nGoal: Linear, obvious logic." },
    FieldInfo { label: "LAW OF COMPLEXITY", desc: "Limits indentation. Deep nesting causes AI to lose scope tracking and context.\n\nGoal: Shallow, flat structures." },
    FieldInfo { label: "LAW OF COMPLEXITY", desc: "Limits function inputs. Too many arguments suggests a missing struct or mixed concerns.\n\nGoal: Clean interfaces." },
    FieldInfo { label: "LAW OF BLUNTNESS", desc: "Limits function naming verbosity. Long names often mask poor abstraction.\n\nGoal: Concise intent." },
    FieldInfo { label: "WORKFLOW AUTOMATION", desc: "Automatically copy the generated 'context.txt' to the clipboard.\n\nGoal: Eliminate manual steps." },
    FieldInfo { label: "WORKFLOW AUTOMATION", desc: "Run the project's formatter (e.g., cargo fmt, prettier) immediately after applying changes.\n\nGoal: Maintain style guide." },
    FieldInfo { label: "VISUALS & FEEDBACK", desc: "Color scheme for the TUI.\nNASA: High Contrast.\nCyberpunk: Neon.\nCorporate: Subtle.\n\nGoal: Eye Candy." },
    FieldInfo { label: "VISUALS & FEEDBACK", desc: "Show animated progress bars during scans and operations.\n\nGoal: Feedback." },
    FieldInfo { label: "WORKFLOW AUTOMATION", desc: "Force AI output to contain a valid PLAN block. Auto-rejects inputs without one.\n\nGoal: Ensure intent is declared before code." },
];

#[must_use]
pub fn get_active_label(field: usize) -> &'static str {
    FIELD_INFOS.get(field).map_or("UNKNOWN", |i| i.label)
}

#[must_use]
pub fn get_active_description(field: usize) -> &'static str {
    FIELD_INFOS.get(field).map_or("", |i| i.desc)
}

#[must_use]
pub fn detect_preset(app: &ConfigApp) -> &'static str {
    match detect_preset_index(app) {
        0 => "STRICT",
        1 => "STANDARD",
        2 => "RELAXED",
        _ => "CUSTOM",
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn get_integrity_score(app: &ConfigApp) -> f64 {
    let t = score_metric(app.rules.max_file_tokens as f64, 1000.0, 3000.0);
    let c = score_metric(app.rules.max_cyclomatic_complexity as f64, 1.0, 15.0);
    let d = score_metric(app.rules.max_nesting_depth as f64, 1.0, 5.0);
    
    let raw = (t + c + d) / 3.0;
    (1.0 - raw).clamp(0.0, 1.0)
}

fn score_metric(val: f64, min: f64, range: f64) -> f64 {
    (val - min) / range
}