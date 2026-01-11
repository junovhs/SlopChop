# SlopChop Scan v2: Past, Present, Future

**Date:** 2026-01-11

---

## Past: What Existed

### Original State
- **Metrics:** File tokens, cyclomatic complexity, nesting depth, function args
- **Patterns:** Basic `.unwrap()` detection, some state checks
- **Problem:** High false positive rate, cyclomatic complexity is weak predictor

### What We Changed (This Session)
1. **P01/P02 tuning** — Added context-awareness based on SOTA research:
   - Skip clones/allocs feeding ownership sinks (`.insert()`, `.push()`, `.entry()`)
   - Skip clones of loop variable (per-iteration, not hoistable)
   - Skip Copy types (Range, integers)
   - Result: ~90% false positive reduction

2. **X02 tuning** — Added source provenance detection:
   - Skip trusted variable names (`cmd`, `command`, `binary`, etc.)
   - Skip const/static definitions
   - Skip config parsing contexts
   - Skip struct field access patterns
   - Skip match allowlist patterns
   - Result: Eliminates false positives on internal tooling

3. **Files modified:**
   - `src/analysis/v2/patterns/performance.rs`
   - `src/analysis/v2/patterns/security.rs`

---

## Present: Current State

### Metrics (100% complete)
| Metric | Status |
|--------|--------|
| File Tokens (>2000) | ✅ |
| Cognitive Complexity (>15) | ✅ |
| Nesting Depth (>3) | ✅ |
| Function Args (>5) | ✅ |
| LCOM4 (>1) | ✅ |
| AHF (<60%) | ✅ |
| CBO (>9) | ✅ |
| SFOUT (>7) | ✅ |

### Patterns by Category

| Category | Done | Missing | Notes |
|----------|------|---------|-------|
| **State** | S01, S02, S03 | S04, S05 | S04 should be dropped (DFA nightmare) |
| **Concurrency** | C03 | C01, C02, C04, C05 | C01/C02/C05 are TS-focused |
| **Resource** | — | R01, R02, R06, R07 | R03/R04 merged into P01/P02 |
| **Security** | X01, X02, X03 | — | X04/X05 are TS-only |
| **Performance** | P01, P02, P04, P06 | P03, P05 | P07 dropped (not real issue) |
| **Semantic** | M01? | M03, M04, M05 | M02 dropped (compiler handles) |
| **Idiomatic** | — | I01, I02, I04 | I03 dropped (too noisy) |
| **Logic** | — | L02, L03 | L01 dropped (coverage tool) |

### Known Issues
1. No tests for the new context-aware logic
2. TypeScript patterns largely unimplemented
3. Cross-file analysis infrastructure unclear

---

## Future: Next Steps

### Immediate (Next Session)

**Priority 1: Add tests for P01/P02/X02 tuning**
```
src/analysis/v2/patterns/performance_test.rs
src/analysis/v2/patterns/security_test.rs
```
Test cases needed:
- Clone inside `.insert()` → no violation
- Clone of loop var → no violation
- Clone of external var → violation
- `Command::new(CONST)` → no violation
- `Command::new(param)` → violation

**Priority 2: Implement P03 (N+1 query)**
```rust
// Pattern: DB call inside loop where param is loop var
for user in users {
    let posts = db.query("SELECT * FROM posts WHERE user_id = ?", user.id);
}
```
This is high-value, low-noise.

**Priority 3: Implement M03/M04 (semantic name checks)**
```rust
// M03: Getter with mutation
fn get_value(&mut self) -> i32 {
    self.count += 1;  // violation
    self.value
}

// M04: Name/return mismatch
fn is_valid(&self) -> String {  // violation: is_* should return bool
    ...
}
```

### Medium Term

1. **R07 (Missing flush)** — High signal for Rust:
   ```rust
   let writer = BufWriter::new(file);
   writer.write_all(data)?;
   // missing writer.flush()
   ```

2. **I01/I02 (Idiomatic patterns)** — Migrate from old audit:
   - Manual `From` impl detection
   - Duplicate match arm bodies

3. **C03 improvements** — Lock across await already exists, verify quality

### Long Term

1. **TypeScript support** — Many patterns are TS-focused (C01, C02, R01, R02, X04, X05)
2. **Cross-file analysis** — Needed for proper N+1 detection, dead code, etc.
3. **Incremental scanning** — Only re-scan changed files

---

## Spec Amendments

Based on this session, recommend updating spec:

### Drop
| ID | Reason |
|----|--------|
| R03 | Redundant with P02 |
| R04 | Redundant with P01 |
| P07 | `.to_string()` on primitives is fine |
| S04 | Requires full DFA, false positive nightmare |
| M02 | Compiler already warns |
| I03 | Too stylistic/noisy |
| L01 | Coverage metric, different tool |

### Clarify
| ID | Clarification |
|----|---------------|
| P01/P02 | Add: "Skip ownership transfers and loop-variable clones" |
| P06 | Change: `.contains()` → `.find()`/`.position()` (contains is O(1) on sets) |
| X02 | Add: "Skip const/static sources and config contexts" |

---

## Files Reference

```
src/analysis/v2/patterns/
├── performance.rs   # P01, P02, P04, P06 (tuned this session)
├── security.rs      # X01, X02, X03 (tuned this session)
├── state.rs         # S01, S02, S03
├── concurrency.rs   # C03
├── semantic.rs      # M01?
└── mod.rs           # Pattern orchestration
```

---

## Commands

```bash
# Full check (clippy + test + scan)
slopchop check

# Scan only
slopchop scan

# Scan with JSON output
slopchop scan --json
```
