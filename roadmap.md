# Warden Protocol Roadmap

## Current State: v0.4.0

The core loop works. You can generate context, chat with AI, apply responses, and verify.

---

## v0.5.0 — Bulletproof Apply

**Theme:** If it applies, it's valid. If it's invalid, it rejects hard.

### Validation Hardening

- [ ] **Truncation detection (smart)**  
  Reject files that are obviously incomplete:
  - Unbalanced braces/brackets (language-aware: skip for Python)
  - Truncation markers: `// ...`, `/* ... */`, `// rest of file`, `// etc`, `// remaining code`
  - Unclosed strings, unclosed block comments
  - Files that end mid-statement (heuristic: ends with `{`, `,`, `(`, `=`)
  
  *Goal: Zero false positives. If Warden rejects it, it was definitely broken.*

- [ ] **Path safety validation**  
  Block dangerous paths before they touch disk:
  - `../` directory traversal
  - Absolute paths (`/etc/passwd`, `C:\Windows\...`)
  - Sensitive targets: `.git/`, `.env`, `.ssh/`, `id_rsa`, `.aws/`, `credentials`
  - Hidden files starting with `.` (configurable)
  
  *Enterprise-grade paranoia.*

- [ ] **Strict format enforcement**  
  If AI doesn't use `<file path="...">` tags, reject immediately with clear error message explaining the required format. No fallback parsing. No guessing. Garbage in = garbage out.

### Workflow Enhancement

- [ ] **Error injection in knit**  
  When `knit --prompt` runs:
  1. Run `warden` scan internally
  2. If violations exist, append them to context:
  ```
  ═══════════════════════════════════════════════════════════════════
  CURRENT VIOLATIONS (FIX THESE)
  ═══════════════════════════════════════════════════════════════════
  
  src/apply/validator.rs:42 [LAW OF COMPLEXITY]
    High Complexity: Score is 12 (Max: 10). Hard to test.
  
  src/lib.rs:156 [LAW OF ATOMICITY]  
    File is 2,341 tokens (Max: 2,000). Split it.
  ```
  
  *AI sees what's broken. AI fixes it. You don't have to explain.*

### Git Integration (Experimental)

- [ ] **`warden apply --commit`**  
  On successful apply:
  1. `git add .`
  2. Auto-generate commit message from `<delivery>` manifest
  3. Commit (no push by default)
  
  Example commit message:
  ```
  warden: update src/apply/validator.rs, add src/apply/safety.rs
  
  Applied via warden apply
  ```

- [ ] **`warden apply --commit --push`**  
  Same as above, but also pushes.

*Philosophy: If it passes validation, commit it. Use git as your undo. Atomic commits per apply.*

### Cut from v0.5

- ~~Backup system~~ — Use git. If you applied broken code, that's on you. `git checkout .` exists.
- ~~Markdown fallback parsing~~ — If AI can't follow format instructions, use a different AI.

---

## v0.6.0 — Intelligence

**Theme:** Understand the code, not just count it.

### Smarter Analysis

- [ ] **Function-level violation reporting**  
  Not just "file has violations" but:
  ```
  src/engine.rs
  
    fn process_batch() [Line 45]
    ├─ Complexity: 14 (max 10)
    ├─ Nesting depth: 5 (max 4)  
    ├─ Contributing factors:
    │   ├─ 3 nested if statements (lines 52, 58, 61)
    │   ├─ 2 match arms with complex guards (lines 67, 89)
    │   └─ while loop with break conditions (line 94)
    └─ Suggestion: Extract inner match to separate function
  
    fn validate_input() [Line 142]
    ├─ Arity: 7 arguments (max 5)
    └─ Suggestion: Group into ValidateOptions struct
  ```
  
  *Learn from the patterns. Understand WHY it's complex.*

- [ ] **Incremental scanning**  
  Only re-analyze changed files:
  - Track file mtimes in `.warden_cache`
  - Or use `git status` to find modified files
  - Full rescan on config change
  
  *Goal: As smart as rustc about what needs recompilation.*

### Smarter Context Generation

- [ ] **Dependency-aware knitting**  
  When file A imports file B:
  - Include B in context even if not explicitly requested
  - Order: dependencies before dependents
  - Show import graph in context header
  
  *AI sees the full picture without you manually selecting files.*

- [ ] **Import graph visualization**  
  `warden deps` or `warden deps src/main.rs`:
  ```
  src/main.rs
  ├─ src/config.rs
  │  └─ src/types.rs
  ├─ src/engine.rs
  │  ├─ src/types.rs
  │  └─ src/analysis.rs
  └─ src/tui/mod.rs
     ├─ src/tui/state.rs
     └─ src/tui/view.rs
  ```

---

## v0.7.0 — Testing & Stability

**Theme:** Trust the tool.

- [ ] **Test suite**
  - Unit tests for each module
  - Integration tests: knit → apply → verify flow
  - Fixture files for each language (Rust, TS, Python)
  - Edge cases: malformed input, huge files, unicode

- [ ] **Performance benchmarks**
  - Scan time vs file count
  - Token counting speed
  - Memory usage on large codebases

- [ ] **CLI stability guarantee**
  - Document all flags and subcommands
  - Semantic versioning discipline
  - Deprecation warnings before removal

---

## v0.8.0 — Ecosystem

**Theme:** CI/CD and tooling integration.

- [ ] **JSON output**: `warden --format json`
- [ ] **SARIF output**: GitHub Code Scanning integration
- [ ] **Exit codes**: Documented, consistent, scriptable
- [ ] **Pre-commit hook**: `warden hook install`
- [ ] **GitHub Action**: Official action for PR checks

---

## v1.0.0 — Release

- [ ] Published to **crates.io**
- [ ] **Homebrew**: `brew install warden` (Mac/Linux package manager)
- [ ] **Scoop/Winget**: Windows package managers
- [ ] Complete documentation site
- [ ] Logo and branding

---

## v2.0.0 — Language Expansion

Way down the line:
- Go
- C/C++ (original Power of 10 target)
- Java/Kotlin

Each language needs: grammar, complexity patterns, naming rules, safety checks.

---

## Future / Speculative

### Metrics Dashboard
Track complexity trends over time. SQLite backend. Charts showing codebase health evolution.

### AI Provider Integration  
When money exists: direct Claude/GPT API calls. `warden chat` command. Self-contained loop without browser.

### Complexity Budget
Instead of per-function limits, allocate complexity budget per file that can be distributed across functions. Some functions can be complex if others are simple.

### Session Branches
`warden session start` creates timestamped branch. Each `warden apply --commit` adds to it. `warden session merge` squashes and merges to main.

---

## Not Doing

- **VS Code Extension** — IDE lock-in, maintenance burden
- **Undo/backup system** — Use git
- **Markdown fallback parsing** — Enforce format discipline
- **Watch mode** — Adds complexity, unclear benefit

---

## Principles

1. **Reject bad input, don't fix it**  
   Warden is a gatekeeper, not a fixer.

2. **Git is the undo system**  
   Don't reinvent version control.

3. **Explicit > Magic**  
   If AI doesn't follow the format, fail loudly.

4. **Learn from violations**  
   Error messages should teach, not just complain.
