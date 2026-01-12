# SlopChop Scan v2: Past, Present, Future

**Date:** 2026-01-12
**Version:** v1.7.0 (The "Context-Aware" Release)

---

## What Was Done Today (v1.7.0)

### Phase 3: The Systems Stress Test (Subject: `thubo`)
- **Stress Test Findings:** Ran SlopChop against a high-performance network pipeline. Discovered a 95% noise rate due to the "Web App" bias of the default ruleset.
- **Architectural Pivot:** Introduced **Governance Profiles** (`application` vs. `systems`).
    - **Systems Mode:** Relaxes architectural metrics (File Size, LCOM4, Complexity) to allow for hardware-level optimization.
    - **Safety Escalation:** Tightens semantic checks (Safety comments, `transmute` warnings) to manage the risk of relaxed structures.
- **Heuristic Auto-Detection:** Implemented a weighted scoring system to detect systems code. Files scoring $\ge 3$ (based on `unsafe`, `Atomic`, `repr(C)`, `no_std`) automatically switch to relaxed structural limits.
- **P03 Semantic Disambiguation:** Tightened N+1 detection to allowlist CPU-level instructions like `Atomic::load` and `Arc::clone`, preventing "I/O hallucinations."

### Phase 4: UX & Agent Ergonomics
- **The "Flight Recorder":** Implemented `slopchop-report.txt`. A persistent, untruncated log file that stores the full results of the last `slopchop check`, specifically designed for AI Agents to read context without terminal pollution.
- **Rich Snippets:** Overhauled violation reporting to include `rustc`-style snippets with line numbers, gutters, and red underlines for immediate context.
- **Live Spinner:** Upgraded the spinner to stream real-time stdout/stderr from child processes (clippy/tests), providing immediate feedback on progress.
- **Adaptive TUI:** Added "Coarse/Fine" stepping to the configuration editor.
    - `Left/Right`: $\pm 1$ step.
    - `Up/Down`: $\pm 10/100/500$ based on value magnitude.
    - Added `EnterAlternateScreen` support to fix terminal stacking glitches.

### Phase 5: Transactional Integrity
- **Meaningful Merges:** Fixed the "useless commit message" issue.
    - The `GOAL:` from the `apply` PLAN block is now persisted in `.slopchop/pending_goal`.
    - `slopchop promote` reads this goal to generate a descriptive merge commit: `feat: [Goal] (promoted)`.

---

## Current Pattern Coverage (Rust)

| Category | IDs | Status |
|----------|-----|--------|
| **State** | S01, S02, S03 | [OK] Stable |
| **Concurrency** | C03, C04 | [OK] High Signal |
| **Security** | X01, X02, X03, **X06**, **X07** | [OK] Production Ready |
| **Performance** | P01, P02, **P03 (Tuned)**, P04, P06 | [OK] Type-Aware |
| **Semantic** | M03, M04, M05 | [OK] Stable |
| **Resource** | R07 | [OK] Stable |
| **Idiomatic** | I01, I02, I05 | [OK] Stable |
| **Logic** | L02, L03 | [OK] Low Noise |

**Total: 23 active patterns**

---

## Current Status

| Category | Status | Notes |
|----------|--------|-------|
| **Architecture** | [OK] Clean | Engine is modular (Worker/Aggregator/Deep). |
| **Profiles** | [OK] Active | Per-file systems detection working. |
| **UX** | [OK] Rich | Code snippets and persistent reports active. |
| **Tests** | [OK] Pass | 68 unit tests passing. |
| **Self-Scan** | [OK] 0 Violations | Passing at `Cognitive Complexity = 15`. |

---

## Future Roadmap: Methodical Audit Phase

### 1. The GitHub Stress Test (Upcoming)
The next major phase involves methodically scanning top-tier Rust repositories to build a "Knowledge Base" of architectural patterns:
- **Targets:** `tokio`, `polars`, `ripgrep`, `axum`. (and maybe some shitty ones too)
- **Goal:** Identify where SlopChop fails to understand idiomatic high-performance patterns and refine the **Systems Profile** thresholds.
- **Output:** A comprehensive findings report for each repo to inform v1.8 logic.

### 2. TypeScript/JS Support
- Map existing "Deferred" patterns (Async races, Event listener leaks) to `tree-sitter-typescript`.
- Implement `tsconfig.json` path resolution for global coupling metrics.

### 3. Semantic Cost Analysis (Research)
- Develop techniques to infer the "Cost" of a function call based on its call tree (e.g., does it eventually hit a syscall or a mutex?).
- Use this to automatically flag high-frequency functions that aren't marked `#[inline]`.

---

## Key Files (The Core)

```
src/
|-- analysis/
|   |-- file_analysis.rs  # Heuristic scoring & profile overrides
|   `-- v2/
|       |-- cognitive.rs  # Cognitive Complexity engine
|       `-- patterns/     # AST logic (P03 Type-Aware)
|-- apply/
|   |-- executor.rs       # Goal persistence
|   `-- verification.rs   # Report generation (Flight Recorder)
`-- cli/
    |-- config_ui/        # Adaptive stepping TUI
    `-- handlers.rs       # Goal-aware promotion
```

"but is the   AGENT-README.md presently accurate or aspirational?"

It is **presently accurate.**

As of the code we just finished writing and you just installed, every claim in that `AGENT-README.md` is backed by logic in your current binary.

Here is the "receipt" for the features in your source code:

| Feature | Logic Location (Source Code) | Status |
| :--- | :--- | :--- |
| **`slopchop-report.txt`** | `src/apply/verification.rs` -> `write_check_report` | **Live.** It writes the dashboard and full logs to disk on every check. |
| **Systems Detection** | `src/analysis/file_analysis.rs` -> `calculate_systems_score` | **Live.** It uses the weighted score (unsafe=1, no_std=5, etc.) to relax rules per-file. |
| **Descriptive Merges** | `src/apply/executor.rs` -> `save_pending_goal` | **Live.** It saves your PLAN goal to `.slopchop/pending_goal`. |
| **Goal Promotion** | `src/cli/handlers.rs` -> `handle_promote` | **Live.** It reads that goal and uses it for the `git merge` message. |
| **Cognitive Complexity** | `src/analysis/ast.rs` -> `process_function_node` | **Live.** It uses the v2 cognitive engine we established earlier. |
| **Rich Snippets** | `src/reporting.rs` -> `print_snippet` | **Live.** It reads the file and draws the gutter/underline. |

### The "Heuristic" Reality
The only thing that is "imperfect" (but still live) is the **Systems Detection**. It relies on `source.contains("unsafe {")` rather than a full AST symbol resolution. It is a highly effective heuristic, but as we discussed in the "Research Questions," making it "Perfectly Reliable" is the future goal. 

For v1.7.0, **it is functional and truthful.** You can verify it by running `slopchop check` on a file with `unsafe` and seeing the complexity limit jump.
