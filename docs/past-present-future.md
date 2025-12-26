# Past / Present / Future
**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-25  
**Canonical policy:** This document states the current operational reality and the single next action.  
**For full v1.0 priorities / blockers / DoD:** see `/docs/v1-brief.md`.

---

## 1) Past (What changed recently)

**Phase 3A (Patch UX & Diagnostics) complete.**
- **Refactored Patch Engine:** Split `src/apply/patch.rs` into atomic submodules (`parser_v1`, `diagnostics`, etc.).
- **Canonical V1:** Enforced `BASE_SHA256` and `MAX_MATCHES: 1`.
- **Diagnostics:** Added "Did you mean?" probe logic and ambiguous match excerpts.
- **Safety:** Removed global replace in favor of single-occurrence splicing.
- **Compliance:** Satisfied 3 Laws (Atomicity, Complexity, Paranoia).

**v0.9.0 shipped.** The architectural pivot is complete and validated:
- **Staged workspace:** `slopchop apply` writes to `.slopchop/stage/worktree`.
- **Transactional promote:** `slopchop apply --promote` promotes verified changes.

---

## 2) Present (Where we are right now)

**Status:** OPERATIONAL / GREEN

### Operator-visible contract
- `slopchop apply` supports V1 (Context-Anchored) and V0 (Search/Replace legacy).
- Diagnostics are deterministic and bounded.
- `slopchop check` is passing (clippy, tests, scan).

### Trust boundary posture
- Patching is strictly anchored and hashed.
- Failures do not modify state.
- Ambiguity is rejected with guidance.

---

## 3) Future (What we do next)

We are in **Phase 3 (Polish) → v1.0.0**.

### Phase 3B: CLI Polish & Event Log (NEXT OBJECTIVE)
To reach v1.0, we need observability and automation stability.
- **Standardize Exit Codes:** Documented, stable codes for success, check-fail, patch-fail, etc.
- **Machine-Readable Events:** Emit `events.jsonl` for audit trails (apply, check, promote).
- **Surface Cleanup:** Remove any remaining Git-related code or config keys.

(Full spec and priorities live in `/docs/v1-brief.md`.)

---

## Immediate Next Action
Begin **Phase 3B (CLI Polish & Event Log)**.