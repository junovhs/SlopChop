XSC7XSC PLAN XSC7XSC
GOAL: Close the remaining v1.0.0 “Patch UX” gap by adding a bounded visual diff summary to 0-match PATCH diagnostics, migrate integration PATCH tests off the deprecated v0.9 SEARCH/REPLACE form to the canonical v1 context-anchored format, and add a compact roadmap doc (plus a pointer update in the living snapshot doc).

CHANGES:

1. Add a deterministic, bounded “Diff summary” section to PATCH 0-match diagnostics (no fuzzy apply; locate-only).
2. Update patch diagnostics unit test to assert the diff summary is present.
3. Update integration PATCH tests to use v1 canonical PATCH payloads (BASE_SHA256 + LEFT_CTX/OLD/RIGHT_CTX/NEW).
4. Add docs/roadmap.md (compact), and update docs/past-present-future.md to reference it (instead of the now-oversized v1 brief).
   XSC7XSC END XSC7XSC

XSC7XSC MANIFEST XSC7XSC
src/apply/patch/diagnostics.rs
src/apply/patch/tests.rs
tests/integration_patch.rs
docs/past-present-future.md
docs/roadmap.md [NEW]
XSC7XSC END XSC7XSC

XSC7XSC FILE XSC7XSC src/apply/patch/diagnostics.rs
// src/apply/patch/diagnostics.rs
//! Patch failure diagnostics with "Did you mean?" probe logic.
//!
//! Policy:
//! - Diagnostics may suggest likely regions and show bounded diffs.
//! - Application is still strict: exact match or reject (no fuzzy apply).

use super::common::PatchInstruction;
use std::fmt::Write;

const DIFF_LINE_LIMIT: usize = 8;

pub fn diagnose_zero_matches(content: &str, search: &str, instr: &PatchInstruction) -> String {
let mut msg = String::from("Patch failed: Could not find exact match for the SEARCH block.\n");

``` //note the triple backticks here, you will see them littered all over this document, they are artifacts from chatgpt
// 1) Probe for "Did you mean?"
if let Some(candidate) = find_closest_candidate(content, search) {
    let _ = writeln!(msg, "\n🔎 Did you mean this region?");
    let _ = writeln!(msg, "{}", "-".repeat(40));
    let _ = writeln!(msg, "{}", indent_lines(&candidate, "  "));
    let _ = writeln!(msg, "{}", "-".repeat(40));

    append_diff_summary(&mut msg, search, &candidate);
}

// 2) Context mismatch details
append_mismatch_details(&mut msg, search);

// 3) Additional hint if we have a declared LEFT_CTX
if let Some(l) = &instr.context_left {
    if !content.contains(l.trim()) {
        msg.push_str("\n⚠️  LEFT_CTX was not found in the file.");
    }
}

msg.push_str("\n\nNEXT: Regenerate the patch using the correct context, or send the full FILE.");
msg
```

}

fn indent_lines(block: &str, prefix: &str) -> String {
if block.is_empty() {
return String::new();
}
let mut out = String::new();
for (i, line) in block.lines().enumerate() {
if i > 0 {
out.push('\n');
}
out.push_str(prefix);
out.push_str(line);
}
out
}

fn append_diff_summary(msg: &mut String, expected: &str, candidate: &str) {
let _ = writeln!(
msg,
"\nDiff summary (expected vs candidate, first {DIFF_LINE_LIMIT} lines):"
);

```
let exp: Vec<&str> = expected.lines().take(DIFF_LINE_LIMIT).collect();
let cand: Vec<&str> = candidate.lines().take(DIFF_LINE_LIMIT).collect();

let max = exp.len().max(cand.len()).min(DIFF_LINE_LIMIT);
for i in 0..max {
    let e = exp.get(i).copied().unwrap_or("");
    let c = cand.get(i).copied().unwrap_or("");

    if e == c {
        let _ = writeln!(msg, " {n:>2}  {line}", n = i + 1, line = e);
    } else {
        let _ = writeln!(msg, "-{n:>2}  {line}", n = i + 1, line = e);
        let _ = writeln!(msg, "+{n:>2}  {line}", n = i + 1, line = c);
    }
}

let exp_more = expected.lines().skip(DIFF_LINE_LIMIT).next().is_some();
let cand_more = candidate.lines().skip(DIFF_LINE_LIMIT).next().is_some();
if exp_more || cand_more {
    let _ = writeln!(msg, "... (truncated)");
}
```

}

fn append_mismatch_details(msg: &mut String, search: &str) {
msg.push_str("\n📝 Context mismatch details:\n");
let head = search.lines().next().unwrap_or("???");
let tail = search.lines().last().unwrap_or("???");
let _ = writeln!(msg, "Expected start: '{}'", head.trim());
let _ = writeln!(msg, "Expected end:   '{}'", tail.trim());
}

