// src/tui/config/helpers.rs
use super::state::ConfigApp;
use crate::config::Theme;

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
    // 6: Auto-Copy
    // 7: Auto-Format
    // 8: Theme (was 10)
    // 9: Progress Bars (was 11)
    // 10: Require Plan (was 12)
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
    let current = if app.rules.max_file_tokens <= 1500 {
        0 // Strict
    } else if app.rules.max_file_tokens <= 2000 {
        1 // Standard
    } else {
        2 // Relaxed
    };

    let next = if forward {
        (current + 1) % 3
    } else {
        (current + 2) % 3
    };

    match next {
        0 => apply_preset(app, 1500, 4, 2),  // Strict
        1 => apply_preset(app, 2000, 8, 3),  // Standard
        2 => apply_preset(app, 3000, 12, 4), // Relaxed
        _ => {}
    }
}

fn apply_preset(app: &mut ConfigApp, tokens: usize, complexity: usize, depth: usize) {
    app.rules.max_file_tokens = tokens;
    app.rules.max_cyclomatic_complexity = complexity;
    app.rules.max_nesting_depth = depth;
}

#[must_use]
pub fn get_active_label(field: usize) -> &'static str {
    match field {
        0 => "GLOBAL PROTOCOL",
        1 => "LAW OF ATOMICITY",
        2..=4 => "LAW OF COMPLEXITY",
        5 => "LAW OF BLUNTNESS",
        6..=7 | 10 => "WORKFLOW AUTOMATION",
        8..=9 => "VISUALS & FEEDBACK",
        _ => "UNKNOWN",
    }
}

const DESCRIPTIONS: &[&str] = &[
    "Select a predefined security clearance level.\n\nStrict: Greenfield/Critical systems.\nStandard: Recommended balance.\nRelaxed: Legacy containment.",
    "Limits file size. Large files confuse AI context windows and make verification impossible. \n\nGoal: Modular, atomic units.",
    "Limits control flow paths. High complexity increases hallucination rates and makes code untestable.\n\nGoal: Linear, obvious logic.",
    "Limits indentation. Deep nesting causes AI to lose scope tracking and context.\n\nGoal: Shallow, flat structures.",
    "Limits function inputs. Too many arguments suggests a missing struct or mixed concerns.\n\nGoal: Clean interfaces.",
    "Limits function naming verbosity. Long names often mask poor abstraction.\n\nGoal: Concise intent.",
    "Automatically copy the generated 'context.txt' to the clipboard.\n\nGoal: Eliminate manual steps.",
    "Run the project's formatter (e.g., cargo fmt, prettier) immediately after applying changes.\n\nGoal: Maintain style guide.",
    // Auto commit removed
    // Commit prefix removed
    "Color scheme for the TUI.\nNASA: High Contrast.\nCyberpunk: Neon.\nCorporate: Subtle.\n\nGoal: Eye Candy.",
    "Show animated progress bars during scans and operations.\n\nGoal: Feedback.",
    "Force AI output to contain a valid PLAN block. Auto-rejects inputs without one.\n\nGoal: Ensure intent is declared before code.",
];

#[must_use]
pub fn get_active_description(field: usize) -> &'static str {
    if field < DESCRIPTIONS.len() {
        DESCRIPTIONS[field]
    } else {
        ""
    }
}

#[must_use]
pub fn detect_preset(app: &ConfigApp) -> &'static str {
    if app.rules.max_file_tokens <= 1500 && app.rules.max_cyclomatic_complexity <= 4 {
        "STRICT"
    } else if app.rules.max_file_tokens >= 3000 {
        "RELAXED"
    } else if app.rules.max_file_tokens == 2000 {
        "STANDARD"
    } else {
        "CUSTOM"
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn get_integrity_score(app: &ConfigApp) -> f64 {
    let t_score = (app.rules.max_file_tokens as f64 - 1000.0) / 3000.0;
    let c_score = (app.rules.max_cyclomatic_complexity as f64 - 1.0) / 15.0;
    let d_score = (app.rules.max_nesting_depth as f64 - 1.0) / 5.0;
    let raw_avg = (t_score + c_score + d_score) / 3.0;
    (1.0 - raw_avg).clamp(0.0, 1.0)
}