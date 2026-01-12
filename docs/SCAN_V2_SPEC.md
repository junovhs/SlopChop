# SlopChop Scan v2.0 Specification

**Status:** Live (v1.7.0)
**Date:** 2026-01-12
**Philosophy:** Do the hard thing first. No shortcuts. Real solutions.

---

## Architecture Overview

Scan v2.0 replaces the legacy linter with a high-integrity governance engine. It moves away from naive AST matching toward **High-Signal Pattern Detection** and **Structural Metrics**.

### Engine Internals
To adhere to our own laws (CBO/SFOUT), the engine is decoupled into three stages:
1.  **Worker (`worker.rs`)**: Parallel IO, Tree-sitter parsing, and local AST pattern matching.
2.  **Aggregator (`aggregator.rs`)**: A pure data container that collects local results and structural scopes without complex logic.
3.  **Deep Analyzer (`deep.rs` / `inspector.rs`)**: Performs global/inter-connected metric calculations (LCOM4, CBO) on the aggregated data.

### Removed Components (The "Chop")
1.  **Legacy `slopchop audit`:** The similarity and dead-code engine was removed due to high false-positive rates (lack of inter-procedural liveness analysis).
2.  **Surgical Patching:** Removed in favor of whole-file replacement to ensure idempotency and robustness against LLM hallucinations.
3.  **Naive Logic Checks:** Removed overly pedantic checks (e.g., generic `get()` in loops) that caused noise in idiomatic Rust code.

---

## Governance Profiles

Different software has different physics. A CLI tool and a lock-free queue have fundamentally different constraints. Rather than guess, SlopChop requires explicit intent.
```toml
# slopchop.toml
profile = "application"  # or "systems"
```

### Profile Definitions

| | **`application`** (Default) | **`systems`** |
|---|---|---|
| **Philosophy** | Maintainability > Performance | Throughput > Abstraction |
| **Use Cases** | CLIs, Web servers, Business logic | Kernels, Lock-free queues, Encoders |
| **Structural Metrics** | Strict | Relaxed |
| **Safety Checks** | Standard | Escalated |

### The Inversion Principle
Systems code trades abstraction for performance but must be paranoid about memory safety. The `systems` profile reflects this by *relaxing* structural metrics while *tightening* safety checks.

### Governance Hint
When violation density exceeds `files × 3`, SlopChop suggests profile adjustment:
```
⚠ High violation density: 47 violations across 8 files (5.9 per file)

  If this is a high-performance systems project, consider:
    profile = "systems"

  Run `slopchop config` to adjust.
```

---

## Metrics

Computed values with thresholds that vary by profile.

| Metric | `application` | `systems` | Category | Rationale |
|--------|---------------|-----------|----------|-----------|
| **File Tokens** | > 2,000 | > 10,000 | Atomicity | Prevents God Files; raised for orchestrators. |
| **Cognitive Complexity** | > 15 | > 50 | Complexity | Measures mental effort; raised for state machines. |
| **Nesting Depth** | > 3 | > 6 | Complexity | Deep nesting correlates with bug density. |
| **Function Args** | > 5 | > 7 | Complexity | High arity indicates weak abstraction. |
| **LCOM4** | > 1 | *Disabled* | State | Orchestrators intentionally coordinate disjoint components. |
| **AHF** | < 60% | *Disabled* | State | Attribute Hiding Factor; less relevant for systems code. |
| **CBO** | > 9 | *Disabled* | State | Coupling is inherent to hardware-adjacent orchestration. |
| **SFOUT** | > 7 | *Disabled* | Performance | Central coordinators naturally call many helpers. |

---

## AST Patterns (Implemented)

Boolean checks. Either the pattern exists or it doesn't.

### State (S)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| S01 | Global mutable declaration | `static mut` |
| S02 | Exported mutable | `pub static` (non-const) |
| S03 | Suspicious global container | `lazy_static! { Mutex<Vec> }` |

### Concurrency (C)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| C03 | Lock across await | `MutexGuard` held across `.await` point |
| C04 | Undocumented sync primitive | `Arc<Mutex<T>>` field without doc comments |

> **C03 Known Limitation:** Currently flags `async_mutex::MutexGuard`, which is safe to hold across await points. This is a false positive when using async-aware mutex implementations. Use config-file suppression or `systems` profile to mitigate.

### Resource (R)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| R07 | Missing flush | `BufWriter::new()` without explicit `.flush()` |

### Security (X)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| X01 | SQL Injection | `format!("SELECT...{}", var)` |
| X02 | Command Injection | `Command::new().arg(user_var)` |
| X03 | Hardcoded Secret | High-entropy string assigned to `key`/`token` var |
| X06 | Dangerous Config | `.dangerous()`, `verify_none` (TLS), `danger_accept_invalid` |
| X07 | Unbounded Deserialization | `bincode::deserialize` (Suggests `options().with_limit()`) |

