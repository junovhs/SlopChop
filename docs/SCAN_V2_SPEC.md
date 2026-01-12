# SlopChop Scan v2.0 Specification

**Status:** Live (v1.6.0)
**Date:** 2026-01-12
**Philosophy:** Do the hard thing first. No shortcuts. Real solutions.

---

## Architecture Overview

Scan v2.0 replaces the legacy linter with a high-integrity governance engine. It moves away from naive AST matching toward **High-Signal Pattern Detection** and **Structural Metrics**.

### Engine Internals (v1.6.0 Refactor)
To adhere to our own laws (CBO/SFOUT), the engine is decoupled into three stages:
1.  **Worker (`worker.rs`)**: Parallel IO, Tree-sitter parsing, and local AST pattern matching.
2.  **Aggregator (`aggregator.rs`)**: A pure data container that collects local results and structural scopes without complex logic.
3.  **Deep Analyzer (`deep.rs` / `inspector.rs`)**: Performs global/inter-connected metric calculations (LCOM4, CBO) on the aggregated data.

### Removed Components (The "Chop")
1.  **Legacy `slopchop audit`:** The similarity and dead-code engine was removed due to high false-positive rates (lack of inter-procedural liveness analysis).
2.  **Surgical Patching:** Removed in favor of whole-file replacement to ensure idempotency and robustness against LLM hallucinations.
3.  **Naive Logic Checks:** Removed overly pedantic checks (e.g., generic `get()` in loops) that caused noise in idiomatic Rust code.

---

## The Bug Categories

| Category | Coverage | Description |
|----------|----------|-------------|
| **State** | Full | LCOM4, AHF, CBO, Mutable Statics |
| **Concurrency** | Full | Mutex across await, Undocumented sync |
| **Resource** | Partial | Missing flush (Allocations moved to Performance) |
| **Security** | Full | Injection (SQL/Cmd), Secrets, TLS, Deserialization |
| **Performance** | Full | N+1 Queries, Cloning/Alloc in loops, Linear Search |
| **Semantic** | Partial | Name/Behavior alignment |
| **Idiomatic** | Full | Manual impls, Duplicate matches, Global mutation |
| **Logic** | Partial | Boundary checks, Unchecked access |

---

## Metrics

Computed values with configurable thresholds in `slopchop.toml`.

| Metric | Default | Category | Rationale |
|--------|---------|----------|-----------|
| **File Tokens** | > 2000 | Atomicity | Prevents God Files; encourages modularity. |
| **Cognitive Complexity** | > 15 | Complexity | Measures mental effort; replaces Cyclomatic. |
| **Nesting Depth** | > 3 | Complexity | Deep nesting correlates with bug density. |
| **Function Args** | > 5 | Complexity | High arity indicates weak abstraction. |
| **LCOM4** | > 1 | State | Value > 1 implies class handles disjoint responsibilities. |
| **AHF** | < 60% | State | Attribute Hiding Factor; measures encapsulation. |
| **CBO** | > 9 | State | Coupling Between Objects; measures dependency fan-in/out. |
| **SFOUT** | > 7 | Performance | Structural Fan-Out; identifies architectural bottlenecks. |

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
| **X06** | Dangerous Config | `.dangerous()`, `verify_none` (TLS), `danger_accept_invalid` |
| **X07** | Unbounded Deserialization | `bincode::deserialize` (Suggests `options().with_limit()`) |

### Performance (P)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| P01 | Clone in loop | `.clone()` inside hot loop (Context-aware skipping) |
| P02 | Allocation in loop | `String::to_string()` / `Vec::new()` in loop |
| P03 | N+1 Query | Explicit DB verbs (`query`, `fetch`) in loop (Tuned) |
| **P04** | Nested Loop | O(n^2) detection |
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
| **I05** | Global Mutation | `std::env::set_var` in library code |

### Logic (L)

| ID | Pattern | Rust Context |
|----|---------|--------------|
| L02 | Boundary ambiguity | `<= .len()` where operand is an index var (`i`, `idx`) |
| L03 | Unchecked access | `[0]` or `.first().unwrap()` without guard |

---

## Deferred / Dropped Patterns

These patterns appear in research but were excluded from v1.6.0.

| Status | ID | Pattern | Reason |
|--------|----|---------|--------|
| **Deferred** | C01, C02, C05 | Async Race/Floating Promise | TypeScript specific. |
| **Deferred** | R01, R02, R05 | Leaking Listener/Spread | TypeScript specific. |
| **Deferred** | X04, X05 | Unsafe Parse/Proto Pollution | TypeScript specific. |
| **Dropped** | S04 | Impure Function | Requires full Data Flow Analysis (Expensive). |
| **Dropped** | S05 | Deep Mutation | Handled by Rust Borrow Checker. |
| **Dropped** | R03, R04 | Allocation in Loop | Merged into P01/P02. |
| **Dropped** | L01 | Untested Public | Replaced by `slopchop mutate`. |

---

## Tuning Notes (v1.6.0)

- **P03 (N+1 Queries):** Significantly tightened. Previously flagged `get()` calls on HashMaps. Now requires explicit database verbs (e.g., `fetch_one`, `query`, `execute`) from `sqlx` or `diesel` patterns.
- **L02 (Boundary Checks):** Tightened to only flag comparisons against `.len()` if the other operand looks like an index variable (`i`, `j`, `idx`, `pos`). This prevents false positives on valid threshold checks like `if buffer.len() >= 1024`.
- **P01/P02 (Loop Alloc):** Added "Ownership Sink" detection. If a clone/allocation is immediately consumed by a collection (`vec.push(x.clone())`), it is considered valid and not flagged.

---

## Research References

- **S01/S02/AHF/LCOM4**: [State-02] State Ownership Spread Metrics.
- **C03/C04**: [Concurrency-02] Resources Across Await Points (DeadWait).
- **P01/P02/P03**: [Performance-03] N+1 & Repeated Async Calls.
- **X01/X02/X06**: [Security-04] Sink-Reaching Heuristics.
- **M03/M04**: [Semantic-04] Return Value Semantics vs Naming.
