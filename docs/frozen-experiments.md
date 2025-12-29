# Frozen Experiments

**Status:** Reference document  
**Created:** 2025-12-28  
**Purpose:** Preserve context on deprioritized features so future work doesn't repeat mistakes or lose insights.

---

## Overview

SlopChop development explored several experimental directions that were frozen (not deleted, not finished - just paused). This document explains what they were, why they were tried, and why they're on hold.

---

## 1. Method B: Type-Surface Context for AI

### What It Was

An alternative to `slopchop pack` that generates minimal context by extracting only function signatures, types, and public APIs - not full source code.

The idea: AI doesn't need to see every line. It needs to understand the *shape* of the codebase. A 50KB pack file could become 5KB of pure signal.

### Implementation

- `slopchop signatures` - Extracts function signatures, struct definitions, trait bounds
- Uses tree-sitter to parse and extract only declaration nodes
- Outputs a condensed "type surface" of the codebase

### Why It Was Frozen

1. **Premature optimization.** `slopchop pack` works fine. Context windows are getting larger. The problem Method B solves may not be a real problem.

2. **Incomplete extraction.** Getting *useful* signatures requires understanding:
   - Which functions are public vs private
   - What types flow across module boundaries
   - Documentation comments (or not?)
   
   This is language-specific and fiddly.

3. **Unknown ROI.** No evidence that AI performs better with signatures vs full code. Would need testing to validate the hypothesis.

### When To Unfreeze

- If context windows become constrained again
- If working on very large codebases where pack produces 500KB+ files
- If someone wants to research AI comprehension of signatures vs source

### Current State

`slopchop signatures` command exists and runs. It's marked `[EXPERIMENTAL]`. It produces output but hasn't been validated for usefulness.

---

## 2. Map: Repository Visualization

### What It Was

A visual representation of the repository structure with optional dependency tracking.

```
$ slopchop map --deps

src/
  |-- apply/
  |     |-- mod.rs (2.1 KB, 450 toks)
  |     |     `-- clipboard/mod.rs
  |     |     `-- stage/mod.rs
  |     `-- parser.rs (1.8 KB, 380 toks)
```

### Why It Exists

Useful for:
- Quick codebase orientation
- Seeing which files depend on what
- Identifying files that import from far away (with `[FAR]` markers)

### Why It Was Frozen (as an experiment)

It works. It's just not clear how often it's useful vs `tree` or IDE features.

The dependency visualization overlaps with `scan --locality` which does deeper analysis.

### When To Unfreeze

- If users report finding it valuable
- If it becomes the foundation for a richer visualization (e.g., interactive TUI)

### Current State

`slopchop map` command exists and works. Marked `[EXPERIMENTAL]`. Low maintenance burden - can stay as-is indefinitely.

---

## 3. Trace: Dependency Tracing

### What It Was

Given a file, show everything it depends on and everything that depends on it. Fan-in, fan-out, transitive closure.

```
$ slopchop trace src/apply/mod.rs

INBOUND (who depends on this):
  src/cli/handlers.rs
  src/bin/slopchop.rs

OUTBOUND (what this depends on):
  src/clipboard/mod.rs
  src/stage/mod.rs
  src/config/mod.rs
```

### Why It Was Deleted (Not Just Frozen)

1. **Redundant with `pack --focus`.** The `--focus` flag already does "give me this file and its dependencies."

2. **Redundant with `scan --locality`.** Locality analysis provides richer dependency insights (distance, hub status, coupling metrics).

3. **Maintenance burden.** Kept the code around but nobody used it.

### What Was Preserved

The useful parts were extracted into `src/map.rs` which powers `slopchop map --deps`. The trace-specific code was deleted.

---

## 4. Dashboard: TUI Interface

### What It Was

A terminal UI (using `ratatui`) that showed:
- Real-time violation counts
- File watcher for auto-rescan
- Config editor
- Apply workflow visualization

### Why It Was Deleted

1. **Overengineered.** Built UI before validating that anyone wanted a TUI.

2. **High maintenance.** TUI code is fiddly. Every CLI change required updating the dashboard.

3. **Low usage.** The CLI workflow (`scan`  `apply`  `check`) is fast enough. Nobody complained about needing a dashboard.

4. **Feature creep.** Dashboard became a dumping ground for "nice to have" features that didn't belong in core.

### What Was Preserved

Nothing. The entire `src/tui/` directory was deleted. If a TUI is ever wanted, rebuild from scratch with clearer requirements.

### Lesson Learned

Don't build UI until you've validated the core workflow. CLI-first, TUI-maybe-later.

---

## 5. General Principles Applied

### Why Features Get Frozen

1. **No clear user need.** Built because "cool" not because "necessary."
2. **Maintenance burden exceeds value.** Code exists, works, but drags down velocity.
3. **Overlap with other features.** Two features solving same problem = delete one.
4. **Premature optimization.** Solving problems that don't exist yet.

### How Features Get Unfrozen

1. **Real user request.** Someone actually wants it.
2. **Clear use case.** Can articulate when/why it's valuable.
3. **Low lift to finish.** Mostly done, just needs polish.
4. **Strategic fit.** Aligns with where SlopChop is going.

### The Default

When in doubt, freeze (don't delete). Code in a frozen state:
- Doesn't break anything
- Doesn't require maintenance
- Preserves optionality
- Can be revived if needed

Delete only when the code actively hurts (maintenance burden, confusion, redundancy).

---

## Summary Table

| Experiment | Status | Rationale |
|------------|--------|-----------|
| `signatures` (Method B) | Frozen | Premature optimization, unknown ROI |
| `map` | Frozen (works) | Low value, overlaps with locality |
| `trace` | Deleted | Redundant with pack --focus and locality |
| `dashboard` | Deleted | Overengineered, no user demand |

---

## For Future Developers

If you're considering reviving any of these:

1. **Read this document first.** Understand why it was frozen.
2. **Articulate the use case.** Who wants it? When would they use it?
3. **Start minimal.** Don't rebuild the whole thing. Add the smallest useful slice.
4. **Validate early.** Ship something, see if it gets used.

If you're considering adding a new experimental feature:

1. **Mark it `[EXPERIMENTAL]`** in the CLI help.
2. **Document the hypothesis.** What problem does it solve? How will you know if it worked?
3. **Set a review date.** Freeze or ship within N weeks.
4. **Bias toward deletion.** If uncertain, delete. You can always rebuild.