pub fn find_closest_candidate(content: &str, search: &str) -> Option<String> {
// We need significant enough probes to be useful and safe
let search_chars: Vec<char> = search.chars().collect();
if search_chars.len() < 40 {
return None;
}

```
let head: String = search_chars.iter().take(20).collect();
let tail: String = search_chars.iter().rev().take(20).rev().collect();

let head_matches: Vec<_> = content.match_indices(&head).collect();
let tail_matches: Vec<_> = content.match_indices(&tail).collect();

find_best_match(content, &head_matches, &tail_matches, search.len())
```

}

fn find_best_match(
content: &str,
head_matches: &[(usize, &str)],
tail_matches: &[(usize, &str)],
search_len: usize,
) -> Option<String> {
for (h_idx, _) in head_matches {
for (t_idx, _) in tail_matches {
if *t_idx > *h_idx && is_plausible_match(*h_idx, *t_idx, search_len) {
return Some(extract_candidate(content, *h_idx, *t_idx));
}
}
}
None
}

fn is_plausible_match(h_idx: usize, t_idx: usize, search_len: usize) -> bool {
let dist = t_idx - h_idx;
let expected_dist = search_len.saturating_sub(40); // Rough bytes estimate

```
// Allow wide variance since edits happen (50% tolerance)
let diff = dist.abs_diff(expected_dist);
diff < (expected_dist / 2)
```

}

fn extract_candidate(content: &str, start_idx: usize, end_idx: usize) -> String {
// Expand around matches
let context_start = start_idx.saturating_sub(50);
let context_end = (end_idx + 50).min(content.len());

```
// Ensure we cut at char boundaries
let safe_start = floor_char_boundary(content, context_start);
let safe_end = ceil_char_boundary(content, context_end);

content[safe_start..safe_end].to_string()
```

}

pub fn diagnose_ambiguous(count: usize, matches: &[(usize, &str)], content: &str) -> String {
let mut msg = format!("Patch failed: Ambiguous match. Found {count} occurrences.\n\n");

```
msg.push_str("🔎 Occurrences found at:\n");
for (i, (idx, _)) in matches.iter().enumerate().take(3) {
    let line_num = content[..*idx].lines().count() + 1;
    let safe_end = ceil_char_boundary(content, *idx + 40);
    let snippet = &content[*idx..safe_end].lines().next().unwrap_or("").trim();
    let _ = writeln!(msg, "{}. Line {}: {}...", i + 1, line_num, snippet);
}

if count > 3 {
    msg.push_str("... and others.\n");
}

msg.push_str("\nNEXT: Add more context (LEFT_CTX / RIGHT_CTX) to make the patch unique.");
msg
```

}

fn floor_char_boundary(s: &str, mut idx: usize) -> usize {
if idx >= s.len() {
return s.len();
}
while !s.is_char_boundary(idx) {
idx = idx.saturating_sub(1);
}
idx
}

fn ceil_char_boundary(s: &str, mut idx: usize) -> usize {
if idx >= s.len() {
return s.len();
}
while !s.is_char_boundary(idx) {
idx += 1;
if idx >= s.len() {
return s.len();
}
}
idx
}
XSC7XSC END XSC7XSC

XSC7XSC FILE XSC7XSC src/apply/patch/tests.rs
// src/apply/patch/tests.rs
use super::*;

#[test]
fn test_v1_parse() -> Result<()> {
let patch = "LEFT_CTX:\nfn foo() {\nOLD:\n    print(1);\nRIGHT_CTX:\n}\nNEW:\n    print(2);";
let (instrs, meta) = parse_patch(patch)?;
assert_eq!(instrs.len(), 1);
assert!(matches!(meta.format, PatchFormat::V1));

```
let i = &instrs[0];
// collect_until_keyword adds newline
assert!(i.search.contains("fn foo() {"));
assert!(i.search.contains("print(1);"));
assert!(i.replace.contains("print(2);"));
Ok(())
```

}

#[test]
fn test_v1_apply() -> Result<()> {
let original = "fn foo() {\n    print(1);\n}\n";
let hash = compute_sha256(original);

```
// V1 requires BASE_SHA256
let patch = format!(
    "BASE_SHA256: {hash}\nLEFT_CTX:\nfn foo() {{\nOLD:\n    print(1);\nRIGHT_CTX:\n}}\nNEW:\n    print(2);"
);

let res = apply(original, &patch)?;
assert_eq!(res, "fn foo() {\n    print(2);\n}\n");
Ok(())
```

}

#[test]
fn test_diagnostic_ambiguous() {
let original = "repeat\nrepeat\nrepeat";
let patch = "<<<< SEARCH\nrepeat\n====\nfixed\n>>>>";
let err = apply(original, patch).unwrap_err();
assert!(err.to_string().contains("Found 3 occurrences"));
assert!(err.to_string().contains("Line 1"));
}

