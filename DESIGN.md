# SlopChop Design Document

> **Status:** V1.0 Hardening Complete (Phase 2)
> **Audience:** Developers (human or AI) working on or extending SlopChop.
> **See also:** [README.md](README.md) for user guide.

---

## Table of Contents

1. [Vision & Philosophy](#vision--philosophy)
2. [Architecture Overview](#architecture-overview)
3. [The Three Laws](#the-three-laws)
4. [The SlopChop Protocol](#the-slopchop-protocol)
5. [Analysis Engine](#analysis-engine)
6. [Apply System (V1.0 Hardened)](#apply-system-v10-hardened)
7. [Context Generation](#context-generation)
8. [Dependency Graph](#dependency-graph)
9. [Roadmap System (V2)](#roadmap-system-v2)
10. [TUI Dashboard](#tui-dashboard)
11. [Security Model](#security-model)
12. [Key Decisions & Rationale](#key-decisions--rationale)
13. [Future Work](#future-work)

---

## Vision & Philosophy

### The Problem

AI coding assistants are powerful but unreliable. They:
- Generate files too large to review meaningfully
- Produce complex functions that can't be tested in isolation
- Truncate code with `// ...` or "rest of implementation"
- Escape markdown fences incorrectly, corrupting output
- Have no memory of project constraints between sessions

### The Solution

**SlopChop is a gatekeeper, not a fixer.** It creates a feedback loop:

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   slopchop pack ──► AI ──► slopchop apply ──► verify ──► commit │
│        ▲                         │                              │
│        │                         ▼                              │
│        └────── rejection ◄───── FAIL                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

When AI output violates constraints:
1. SlopChop rejects the entire response
2. Generates a structured error message
3. Copies it to clipboard for pasting back to AI
4. AI corrects and resubmits

**The AI learns the constraints through rejection, not instruction.**

### Core Principles

| # | Principle | Meaning |
|---|-----------|---------|
| 1 | **Reject bad input, don't fix it** | SlopChop is a gatekeeper, not a linter with autofix |
| 2 | **Git is the undo system** | Don't reinvent version control. Commit on success |
| 3 | **Explicit > Magic** | Fail loudly on format violations |
| 4 | **Context is king** | Give AI exactly what it needs, nothing more |
| 5 | **Graph over glob** | Understand structure, don't just pattern match |
| 6 | **Self-hosting** | SlopChop passes its own rules |

### What SlopChop Is NOT

- **Not a linter** — It doesn't suggest fixes, it rejects
- **Not an IDE plugin** — CLI-first, composable with any editor
- **Not AI-specific** — The constraints help human reviewers too
- **Not prescriptive about style** — Cares about size and complexity, not formatting

---

## Architecture Overview

```
src/
├── analysis/          # The Three Laws enforcement
│   ├── ast.rs         # Tree-sitter query compilation
│   ├── checks.rs      # Violation detection logic
│   ├── metrics.rs     # Complexity, depth, arity calculations
│   └── mod.rs         # RuleEngine orchestration
│
├── apply/             # AI response → filesystem (V1.0 Hardened)
│   ├── backup.rs      # Backup creation & rollback logic
│   ├── extractor.rs   # Protocol parsing
│   ├── manifest.rs    # MANIFEST block parsing
│   ├── validator.rs   # Integrity & Path Safety
│   ├── writer.rs      # Atomic Transactional Writes
│   ├── verification.rs# Post-apply check commands
│   ├── git.rs         # Git commit/push operations
│   ├── messages.rs    # Error message formatting
│   ├── types.rs       # ApplyContext, ApplyOutcome types
│   └── mod.rs         # Orchestration
│
├── audit/             # Consolidation Audit (God Tier)
│   ├── dead_code/     # Reachability analysis
│   ├── patterns/      # Structural pattern matching
│   ├── report/        # AI/JSON/Terminal reporting
│   ├── fingerprint.rs # AST hashing (Weisfeiler-Lehman)
│   ├── similarity.rs  # Clustering & duplication detection
│   ├── diff.rs        # Structural diffing
│   ├── enhance.rs     # Refactoring plan generation
│   └── scoring.rs     # Opportunity impact analysis
│
├── clipboard/         # Cross-platform clipboard
│   ├── platform.rs    # OS abstraction
│   ├── temp.rs        # Smart copy (temp file)
│   └── mod.rs         # Helper functions
│
├── graph/             # Dependency analysis
│   ├── defs/          # Symbol definition extraction
│   ├── rank/          # PageRank importance
│   ├── imports.rs     # Import extraction
│   └── resolver.rs    # File resolution
│
├── pack/              # Context generation
│   ├── focus.rs       # Foveal/Peripheral calculation
│   ├── formats.rs     # XML/Text formatting
│   └── mod.rs         # CLI entry point
│
├── roadmap_v2/        # Task management (tasks.toml)
│   ├── cli/           # Subcommands
│   ├── parser.rs      # Command parsing
│   ├── storage.rs     # TOML persistence
│   └── validation.rs  # Logic checks
│
├── trace/             # Dependency tracing
│   ├── runner.rs      # Recursive tracer
│   └── output.rs      # Graph rendering
│
├── tui/               # Terminal Dashboard
│   ├── dashboard/     # Interactive UI
│   ├── config/        # Config editor
│   ├── watcher.rs     # Clipboard monitoring
│   └── runner.rs      # Crossterm setup
│
├── cli/               # CLI Argument Parsing
│   ├── args.rs        # Clap struct definitions
│   └── handlers.rs    # Command dispatch
│
├── config/            # Configuration
│   ├── io.rs          # File loading
│   └── types.rs       # Struct definitions
│
├── bin/
│   └── slopchop.rs    # Binary entry point
│
├── clean.rs           # Cleanup utilities
├── constants.rs       # Global patterns
├── detection.rs       # Project type detection
├── discovery.rs       # File finding (git/walk)
├── error.rs           # Error types
├── lang.rs            # Language definitions
├── project.rs         # Project metadata
├── prompt.rs          # System prompt generator
├── reporting.rs       # Scan reports
├── signatures.rs      # Signature map generation
├── skeleton.rs        # Code minimization
├── spinner.rs         # UX utils
├── tokens.rs          # Token counting (tiktoken)
├── types.rs           # Shared domain types
├── wizard.rs          # Setup wizard
└── lib.rs             # Crate root
```

### Data Flow

```
User runs "slopchop pack --focus file.rs"
         │
         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    discovery    │────►│      graph      │────►│      pack       │
│   (find files)  │     │  (build deps)   │     │ (generate ctx)  │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                         ▼
                                                 context.txt + prompt
                                                         │
                                                    [TO AI]
                                                         │
                                                         ▼
                                                 AI response (Protocol)
                                                         │
                                                         ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│    extractor    │────►│    validator    │────►│     writer      │
│ (parse blocks)  │     │ (check integrity)│    │(atomic transact)│
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                         │
                                                  [FAIL? Rollback]
                                                         │
                                                         ▼
                                                 ┌───────────────┐
                                                 │ verification  │
                                                 │ (cargo test)  │
                                                 └───────────────┘
                                                         │
                                    ┌────────────────────┴────────────────────┐
                                    ▼                                         ▼
                              [PASS: commit]                          [FAIL: reject]
                                    │                                         │
                                    ▼                                         ▼
                              git commit/(push)                    copy feedback to clipboard
```

---

## The Three Laws

SlopChop enforces structural constraints. These are configurable but opinionated defaults.

### Law of Atomicity

**Files must be small enough to reason about.**

```toml
[rules]
max_file_tokens = 2000  # ~500 lines of code
```

**Why:** Large files can't be meaningfully reviewed. AI-generated code tends toward monolithic files. Forcing small files creates natural modularity.

### Law of Complexity

**Functions must be simple enough to test.**

```toml
[rules]
max_cyclomatic_complexity = 8   # Branches per function
max_nesting_depth = 3           # if/for/while depth
max_function_args = 5           # Parameter count
max_function_words = 5          # Words in function name
```

**Why:**
- High complexity = hard to test exhaustively
- Deep nesting = hard to follow control flow
- Many arguments = function doing too much
- Long names = unclear responsibility

**Implementation:** Tree-sitter queries count:
- Complexity: `if`, `match`, `for`, `while`, `&&`, `||`
- Depth: Nested `block` and `body` nodes
- Arity: Children of `parameters`/`arguments` nodes

### Law of Paranoia (Rust-specific)

**No panic paths in production code.**

```rust
// REJECTED
let value = thing.unwrap();
let other = thing.expect("msg");

// ALLOWED
let value = thing.unwrap_or(default);
let value = thing.unwrap_or_else(|| compute());
let value = thing?;
```

**Why:** `.unwrap()` and `.expect()` are hidden crash paths. In production, explicit error handling is safer.

**Implementation:** Tree-sitter query matches `call_expression` where method is `unwrap` or `expect`.

---

## The SlopChop Protocol

### Why Not Markdown Fences?

AI models frequently mess up markdown code fences:
- Nested fences get escaped wrong
- Closing fences match incorrectly
- Language tags vary unpredictably

The `#__SLOPCHOP_FILE__#` and `#__SLOPCHOP_END__#` delimiters:
- Never appear in normal code
- Unambiguous start/end
- Don't require escape sequences
- Machine-parseable

### Format Specification

```
#__SLOPCHOP_PLAN__#
GOAL: What you're doing
CHANGES:
1. First change
2. Second change
#__SLOPCHOP_END__#

#__SLOPCHOP_MANIFEST__#
src/file1.rs
src/file2.rs [NEW]
src/old.rs [DELETE]
#__SLOPCHOP_END__#

#__SLOPCHOP_FILE__# src/file1.rs
// Complete file content
// No truncation allowed
#__SLOPCHOP_END__#

===ROADMAP===
CHECK
id = task-id
ADD
id = new-task
text = New feature
section = v0.2.0
===ROADMAP===
```

### Block Types

| Block | Purpose | Required |
|-------|---------|----------|
| `PLAN` | Human-readable summary | Recommended |
| `MANIFEST` | Declares all files being touched | **MANDATORY** (V1.0) |
| File blocks | Actual file content | Required |
| `ROADMAP` | Task updates | Optional |

### Markers

| Marker | Meaning |
|--------|---------|
| `[NEW]` | File doesn't exist, will be created |
| `[DELETE]` | File will be removed |
| *(none)* | File exists, will be updated |

---

## Apply System (V1.0 Hardened)

### Integrity Checks

SlopChop V1.0 strictly enforces **Manifest Integrity**:
- **Completeness:** Every file in the `MANIFEST` (marked New/Update) MUST have a corresponding `#__SLOPCHOP_FILE__#` block.
- **Consistency:** Every `#__SLOPCHOP_FILE__#` block MUST be listed in the `MANIFEST`.
- **Delete Safety:** Files marked `[DELETE]` MUST NOT have content blocks.

If any check fails, the **entire** operation is rejected. Silent partial applies are impossible.

### Atomic Transactions & Rollback

`slopchop apply` is fully transactional:

1. **Backup Phase:** All target files are backed up to `.slopchop_apply_backup/`.
2. **Write Phase:** Files are written using **Atomic Rename** (write temp → rename).
3. **Rollback Phase:** If any write fails (IO error, permission, symlink escape), the system **automatically rolls back**:
   - Restores modified files from backup.
   - Deletes newly created files.

The repo is guaranteed to be in either the "Pre-Apply" or "Post-Apply" state. Never a broken middle state.

### Validation Rules

**Path Safety:**
- No `../` traversal
- No absolute paths
- No sensitive directories (`.git`, `.env`, `.ssh`, `.aws`)
- **Symlink Protection:** Writes through symlinks that point outside the repo root are blocked.

**Protected Files:**
- `ROADMAP.md` — Use roadmap commands instead
- `slopchop.toml`, `Cargo.lock`, `package-lock.json`

**Content Safety:**
- No truncation markers (`// ...`, `/* ... */`, `# ...`)
- No lazy phrases ("rest of implementation", "remaining code")
- No empty files
- No markdown fences in non-markdown files

### Git Integration

On verification pass:
1. Stage all changes (`git add -A`)
2. Commit with PLAN's GOAL as message
3. Push (Optional, defaults to **false** in V1.0)

**CLI Overrides:**
- `--no-commit`: Skip git operations completely.
- `--no-push`: Commit locally, do not push.
- `--force`: Skip interactive confirmation.
- `--dry-run`: Show plan and verify integrity without writing files.

---

## Roadmap System (V2)

### The Source of Truth

**`tasks.toml`** is the database. `ROADMAP.md` is a generated artifact.

AI interacts exclusively with `tasks.toml` via the `===ROADMAP===` block protocol.

```toml
[[tasks]]
id = "feature-x"
text = "Implement feature X"
status = "done"
section = "v0.1.0"
test = "tests/feature_x.rs::test_feature"
```

---

## Security Model

### Threat Model

**Attacker:** Malicious or confused AI generating dangerous file operations.

### Defenses

| Threat | Defense |
|--------|---------|
| Partial Apply | Manifest integrity checks + Atomic Rollback |
| Path traversal | Block `..` in any path component |
| Symlink Escape | Resolve & block paths outside root |
| Absolute paths | Block `/` or `C:\` prefixes |
| Sensitive files | Blocklist: `.env`, `.ssh/`, `.aws/`, `.gnupg/`, `credentials` |
| Backup overwrite | Block `.slopchop_apply_backup/` |
| Truncation | Detect comment patterns and lazy phrases |
| Protected files | Block config/lock file overwrites |

---

## Key Decisions & Rationale

### Why Rust?

- **Performance:** Parallel file analysis via rayon
- **Reliability:** No null pointer crashes
- **Tree-sitter:** First-class Rust bindings
- **Single binary:** Easy distribution
- **Dogfooding:** SlopChop enforces its own rules on itself

### Why Tree-sitter Over LSP?

- **No server overhead:** Parse on-demand
- **Language-agnostic queries:** Same patterns for all languages
- **Simpler deployment:** No language server installation

### Why Custom Protocol Over Markdown?

- **Unambiguous:** No fence-escape issues
- **Distinctive:** Delimiters never appear in code
- **Parseable:** Clean regex patterns

### Why Reject Instead of Fix?

- **Teaching:** AI learns through failure
- **Safety:** Auto-fix could mask deeper problems
- **Simplicity:** Rejection is stateless

---

## Future Work

### Watch Mode

`slopchop watch` — Background clipboard monitoring with hotkey application.

The watcher infrastructure exists (`src/tui/watcher.rs`). Remaining work:
- Global hotkey registration
- System notification integration
- Diff preview modal

### Additional Languages

Adding a language requires:
1. Add `tree-sitter-{lang}` dependency
2. Add variant to `Lang` enum in `lang.rs`
3. Implement query methods
4. Add to language detection

### Distribution

Planned for v1.0:
- crates.io publication
- Homebrew formula
- GitHub Releases with binaries

---

*Last updated: 2025-12-13 (V1.0 Hardening)*
