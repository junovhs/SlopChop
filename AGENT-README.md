# SlopChop Agent Protocol (v1.7.0)

You are operating in a repository governed by **SlopChop**. High-integrity development is mandatory. You must prioritize structural health and memory safety over speed.

## 1. The Flight Recorder (Primary Data Source)

**Your terminal view is likely truncated.** Do not rely on stdout for full violation lists or compiler errors.

After every check, SlopChop generates a persistent file at the repository root:
📄 **`slopchop-report.txt`**

**Workflow Requirements:**
1. Run `slopchop check`.
2. If it fails, **immediately read `slopchop-report.txt`**.
3. Use the "DASHBOARD" section of the report to identify the most complex/largest files.
4. Use the "FULL OUTPUT LOGS" section to see untruncated `cargo clippy` or test failures.

## 2. Governance Profiles

This repository uses a **Context-Aware** governance model. SlopChop automatically detects "Systems Code" (usage of `unsafe`, `Atomic`, `no_std`, or `repr(C)`) and adjusts the physics of the laws accordingly.

| Metric | `application` (Default) | `systems` (Relaxed) |
| :--- | :--- | :--- |
| **File Tokens** | < 2,000 | < 10,000 |
| **Cognitive Complexity** | ≤ 15 | ≤ 50 |
| **Nesting Depth** | ≤ 3 | ≤ 6 |
| **LCOM4 / CBO / SFOUT** | Strict | **Disabled** |

**Note on Safety:** In `systems` mode, structural rules are relaxed, but **Safety Checks are Escalated**. Every `unsafe` block **must** have a `// SAFETY:` comment or the check will fail.

## 3. Mandatory Commands

```bash
slopchop check              # THE GATE - Runs metrics, safety scan, clippy, and tests.
slopchop scan               # Internal metrics only (fast).
slopchop config             # Use this to view or adjust project-wide thresholds.
```

## 4. Technical Goals & Commits

When using the `XSC7XSC` protocol to deliver code via `PLAN` blocks:
- Always include a **`GOAL: <summary>`** line.
- SlopChop persists this goal. When the user runs `slopchop promote`, this goal is used to generate a high-quality merge commit.
- Generic commit messages (e.g., "update code") are a violation of protocol.

## 5. "Eating Your Vegetables"

Do the hardest things FIRST. **Dishonorable behavior includes:**
- Adding `#[allow(...)]` to bypass a metrics violation instead of refactoring.
- Ignoring the `LAW OF PARANOIA` (`.unwrap()`/`.expect()`).
- Claiming a task is complete without verifying that `slopchop-report.txt` shows **Status: PASSED**.

## 6. Verification Loop

```
1. Make changes.
2. Run `slopchop check`.
3. READ `slopchop-report.txt` to find the root cause of failures.
4. Refactor logic to reduce Cognitive Complexity or fix Safety documentation.
5. GOTO 2 until the report shows Status: PASSED.
```

The `slopchop-report.txt` dashboard is the ground truth. If a file appears at the top of the "Top 5 Cognitive Complexity" list, it is your primary target for refactoring.