#[test]
fn test_diagnostic_zero_match_probe() {
// Original has long enough context to trigger probe logic
let original = "fn main() { // start context is sufficiently long to be unique and detectable\n    let x = 100000000000;\n} // end context must also be long enough to be found";

```
// Patch expects x = 5, but we ensure context is >20 chars so probe finds it
let left = "fn main() { // start context is sufficiently long to be unique and detectable\n    let x = ";
let right = ";\n} // end context must also be long enough to be found";

// The search string must be > 40 chars
let search = format!("{left}5555555555555555{right}");
let replace = format!("{left}2{right}");

// Manual instruction construction to test diagnostic logic directly
let instr = PatchInstruction {
    search: search.clone(),
    replace,
    context_left: Some(left.to_string()),
};

let diag = diagnose_zero_matches(original, &search, &instr);

assert!(diag.contains("Did you mean this region?"));
assert!(diag.contains("Diff summary"));
assert!(diag.contains("100000000000"));
```

}
XSC7XSC END XSC7XSC

XSC7XSC FILE XSC7XSC tests/integration_patch.rs
// tests/integration_patch.rs
// slopchop:ignore (test file - allowed .unwrap() for test clarity)
use slopchop_core::apply::types::ApplyOutcome;
use slopchop_core::apply::process_input;
use slopchop_core::stage::StageManager;
use std::fs;
use tempfile::TempDir;

use sha2::{Digest, Sha256};

const SIGIL: &str = "XSC7XSC";

fn setup_env() -> anyhow::Result<(TempDir, String)> {
let env = TempDir::new()?;
let root = env.path().to_string_lossy().to_string();
Ok((env, root))
}

fn apply(root: &std::path::Path, payload: &str) -> anyhow::Result<ApplyOutcome> {
process_input(root, payload, false, false)
}

fn sha256_hex(bytes: &[u8]) -> String {
let mut hasher = Sha256::new();
hasher.update(bytes);
let out = hasher.finalize();
format!("{out:x}")
}

#[test]
fn test_patch_success() -> anyhow::Result<()> {
let (env, _) = setup_env()?;
let root = env.path();

```
// 1) Base file
let base_payload = format!(
    r#"{SIGIL} MANIFEST {SIGIL}
```

src/target.rs [NEW]
{SIGIL} END {SIGIL}

{SIGIL} FILE {SIGIL} src/target.rs
fn main() {{
println!("Old");
}}
{SIGIL} END {SIGIL}
"#
);
apply(root, &base_payload)?;

```
// Compute BASE_SHA256 from staged bytes
let stage = StageManager::new(root);
let path = stage.worktree().join("src/target.rs");
let bytes = fs::read(&path)?;
let base_sha = sha256_hex(&bytes);

// 2) Canonical v1 PATCH payload
let patch_payload = format!(
    r#"{SIGIL} MANIFEST {SIGIL}
```

src/target.rs
{SIGIL} END {SIGIL}

{SIGIL} PATCH {SIGIL} src/target.rs
BASE_SHA256: {base_sha}
MAX_MATCHES: 1
LEFT_CTX:
fn main() {{
OLD:
println!("Old");
RIGHT_CTX:
}}
NEW:
println!("New");
{SIGIL} END {SIGIL}
"#
);
apply(root, &patch_payload)?;

```
// 3) Verify staged content
let content = fs::read_to_string(path)?;
if !content.contains("println!(\"New\");") {
    eprintln!("CONTENT WAS:\n{content}");
}

assert!(content.contains("println!(\"New\");"));
assert!(!content.contains("println!(\"Old\");"));
Ok(())
```

}

#[test]
fn test_patch_reject_ambiguous() -> anyhow::Result<()> {
let (env, _) = setup_env()?;
let root = env.path();

```
// 1) Setup ambiguous file
let base_payload = format!(
    r"{SIGIL} MANIFEST {SIGIL}
```

ambig.rs [NEW]
{SIGIL} END {SIGIL}

{SIGIL} FILE {SIGIL} ambig.rs
repeat
repeat
{SIGIL} END {SIGIL}
"
);
apply(root, &base_payload)?;

```
// Compute BASE_SHA256 from staged bytes
let stage = StageManager::new(root);
let path = stage.worktree().join("ambig.rs");
let bytes = fs::read(&path)?;
let base_sha = sha256_hex(&bytes);

// 2) Attempt ambiguous v1 patch (anchor occurs twice)
let patch_payload = format!(
    r"{SIGIL} MANIFEST {SIGIL}
```

ambig.rs
{SIGIL} END {SIGIL}

{SIGIL} PATCH {SIGIL} ambig.rs
BASE_SHA256: {base_sha}
MAX_MATCHES: 1
LEFT_CTX:
OLD:
repeat
RIGHT_CTX:
NEW:
fixed
{SIGIL} END {SIGIL}
"
);

