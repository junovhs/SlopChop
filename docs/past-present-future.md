# Past / Present / Future
**Status:** Canonical (living snapshot)  
**Last updated:** 2025-12-25  
**Canonical policy:** This document states the current operational reality and the single next action.  
**For full v1.0 priorities / blockers / DoD:** see `/docs/v1-brief.md`.

---

## 1) Past (What changed recently)

**Phase 3 (Polish) complete.**
- **Exit Codes:** Standardized `SlopChopExit` enum for predictable scripting automation.
- **Event Log:** `EventLogger` writes structured JSONL to `.slopchop/events.jsonl` for audit trails.
- **Surface Cleanup:** Removed Git-related keys (`auto_commit`, etc.) from documentation and internal config to match the architecture.

**Phase 3A (Patch UX) complete.**
- **Canonical V1 Patching:** Strict context anchoring, SHA256 locking, and `MAX_MATCHES: 1`.
- **Diagnostics:** "Did you mean?" probe logic and unambiguous failure messaging.

**v0.9.0 shipped.** The architectural pivot is complete and validated:
- **Staged workspace:** `slopchop apply` writes to `.slopchop/stage/worktree`.
- **Transactional promote:** `slopchop apply --promote` promotes verified changes.

---

## 2) Present (Where we are right now)

**Status:** GOLD / RELEASE CANDIDATE (v1.0.0 Ready)

### Operator-visible contract
- `slopchop apply` ingests to stage (File + Patch).
- `slopchop check` verifies stage.
- `slopchop apply --promote` commits to workspace.
- `events.jsonl` records the history.

### Trust boundary posture
- Zero trust for input payloads (must match hash, must match anchor).
- Zero accidental mutation (writes restricted to stage until explicit promote).
- Zero ambiguity (patches reject on >1 match).

---

## 3) Future (What we do next)

We have met the definition of done for v1.0.0.

### Immediate Next Action
**Release v1.0.0**.