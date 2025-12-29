# Past / Present / Future

**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-28  
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.2.x: The Law of Locality was added.**
- Stability Classifier computing fan-in (Cₐ), fan-out (Cₑ), Instability (I), and Skew (K)
- Node identity classification (StableHub, VolatileLeaf, IsolatedDeadwood, GodModule)
- Universal Locality Algorithm validating dependency edges against distance thresholds
- CLI integration via `slopchop scan --locality`

**v1.3.0: Consolidation Era completed.**
- Fixed self-violation: `src/apply/parser.rs` split into `parser.rs` + `blocks.rs`
- Removed 4 commands: `trace`, `fix`, `prompt`, `dashboard`
- Deleted ~2000 lines of unused code (`src/trace/`, `src/tui/`)
- Prescriptive violations: errors now include ANALYSIS and SUGGESTION sections
- Locality scan respects `mode = "warn"` (doesn't block verification pipeline)
- Modularized analysis checks into `checks/naming.rs`, `checks/complexity.rs`, `checks/banned.rs`

---

## 2) Present (Where we are right now)

**Status:** STABLE

SlopChop passes all its own checks. The codebase is clean and consolidated.

### Core Commands

| Command | Purpose |
|---------|---------|
| `scan` | Structural violation detection |
| `check` | Gate (external commands + scan) |
| `apply` | Staged ingestion with XSC7XSC protocol |
| `pack` | AI context generation |
| `clean` | Remove artifacts |

### Experimental Commands

| Command | Purpose |
|---------|---------|
| `scan --locality` | Topological integrity scanning (advisory mode) |
| `audit` | Code duplication detection |
| `map` | Repository visualization |
| `signatures` | Type-surface maps for AI |

### Current Violations

None. `slopchop check` passes.

---

## 3) Future (What we do next)

**The consolidation is complete. SlopChop is stable.**

Potential future work (not prioritized):
- **TypeScript Hardening:** Improve TS import resolution for complex projects
- **Locality Validation:** Run locality in `mode = "error"` once validated on real projects
- **Multi-language:** Python support if a real use case emerges

### Immediate Next Action

**None required.** Ship it, use it, iterate based on real usage.

---

## 4) Non-Goals (What we are NOT doing)

- **Python support:** Not a real use case yet. Depth over breadth.
- **Test coverage enforcement:** Belongs in separate tooling.
- **Advanced visualization:** Dashboard dreams are dead.
- **Method B optimization:** Signatures/map experiments are frozen.
