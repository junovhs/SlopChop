# Past / Present / Future

**Status:** Canonical (living snapshot)  
**Last updated:** 2026-01-03 (v1.3.3)  
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.3.3: Cross-Platform Patch Reliability.**
- Fixed critical CRLF hash flip-flopping bug on Windows that blocked patch workflow.
- Fixed multi-patch verification: only first patch to each file is hash-verified.
- Consolidated to single `compute_sha256()` function with line ending normalization.
- Added `apply_with_options()` to support skip-hash for chained patches.
- Added `test_eol_normalization` and `test_hash_stability` tests.

**v1.3.2: Patch Security & Stress Test Hardening.**
- Fixed critical vulnerabilities: S03 (Null Byte in Path) and I01 (Sigil Injection).
- Verified semantic matcher robustness (W06: Trailing Newline Fallback).
- Strengthened protocol parser with specific prefix binding.
- Systematic stress testing of Categories 1-9 completed.

**v1.3.1: Doc Archival & Verification.**
- Archived v1.3.0 feature proposals and stress tests.
- Bumped version to v1.3.1.
- Verified zero-violation state on the new topology.

**v1.3.0: Locality v2 & Consolidation.**
- **Locality v2:** Cycle detection, auto-hub detection, and layer inference.
- Refactored analysis module to resolve file size and complexity violations.
- Fixed self-violation: `src/apply/parser.rs` split into `parser.rs` + `blocks.rs`
- Removed 4 commands: `trace`, `fix`, `prompt`, `dashboard`
- Deleted ~2000 lines of unused code (`src/trace/`, `src/tui/`)
- Prescriptive violations: errors now include ANALYSIS and SUGGESTION sections
- Modularized analysis checks into `checks/naming.rs`, `checks/complexity.rs`, `checks/banned.rs`

**v1.2.x: The Law of Locality was added.**
- Stability Classifier computing fan-in, fan-out, Instability, and Skew
- Node identity classification (StableHub, VolatileLeaf, IsolatedDeadwood, GodModule)
- Universal Locality Algorithm validating dependency edges
- CLI integration via `slopchop scan --locality`

---

## 2) Present (Where we are right now)

**Status:** STABLE - Patch workflow fully operational

SlopChop passes all its own checks. Hash computation is cross-platform stable. Multi-patch payloads work correctly.

### Core Commands

| Command | Purpose |
|---------|---------|
| `scan` | Structural violation detection |
| `check` | Gate (external commands + scan) |
| `apply` | Staged ingestion with XSC7XSC protocol |
| `pack` | AI context generation |
| `clean` | Remove artifacts |

### Experimental Commands

| Command | Purpose | Notes |
|---------|---------|-------|
| `scan --locality` | Topological integrity scanning | Works but has false positives |
| `audit` | Code duplication detection | |
| `map` | Repository visualization | |
| `signatures` | Type-surface maps for AI | |

### Known Issues

1. **Paste-back packet broken**: When verification fails, the AI feedback message prints but is no longer auto-copied to clipboard. This broke during a refactor and was never restored.

2. **No config UI**: After deleting the bloated TUI dashboard, there's no interactive way to configure SlopChop. Users must edit `slopchop.toml` manually.

---

## 3) Future (What we do next)

### v1.3.4: UX Polish (Immediate Next Action)

| Feature | Description | Effort |
|---------|-------------|--------|
| **Fix paste-back** | Restore auto-copy of AI feedback to clipboard on verification failure | Small |
| **`slopchop config`** | Minimal interactive config editor using crossterm directly (no ratatui) | Medium |
| **Configurable output** | Option to write fix packet to file instead of clipboard | Small |

See `docs/v1.3.4-ux-spec.md` for full technical specification.

### After v1.3.4

| Feature | Description | Priority |
|---------|-------------|----------|
| Locality validation | Run on 3-5 external Rust repos to battle-test heuristics | Medium |
| `mode = "error"` default | Switch locality to blocking mode once validated | Low |
| TypeScript imports | Better path alias and index file resolution | Low |
| Distribution | Scoop, Winget, Homebrew packages | Low |

---

## 4) Non-Goals (What we are NOT doing)

These were considered and deliberately rejected:

| Feature | Reason |
|---------|--------|
| **History/generations** | Stage is ephemeral by design. Users wipe it immediately after promote. No value in tracking generations. |
| **75% PATCH threshold** | Micromanaging. AI can decide when to use PATCH vs FILE. Automatic rejection adds friction without benefit. |
| **META block** | Redundant. BASE_SHA256 per-patch already catches staleness. Belt-and-suspenders paranoia with no real-world failure mode. |
| **Python support** | Not a real use case yet. |
| **Test coverage enforcement** | Separate tooling (cargo-tarpaulin, etc). |
| **Advanced visualization** | Dashboard was bloat. Deleted. |
| **Method B (signatures)** | Frozen experiment. Context windows are large enough now. |

---

## 5) Architecture Notes

### The Paste-Back Loop (Target State)

```
┌─────────────────────────────────────────────────────────────┐
│  1. User runs: slopchop apply -c                            │
│  2. SlopChop writes to stage, runs verification             │
│  3. If verification FAILS:                                  │
│     a. Generate AI feedback packet (violations + context)   │
│     b. Auto-copy to clipboard (or write to file per config) │
│     c. Print: "Paste this back to the AI"                   │
│  4. User pastes to AI, gets fix                             │
│  5. User copies AI response, runs slopchop apply -c again   │
│  6. Repeat until green                                      │
│  7. User runs: slopchop apply --promote                     │
└─────────────────────────────────────────────────────────────┘
```

### The Config Command (Target State)

```
$ slopchop config

┌─ SlopChop Configuration ──────────────────┐
│                                           │
│  Rules                                    │
│  ├─ Max file tokens    [2000]             │
│  ├─ Max complexity     [8]                │
│  ├─ Max nesting        [3]                │
│  └─ Max args           [5]                │
│                                           │
│  Preferences                              │
│  ├─ [x] Auto-copy to clipboard            │
│  ├─ [ ] Write fix packet to file          │
│  ├─ [x] Require PLAN block                │
│  └─ [ ] Auto-promote on green             │
│                                           │
│  [Save]  [Cancel]                         │
└───────────────────────────────────────────┘
```

Minimal TUI using only `crossterm` for input handling. No ratatui, no widgets, no state machine complexity. ~200 lines of code.
