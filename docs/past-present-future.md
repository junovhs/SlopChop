# Past / Present / Future

**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-28  
**Canonical policy:** This document states the current operational reality and the single next action.

---

## 1) Past (What changed recently)

**v1.2.x: The Law of Locality was added.**
- Stability Classifier computing fan-in (C?), fan-out (C?), Instability (I), and Skew (K)
- Node identity classification (StableHub, VolatileLeaf, IsolatedDeadwood, GodModule)
- Universal Locality Algorithm validating dependency edges against distance thresholds
- CLI integration via `slopchop scan --locality`

**v1.3.x: Consolidation Phase began.**
- Fixed self-violation: `src/apply/parser.rs` split into `parser.rs` + `blocks.rs`
- Fixed locality enforcement: `mode = "warn"` now properly skips blocking in verification pipeline
- `slopchop check` now passes on the SlopChop codebase itself

---

## 2) Present (Where we are right now)

**Status:** CONSOLIDATION PHASE (In Progress)

### Completed
- ? `src/apply/parser.rs` is now under 2000 tokens
- ? Locality scan respects `mode = "warn"` (doesn't block verification)
- ? `slopchop check` passes

### Core Commands (Battle-Tested)

| Command | Purpose | Status |
|---------|---------|--------|
| `scan` | Structural violation detection | ? Stable |
| `check` | Gate (external commands + scan) | ? Stable |
| `apply` | Staged ingestion with XSC7XSC protocol | ? Stable |
| `pack` | AI context generation | ? Stable |

### Experimental Commands (Keep but Mark)

| Command | Purpose | Status |
|---------|---------|--------|
| `scan --locality` | Topological integrity scanning | ?? Advisory mode only |
| `signatures` | Type-surface maps for AI | ?? Experimental |
| `map` | Repository visualization | ?? Experimental |
| `audit` | Code duplication detection | ?? Experimental |

### Commands to Remove (Not Yet Done)

| Command | Reason |
|---------|--------|
| `trace` | Redundant with `pack --focus` |
| `fix` | Unused, modifies files unpredictably |
| `prompt` | Not core, can be done via `pack --prompt` |
| `dashboard` | Unused except for config; over-promised visualization |

---

## 3) Future (What we do next)

**Remaining Consolidation Tasks:**

1. ~~**Self-Compliance:** Fix all violations in SlopChop itself~~ ? Done
2. **Surface Reduction:** Remove `trace`, `fix`, `prompt`, `dashboard` commands
3. **Feature Absorption:** Merge useful `trace` concepts into `pack` (trace can just be deleted, nothing worth saving)
4. ~~**Locality Stabilization:** Set `mode = "warn"` as default~~ ? Done (was already default)
5. **Prescriptive Violations:** Make error messages actionable, not just descriptive
6. **TypeScript Hardening:** Improve TS import resolution for real-world projects

### Immediate Next Action

**Remove `trace`, `fix`, `prompt`, `dashboard` commands from the CLI.**

This requires:
- Update `src/lib.rs` - remove `trace` and `tui` modules
- Update `src/cli/args.rs` - remove command variants
- Update `src/cli/handlers.rs` - remove handlers
- Update `src/cli/mod.rs` - remove exports
- Create `src/map.rs` - extract map functionality from trace
- Delete `src/trace/` directory
- Delete `src/tui/` directory

---

## 4) Non-Goals (What we are NOT doing)

- **Python support:** Not a real use case yet. Depth over breadth.
- **Test coverage enforcement:** Belongs in the separate Roadmap project.
- **Advanced visualization:** Dashboard dreams are deferred indefinitely.
- **Method B optimization:** Signatures/trace experiments are frozen, not expanded.