```
let result = apply(root, &patch_payload)?;
match result {
    ApplyOutcome::ParseError(msg) => {
        assert!(msg.contains("Ambiguous"), "Expected ambiguous error, got: {msg}");
    }
    _ => panic!("Expected ParseError due to ambiguity"),
}
Ok(())
```

}

#[test]
fn test_patch_sha256_verification() -> anyhow::Result<()> {
let (env, _) = setup_env()?;
let root = env.path();

```
// 1) Setup file
let base_payload = format!(
    r"{SIGIL} MANIFEST {SIGIL}
```

secure.rs [NEW]
{SIGIL} END {SIGIL}

{SIGIL} FILE {SIGIL} secure.rs
secret_data
{SIGIL} END {SIGIL}
"
);
apply(root, &base_payload)?;

```
// 2) Attempt v1 patch with WRONG SHA
let patch_payload = format!(
    r"{SIGIL} MANIFEST {SIGIL}
```

secure.rs
{SIGIL} END {SIGIL}

{SIGIL} PATCH {SIGIL} secure.rs
BASE_SHA256: badhash123
MAX_MATCHES: 1
LEFT_CTX:
OLD:
secret_data
RIGHT_CTX:
NEW:
exposed
{SIGIL} END {SIGIL}
"
);

```
let result = apply(root, &patch_payload)?;
match result {
    ApplyOutcome::ParseError(msg) => {
        assert!(
            msg.contains("Base SHA256 verification failed"),
            "Expected SHA verification failure, got: {msg}"
        );
    }
    _ => panic!("Expected ParseError due to base SHA mismatch"),
}
Ok(())
```

}
XSC7XSC END XSC7XSC

XSC7XSC FILE XSC7XSC docs/past-present-future.md

# Past / Present / Future

**Status:** Canonical (living snapshot)
**Last updated:** 2025-12-25
**Canonical policy:** This document states the current operational reality and the single next action.
**For the roadmap and historical milestones:** see `/docs/roadmap.md` (and `/docs/archived/` for immutable snapshots).

---

## 1) Past (What changed recently)

**v1.0.0 Released.**

* **Trust Boundary:** Complete staging, verification, and promotion architecture.
* **Protocol:** Canonical `XSC7XSC` with context-anchored patching and SHA256 locking.
* **Observability:** Structured event logging and standardized exit codes.
* **Hygiene:** Zero Git dependency; clean 3 Laws compliance.

**Phase 3 (Polish) complete.**

* **Exit Codes:** Standardized `SlopChopExit`.
* **Event Log:** `events.jsonl` audit trail.
* **Cleanup:** Removed deprecated Git configs.

---

## 2) Present (Where we are right now)

**Status:** STABLE / MAINTENANCE

### Operator-visible contract

* `slopchop apply` (Stage)
* `slopchop check` (Verify)
* `slopchop apply --promote` (Commit)
* `slopchop pack` (Context)

### Trust boundary posture

* System is now a self-contained, high-integrity gatekeeper.

---

## 3) Future (What we do next)

We are in **Post-v1.0 Era**.

### Objectives

* Monitor stability.
* Distribution (Homebrew/Scoop/Winget packaging).
* User feedback loop.

### Immediate Next Action

**Celebrate.** (And then maybe `git tag v1.0.0`).
XSC7XSC END XSC7XSC

XSC7XSC FILE XSC7XSC docs/roadmap.md

# SlopChop Roadmap (Compact)

**Status:** v1.0.0 shipped (trust boundary complete).
**Last updated:** 2025-12-25.

This file is intentionally compact. For immutable historical snapshots, see `/docs/archived/`.

---

## v1.0.0 — Production Trust Boundary (Shipped)

### Patch UX (hardened)

* [x] Canonical context-anchored PATCH format (BASE_SHA256 + LEFT_CTX/OLD/RIGHT_CTX/NEW)
* [x] “Did you mean?” diagnostics (locate-only; never fuzzy-apply)
* [x] Bounded visual diff summary on 0-match failures
* [x] Strict ambiguity rejection with bounded match excerpts

### Observability & contract

* [x] Machine-readable event log (`.slopchop/events.jsonl`, NDJSON/JSONL)
* [x] Stable exit code taxonomy (scriptable)

### Repo safety model

* [x] Stage-only writes by default
* [x] Stage-aware `check` and `pack`
* [x] Transactional promote (touched-path scoped, backup + rollback)

---

## Post-v1.0 — Deliberately Deferred

* [ ] Distribution packaging: Homebrew / Scoop / Winget
* [ ] Additional ergonomics (optional): richer paste-back packets, advanced installer automation
  XSC7XSC END XSC7XSC
