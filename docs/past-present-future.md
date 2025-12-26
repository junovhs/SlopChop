# Past / Present / Future
**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-25  
**Canonical policy:** This document states the current operational reality and the single next action.  
**For full v1.0 priorities / blockers / DoD:** see `/docs/v1-brief.md`.

---

## 1) Past (What changed recently)

**v1.0.0 Released.**
- **Trust Boundary:** Complete staging, verification, and promotion architecture.
- **Protocol:** Canonical `XSC7XSC` with context-anchored patching and SHA256 locking.
- **Observability:** Structured event logging and standardized exit codes.
- **Hygiene:** Zero Git dependency; clean 3 Laws compliance.

**Phase 3 (Polish) complete.**
- **Exit Codes:** Standardized `SlopChopExit`.
- **Event Log:** `events.jsonl` audit trail.
- **Cleanup:** Removed deprecated Git configs.

---

## 2) Present (Where we are right now)

**Status:** STABLE / MAINTENANCE

### Operator-visible contract
- `slopchop apply` (Stage)
- `slopchop check` (Verify)
- `slopchop apply --promote` (Commit)
- `slopchop pack` (Context)

### Trust boundary posture
- System is now a self-contained, high-integrity gatekeeper.

---

## 3) Future (What we do next)

We are in **Post-v1.0 Era**.

### Objectives
- Monitor stability.
- Distribution (Homebrew/Scoop/Winget packaging).
- User feedback loop.

### Immediate Next Action
**Celebrate.** (And then maybe `git tag v1.0.0`).