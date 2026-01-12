# Case Study: Thubo vs. SlopChop
**When Governance Meets Different Physics**

**Date:** 2026-01-12
**Subject:** [Thubo](https://github.com/Mallets/thubo) (High-performance network pipeline)
**Tool:** [SlopChop v1.7.0]

---

## 1. Executive Summary
In January 2026, we ran SlopChop's default ruleset against `thubo`, a lock-free network pipeline library. The scan produced 70+ violations across 15 files.

**What the scan revealed:**
*   **Real Signal:** 28 violations of the **Law of Paranoia** — undocumented `unsafe` blocks. These are legitimate technical debts that even `cargo clippy -- -W clippy::undocumented_unsafe_blocks` confirms.
*   **Domain Mismatch:** Violations for file size (8500 tokens), coupling (CBO: 42), and complexity (CC: 46) that reflect *intentional architectural decisions* for a throughput-optimized system.
*   **False Positive:** `AtomicU64::load()` flagged as a database N+1 query.

**The Insight:** SlopChop didn't fail — it correctly identified that Thubo operates under different physics than a typical application. The tool's defaults optimize for maintainability; Thubo optimizes for nanoseconds.

---

## 2. Two Kinds of Software

| | **Application Code** | **Systems Code** |
|---|---|---|
| **Constraint** | I/O bound | CPU/Memory bound |
| **Goal** | Maintainability, Correctness | Throughput, Latency |
| **Cost of Abstraction** | Negligible (5ns call vs 5ms I/O) | Critical (5ns matters when you have millions) |
| **Coupling** | Minimize — enables refactoring | Intentional — orchestrators coordinate many components |
| **Unsafe** | Rare or absent | Pervasive and necessary |
| **Examples** | CLIs, Web servers, SlopChop | Kernels, Lock-free queues, Thubo |

A governance tool that doesn't recognize this distinction will either:
- Annoy systems programmers with irrelevant noise, or
- Miss real issues in application code by being too permissive

---

## 3. The Solution: Explicit Governance Profiles

Rather than auto-detecting project type (which proved unreliable and inconsistent), SlopChop v1.7 introduces **explicit profiles** configured via `slopchop.toml` or the TUI.
```toml
# slopchop.toml
profile = "systems"  # or "application" (default)
```

### Profile Comparison

| Rule | `application` | `systems` |
|------|---------------|-----------|
| Max file tokens | 2,000 | 10,000 |
| Max complexity | 15 | 50 |
| LCOM4 / CBO / SFOUT | Enabled | **Disabled** |
| Law of Paranoia (`unsafe`) | Warning | **Error** |
| `transmute` detection | Warning | **Error** |
| SAFETY comment required | Suggested | **Mandatory** |

**The Inversion Principle:** Systems mode *relaxes* structural metrics while *tightening* safety checks. This matches reality — systems code trades abstraction for performance, but must be paranoid about memory safety.

---

## 4. Fixing the False Positive (P03)

The `AtomicU64::load()` hallucination exposed a flaw in semantic pattern matching. The P03 rule was triggering on the word `load` without understanding context.

**Old approach (blocklist):** Flag `load`/`fetch`/`query` unless receiver contains `db`, `conn`, `pool`.
*Problem:* Fragile. Misses `repository.load()`, false-positives on `Atomic::load()`.

**New approach (allowlist):** Maintain a list of known-safe patterns:
- `Atomic*::load`, `Atomic*::store`
- `Arc::clone`, `Rc::clone`
- `MaybeUninit::assume_init`

If the method matches an allowlisted pattern, suppress the warning. This is a shorter list with fewer false positives.

---

## 5. Inline Overrides (v1.7)

For cases where systems authors need to "sign off" on specific complex code without relaxing the entire project:
```rust
#[allow(slopchop::cbo)]
struct PipelineOrchestrator {
    // High coupling is intentional here
}

#[allow(slopchop::complexity)]
fn state_machine_step(&mut self) {
    // CC: 45, but it's a single logical state machine
}
```

This provides an escape hatch that makes strict defaults palatable.

---

## 6. The Governance Hint

When a scan produces high violation density, SlopChop will suggest profile adjustment:
```
⚠ High violation density: 47 violations across 8 files (5.9 per file)

  If this is a high-performance systems project, consider:
    profile = "systems"

  This relaxes structural metrics while enforcing strict safety checks.
  Run `slopchop config` to adjust.
```

**Threshold:** `violations > (files × 3)` — relative to project size, not absolute.

---

## 7. Lessons for Thubo

The scan successfully identified real technical debt:

1. **28 undocumented `unsafe` blocks** across `ringbuf/common.rs`, `priority.rs`, `chunk.rs`, and others. These should have `// SAFETY:` comments explaining invariants.

2. **4 undocumented `unsafe impl Send/Sync`** — these are particularly important to document since they assert thread-safety properties the compiler can't verify.

Thubo maintainers can verify with:
```bash
cargo clippy -- -W clippy::undocumented_unsafe_blocks
```

---

## 8. Roadmap

### v1.7 (Immediate)
- [ ] Add `profile` field to `slopchop.toml`
- [ ] Implement `systems` profile with adjusted thresholds
- [ ] Add governance hint based on violation density
- [ ] Implement `#[allow(slopchop::*)]` inline overrides
- [ ] Allowlist for safe atomic/Arc patterns in P03

### v1.8 (Future)
- [ ] `script` profile for prototypes (minimal governance)
- [ ] TUI profile selector in config screen
- [ ] Per-directory profile overrides (`src/core/` = systems, `src/api/` = application)

---

## 9. Research Questions

1. **Profile Coverage:** What minimal set of profiles covers 90% of software? Current hypothesis: `application`, `systems`, `script`.

2. **Semantic Disambiguation:** Can we use import analysis to auto-populate the P03 allowlist? (e.g., if `std::sync::atomic` is imported, allowlist atomic methods)

3. **Violation Density Calibration:** What's the optimal threshold for the governance hint? Is `files × 3` too aggressive? Too permissive?

---

## 10. Conclusion

This case study demonstrates that SlopChop works — it found the one category of real issues (SAFETY comments) buried in domain-appropriate architectural decisions. The lesson isn't that the tool failed, but that **governance requires context**.

By introducing explicit profiles, we give developers control over that context while keeping the command surface simple: `slopchop check` does the right thing based on your declared intent.

> *"Governance is a social contract between you and your code. The tool should enforce the contract you chose, not guess what contract you wanted."*