> **Note:** Undocumented `unsafe` blocks are handled by `cargo clippy -- -W clippy::undocumented_unsafe_blocks`. SlopChop defers to clippy for Rust-specific safety lints.

### Performance (P)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| P01 | Clone in loop | `.clone()` inside hot loop |
| P02 | Allocation in loop | `String::to_string()` / `Vec::new()` in loop |
| P03 | N+1 Query | DB operations (`query`, `fetch`, `execute`) in loop |
| P04 | Nested Loop | O(n²) detection |
| P06 | Linear Search | `.find()` / `.position()` in loop |

### Semantic (M)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| M03 | Getter with mutation | `get_*` that takes `&mut self` |
| M04 | Name/Return mismatch | `is_*` / `has_*` returning non-`bool` |
| M05 | Side-effecting calc | `calculate_*` that takes `&mut self` |

### Idiomatic (I)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| I01 | Manual From impl | `impl From` (Suggests `derive_more`) |
| I02 | Match duplication | Identical bodies in `match` arms |
| I05 | Global Mutation | `std::env::set_var` in library code |

### Logic (L)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| L02 | Boundary ambiguity | `<= .len()` where operand is an index var (`i`, `idx`) |
| L03 | Unchecked access | `[0]` or `.first().unwrap()` without guard |

---

## Tuning Notes (v1.7.0)

### P03 (N+1 Queries)
Uses an **allowlist** of known-safe patterns that are never flagged:
- `Atomic*::load`, `Atomic*::store`, `Atomic*::fetch_*`
- `Arc::clone`, `Rc::clone`
- `MaybeUninit::assume_init`

All other `load`/`fetch`/`query` calls in loops are flagged for review.

### L02 (Boundary Checks)
Tightened to only flag comparisons against `.len()` if the other operand looks like an index variable (`i`, `j`, `idx`, `pos`). This prevents false positives on valid threshold checks like `if buffer.len() >= 1024`.

### P01/P02 (Loop Allocation)
Added "Ownership Sink" detection. If a clone/allocation is immediately consumed by a collection (`vec.push(x.clone())`), it is considered valid and not flagged.

---

## Suppressions

For edge cases where a warning is intentional, suppressions are configured in `slopchop.toml` — not inline attributes. This keeps source files tool-agnostic and suppression decisions auditable.
```toml
[suppressions]
"src/pipeline/orchestrator.rs" = ["cbo", "sfout"]
"src/state_machine.rs::step" = ["complexity"]
```

**Recommendation:** Profiles and global thresholds should handle 95% of cases. Suppressions are a last resort.

---

## Deferred / Dropped Patterns

| Status | ID | Pattern | Reason |
|--------|----|---------|--------|
| **Deferred** | C01, C02, C05 | Async Race/Floating Promise | TypeScript specific. |
| **Deferred** | R01, R02, R05 | Leaking Listener/Spread | TypeScript specific. |
| **Deferred** | X04, X05 | Unsafe Parse/Proto Pollution | TypeScript specific. |
| **Dropped** | S04 | Impure Function | Requires full Data Flow Analysis (expensive). |
| **Dropped** | S05 | Deep Mutation | Handled by Rust Borrow Checker. |
| **Dropped** | R03, R04 | Allocation in Loop | Merged into P01/P02. |
| **Dropped** | L01 | Untested Public | Replaced by `slopchop mutate`. |

---

## The Bug Categories

| Category | Coverage | Description |
|----------|----------|-------------|
| **State** | Full | LCOM4, AHF, CBO, Mutable Statics |
| **Concurrency** | Full | Mutex across await, Undocumented sync |
| **Resource** | Partial | Missing flush |
| **Security** | Full | Injection (SQL/Cmd), Secrets, TLS, Deserialization |
| **Performance** | Full | N+1 Queries, Cloning/Alloc in loops, Linear Search |
| **Semantic** | Partial | Name/Behavior alignment |
| **Idiomatic** | Full | Manual impls, Duplicate matches, Global mutation |
| **Logic** | Partial | Boundary checks, Unchecked access |

---

## Research References

- **S01/S02/AHF/LCOM4**: [State-02] State Ownership Spread Metrics.
- **C03/C04**: [Concurrency-02] Resources Across Await Points (DeadWait).
- **P01/P02/P03**: [Performance-03] N+1 & Repeated Async Calls.
- **X01/X02/X06**: [Security-04] Sink-Reaching Heuristics.
- **M03/M04**: [Semantic-04] Return Value Semantics vs Naming.
