# Locality v2: Antifragile Architecture Detection

**Status:** Proposal  
**Created:** 2025-12-28  
**Author:** Claude (via consultation with Spencer)

---

## Executive Summary

The current locality system requires manual configuration to avoid false positives. This proposal upgrades it to be zero-config and self-correcting by inferring architectural intent from the codebase structure itself.

---

## 1. Problem Statement

### Current Behavior

The locality scanner produces violations that require manual intervention:

```
▸ MISSING_HUB
    src\clipboard\mod.rs (fan-in: 4)
    → Add to [rules.locality].hubs in slopchop.toml

▸ TIGHT_COUPLING
    src\cli\handlers.rs → src\apply\mod.rs
    → 'cli' ↔ 'apply' coupled. Merge or extract shared interface
```

### Why This Is Fragile

1. **Manual hub declarations decay.** Every new utility module requires config updates. Forget once, get noise forever.

2. **False positive tight coupling.** CLI → Apply is correct layering, not bad coupling. The algorithm cannot distinguish intended architecture from accidental spaghetti.

3. **Configuration as debt.** The more config required, the less the tool gets used. Users disable features rather than maintain them.

### The Antifragile Principle

A system is antifragile when it gets *better* under stress rather than worse. An antifragile locality system would:

- Require zero configuration
- Automatically adapt as the codebase evolves
- Only flag genuine architectural violations
- Never produce false positives for standard patterns

---

## 2. Current Architecture

### What Exists

```
src/graph/locality/
├── classifier.rs    # Computes node identity (Hub, Leaf, Deadwood, GodModule)
├── coupling.rs      # Measures cross-module coupling
├── distance.rs      # LCA-based topological distance
├── validator.rs     # Universal Locality Algorithm (D ≤ 2 pass, D > 2 needs Hub)
└── mod.rs
```

### Current Algorithm

```
For each edge (A → B):
  1. Compute distance D via LCA
  2. If D ≤ 2: PASS (L1 cache)
  3. If D > 2 and B is Hub (K ≥ 1.0): PASS
  4. If D > 2 and B is not Hub: FAIL
```

### Current Config

```toml
[rules.locality]
max_distance = 4
hub_threshold = 1.0      # Skew K threshold for hub status
min_hub_afferent = 5     # Minimum fan-in for hub status
hubs = []                # Manual hub list (the fragile part)
```

---

## 3. Proposed Solution

### 3.1 Auto-Hub Detection

**Change:** Remove manual `hubs` config. Compute hub status entirely from graph metrics.

**Current:**
```toml
hubs = ["src/clipboard/mod.rs", "src/stage/mod.rs"]
```

**Proposed:**
```toml
auto_hub_threshold = 3   # fan-in >= 3 = automatic hub (or remove entirely, just use algorithm)
```

**Algorithm change:**
```rust
fn is_hub(node: &Node, graph: &Graph) -> bool {
    let fan_in = graph.afferent_coupling(node);
    let fan_out = graph.efferent_coupling(node);
    let skew = compute_skew(fan_in, fan_out);
    
    // Auto-hub: High fan-in relative to fan-out
    fan_in >= 3 && skew >= 0.5
}
```

**Result:** clipboard and stage automatically become hubs because they have high fan-in. No config needed. New utility modules auto-promote as usage grows.

---

### 3.2 Directional Coupling Detection

**Problem:** Current algorithm flags all cross-module edges as coupling. But A → B is not the same as A ↔ B.

**Proposed:** Only flag *bidirectional* coupling.

```rust
fn detect_coupling(graph: &Graph) -> Vec<CouplingViolation> {
    let mut violations = Vec::new();
    
    for (module_a, module_b) in graph.cross_module_edges() {
        let a_to_b = graph.has_edge(module_a, module_b);
        let b_to_a = graph.has_edge(module_b, module_a);
        
        if a_to_b && b_to_a {
            // Bidirectional = actual coupling problem
            violations.push(CouplingViolation::Circular { a: module_a, b: module_b });
        }
        // Unidirectional edges are fine - that's just layering
    }
    
    violations
}
```

**Result:** `cli → apply` stops being flagged. Only actual circular dependencies get reported.

---

### 3.3 Layer Inference

**Advanced option:** Infer layer order from the dependency graph itself.

```rust
fn infer_layers(graph: &Graph) -> Vec<Vec<Module>> {
    // Topological sort of modules by dependency depth
    // Modules with no internal dependencies = bottom layer
    // Modules that only depend on lower layers = next layer up
    // etc.
}
```

