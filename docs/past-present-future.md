# Past / Present / Future

**Status:** Canonical (living snapshot)
**Last updated:** 2025-12-26
**Canonical policy:** This document states the current operational reality and the single next action.
**For full v1.0 priorities / blockers / DoD:** see `/docs/v1-brief.md`.

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
* No further structural changes required for correctness or safety.

---

## 2.1) Deferred Follow-Ups (Intentionally Paused)

These items were identified during final v1.0 polish but explicitly deferred to avoid reopening scope.
They are **non-blocking**, **UX- or transport-facing**, and do **not** affect correctness of the trust boundary.

### Patch UX (Diagnostics & Transport Robustness)

* Add bounded visual diff summary to **0-match PATCH diagnostics** (locate-only; never fuzzy-apply).
* Improve PATCH diagnostics messaging for ambiguity vs context mismatch.
* Migrate remaining integration tests off deprecated v0.9 SEARCH/REPLACE form to canonical v1 PATCH format.

### Transport Hardening (AI / Markdown Interfaces)

* Tolerate indented or prefixed `XSC7XSC` sigil lines (e.g. list items, blockquotes).
* Optional sanitize mode to strip UI-injected markdown fencing (``` / ~~~) from non-markdown FILE content.
* Improve parser error hints when sigils are detected but not recognized due to formatting noise.

**Rationale:**
These changes improve resilience when copying payloads through hostile renderers (chat UIs, markdown editors) but do not weaken the deterministic apply model. Deferred to avoid destabilizing v1.0 at release time.

---

## 3) Future (What we do next)

We are in **Post-v1.0 Era**.

### Objectives

* Monitor stability.
* Distribution (Homebrew / Scoop / Winget).
* User feedback loop.
* Opportunistic UX hardening based on real-world friction.

### Immediate Next Action

**Celebrate.**
Then pivot to building actual applications using SlopChop as a boring, trusted substrate.