This would produce:
```
Layer 0 (infrastructure): tokens, constants, error
Layer 1 (utilities): clipboard, stage, skeleton
Layer 2 (core logic): apply, pack, analysis, audit
Layer 3 (interface): cli
Layer 4 (entrypoint): bin
```

**Violation rule:** Edges must flow downward. Layer N can depend on Layer N-1, N-2, etc. Layer N cannot depend on Layer N+1.

**Result:** No config needed. Layer order is computed. Violations only fire for upward dependencies.

---

### 3.4 Encapsulation Enforcement

**Problem:** `verification.rs` imports `cli::locality` directly instead of through `cli/mod.rs`.

**Current behavior:** Flags as ENCAPSULATION_BREACH with manual suggestion.

**Proposed:** Automatic enforcement rule.

```rust
fn check_encapsulation(import: &Import) -> Option<Violation> {
    let target_path = import.target_path();
    
    // If importing from a directory that has mod.rs,
    // the import must go through mod.rs
    if target_path.parent_has_mod_rs() && !target_path.is_mod_rs() {
        Some(Violation::EncapsulationBreach {
            from: import.source(),
            to: target_path,
            should_use: target_path.parent_mod_rs(),
        })
    } else {
        None
    }
}
```

**Result:** This is already working. No change needed, just noting it's correct.

---

## 4. Configuration After Changes

### Before (Fragile)

```toml
[rules.locality]
max_distance = 4
hub_threshold = 1.0
min_hub_afferent = 5
hubs = [
    "src/clipboard/mod.rs",
    "src/stage/mod.rs",
    # ... grows forever ...
]
```

### After (Antifragile)

```toml
[rules.locality]
mode = "warn"   # or "error" once validated
# That's it. Everything else is inferred.
```

Optional tuning (rarely needed):
```toml
[rules.locality]
mode = "warn"
auto_hub_fan_in = 3        # Override auto-hub threshold (default: 3)
exempt_patterns = ["test"] # Skip test files
```

---

## 5. Implementation Plan

### Phase 1: Auto-Hub (Low Risk)

**Effort:** 1-2 hours  
**Files:** `src/graph/locality/classifier.rs`, `src/config/locality.rs`

1. Change `is_hub()` to use fan-in threshold without manual list
2. Keep `hubs` config as override, but make it optional
3. Default behavior: auto-detect

**Test:** Run `slopchop scan --locality` — MISSING_HUB violations should disappear.

### Phase 2: Directional Coupling (Medium Risk)

**Effort:** 2-3 hours  
**Files:** `src/graph/locality/coupling.rs`, `src/cli/locality.rs`

1. Modify coupling detection to track edge direction
2. Only flag bidirectional edges
3. Update output format to show direction

**Test:** TIGHT_COUPLING for `cli → apply` should disappear. Actual circular deps (if any) still flagged.

### Phase 3: Layer Inference (Optional, Higher Risk)

**Effort:** 3-4 hours  
**Files:** New `src/graph/locality/layers.rs`

1. Implement topological layer computation
2. Add layer violation detection
3. Show inferred layers in output

**Test:** Run on SlopChop itself, verify inferred layers match intuition.

---

## 6. Success Criteria

| Metric | Before | After |
|--------|--------|-------|
| Config lines needed | 5-10+ | 1-2 |
| False positives on SlopChop | 7 | 0 |
| Manual hub declarations | Required | Never |
| New module onboarding | Edit config | Automatic |

**Definition of Done:**

1. `slopchop scan --locality` on SlopChop produces 0 violations (or only genuine ones)
2. No `hubs = [...]` in default config
3. TIGHT_COUPLING only fires for bidirectional dependencies
4. Documentation updated

---

## 7. Risks and Mitigations

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Auto-hub threshold wrong | Medium | Make it configurable, default conservative |
| Layer inference incorrect | Low | Phase 3 is optional, can skip |
| Breaks existing behavior | Low | Keep old config as fallback |

---

## 8. Decision

**Recommended path:** Implement Phase 1 and Phase 2. Skip Phase 3 unless needed.

This gives 90% of the benefit (zero false positives, zero config) with 50% of the effort.

---

## Appendix: Research Foundation

The locality system is based on:

- **Martin's Stability Metrics** — Fan-in/fan-out, Instability (I), Abstractness
- **LCA Distance** — Topological distance via Lowest Common Ancestor in module tree
- **Skew (K)** — Novel metric: `K = ln((Cₐ+1)/(Cₑ+1))` distinguishing hubs from leaves

See `docs/software-topology-brief.md` for full theoretical background.
