# Warden

**The Code Guardian for AI-Assisted Development**

Warden is a code quality enforcement tool designed for developers who use AI assistants (like Claude, GPT, etc.) for coding. It creates a closed-loop workflow where your codebase is packaged with quality constraints, sent to an AI, and the AI's response is validated and applied—automatically committing and pushing only when all rules pass.

Think of it as a gatekeeper that ensures AI-generated code meets structural quality standards before it ever touches your repository.

---

## Philosophy: The Three Laws

Warden enforces three fundamental laws designed to keep code maintainable, testable, and safe. These aren't style preferences—they're structural constraints that prevent the kind of complexity that makes codebases unmaintainable.

### 1. Law of Atomicity
**Files must be small enough to reason about.**

```
Files MUST be < 2000 tokens
```

Large files are cognitive black holes. When a file exceeds ~2000 tokens, it's doing too much. Warden forces you (and your AI assistant) to split large files into focused modules. This makes code easier to review, test, and understand.

### 2. Law of Complexity
**Functions must be simple enough to test.**

```
Cyclomatic Complexity: MUST be ≤ 8 per function
Nesting Depth:         MUST be ≤ 3 levels
Function Arguments:    MUST be ≤ 5 parameters
```

Complex functions are untestable functions. If a function has too many branches, it has too many test cases. If it's nested too deep, the logic is too convoluted. If it takes too many arguments, it has too many responsibilities.

### 3. Law of Paranoia (Rust-specific)
**Handle errors explicitly. No shortcuts.**

```
Use Result<T, E> for I/O and fallible operations
NO .unwrap() or .expect() calls
```

In Rust codebases, Warden bans `.unwrap()` and `.expect()` calls. These are time bombs—they work until they don't, and then your program panics. Use `?`, `unwrap_or`, `unwrap_or_default`, or handle the error explicitly.

---

## Installation

### From Source (Rust required)

```bash
git clone https://github.com/yourusername/warden.git
cd warden
cargo install --path .
```

### Verify Installation

```bash
warden --version
```

---

## Quick Start

### 1. Initialize Your Project

```bash
cd your-project
warden --init
```

This launches an interactive wizard that:
- Detects your project type (Rust, Node/TypeScript, Python, Go)
- Asks for strictness level (Strict, Standard, Relaxed)
- Generates a `warden.toml` configuration file

Alternatively, Warden auto-generates a default config on first run.

### 2. Scan Your Codebase

```bash
warden
```

This analyzes every file and reports violations:

```
LAW VIOLATIONS DETECTED

src/complex.rs
  ├─ Line 45: High Complexity: Score is 12 (Max: 8). Hard to test.
  └─ Line 89: Deep Nesting: Max depth is 5 (Max: 3). Extract logic.

src/big_file.rs
  └─ File exceeds 2000 tokens (3,456 found). Split this file.

Summary: 3 violations in 2 files
```

### 3. Use the TUI Dashboard

```bash
warden --ui
```

Opens an interactive terminal interface showing:
- Health score (percentage of clean files)
- File list with violation counts
- Inspector panel with detailed violation info
- Keyboard navigation (j/k to move, s to sort, f to filter errors)

---

## The Workflow: AI-Assisted Development

Warden's superpower is its integration with AI coding assistants. Here's the complete workflow:

### Step 1: Pack Your Codebase

```bash
warden pack
```

This generates `context.txt` containing:
- The system prompt (The 3 Laws + Nabla Protocol instructions)
- Your entire codebase in Nabla format
- Any existing violations flagged for priority fixing
- A reminder footer with constraints

The file path is automatically copied to your clipboard for easy attachment.

**Options:**
```bash
warden pack --copy          # Copy content directly to clipboard
warden pack --stdout        # Print to stdout (for piping)
warden pack --skeleton      # Compress all files to signatures only
warden pack src/main.rs     # Focus mode: full content for target, skeleton for rest
warden pack --noprompt      # Skip the system prompt (just raw code)
warden pack --git-only      # Only include git-tracked files
warden pack --code-only     # Skip non-code files (README, etc.)
```

### Step 2: Send to Your AI

Paste or attach `context.txt` to your AI conversation. The system prompt teaches the AI:
- The three laws and their limits
- The Nabla Protocol format for responses
- That it must never truncate files

### Step 3: Get AI Response

The AI responds with changes in Nabla format:

```
∇∇∇ PLAN ∇∇∇
GOAL: Refactor authentication module
CHANGES:
1. Extract validation logic to new file
2. Simplify the login function
∆∆∆

∇∇∇ MANIFEST ∇∇∇
src/auth/mod.rs
src/auth/validate.rs [NEW]
src/old_auth.rs [DELETE]
∆∆∆

∇∇∇ src/auth/mod.rs ∇∇∇
// src/auth/mod.rs
pub mod validate;

pub fn login(user: &str, pass: &str) -> Result<Session, AuthError> {
    validate::check_credentials(user, pass)?;
    Session::create(user)
}
∆∆∆

∇∇∇ src/auth/validate.rs ∇∇∇
// src/auth/validate.rs
use crate::error::AuthError;

pub fn check_credentials(user: &str, pass: &str) -> Result<(), AuthError> {
    if user.is_empty() {
        return Err(AuthError::EmptyUsername);
    }
    // validation logic...
    Ok(())
}
∆∆∆
```

### Step 4: Apply the Changes

Copy the AI's response to your clipboard, then:

```bash
warden apply
```

Warden will:
1. **Show the PLAN** and ask for confirmation
2. **Validate all paths** (blocks `../`, absolute paths, `.git/`, `.env`, etc.)
3. **Check for truncation** (detects `// ...`, `/* remaining code */`, etc.)
4. **Create backups** of all modified files
5. **Write the changes** to disk
6. **Run verification** (`warden check` commands from config)
7. **Auto-commit and push** if everything passes

If validation fails, Warden:
- Shows you exactly what went wrong
- Generates a rejection message
- Copies it to your clipboard
- You paste it back to the AI for correction

### The Loop

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│ warden pack │────▶│   Send to   │────▶│   AI edits  │
│             │     │     AI      │     │   codebase  │
└─────────────┘     └─────────────┘     └─────────────┘
       ▲                                       │
       │                                       ▼
       │                               ┌─────────────┐
       │            ❌ Rejected        │ warden apply│
       └───────────────────────────────│  (validate) │
                                       └──────┬──────┘
                                              │ ✅ Valid
                                              ▼
                                       ┌─────────────┐
                                       │  Auto-commit │
                                       │  & push      │
                                       └─────────────┘
```

---

## The Nabla Protocol

The Nabla Protocol is the structured format Warden uses to parse AI responses. It uses the `∇∇∇` (nabla) and `∆∆∆` (delta) symbols as delimiters.

### Why Not Markdown Code Fences?

AI models often nest markdown fences incorrectly or escape them. The nabla symbols are unambiguous and never conflict with code content.

### Block Types

**PLAN Block** (optional but recommended):
```
∇∇∇ PLAN ∇∇∇
GOAL: What you're trying to achieve
CHANGES:
1. First change
2. Second change
∆∆∆
```

**MANIFEST Block** (declares all files being touched):
```
∇∇∇ MANIFEST ∇∇∇
path/to/file1.rs
path/to/file2.rs [NEW]
path/to/old_file.rs [DELETE]
∆∆∆
```

**File Blocks** (one per file):
```
∇∇∇ path/to/file.rs ∇∇∇
// Complete file content here
// NO truncation allowed
∆∆∆
```

### Critical Rules

1. **Every file in MANIFEST must have a matching file block** (unless marked `[DELETE]`)
2. **Paths must match exactly** between manifest and file blocks
3. **No truncation** — files must be complete. No `// ...` or `/* rest of code */`
4. **No markdown fences** — the nabla delimiters ARE the fence

---

## Commands Reference

### Core Commands

| Command | Description |
|---------|-------------|
| `warden` | Scan codebase and report violations |
| `warden --ui` | Open TUI dashboard |
| `warden --init` | Run configuration wizard |

### Workflow Commands

| Command | Description |
|---------|-------------|
| `warden pack` | Generate context.txt for AI |
| `warden apply` | Apply changes from clipboard |
| `warden check` | Run configured check commands |
| `warden fix` | Run configured fix commands |
| `warden prompt` | Output just the system prompt |

### Pack Options

| Flag | Description |
|------|-------------|
| `--copy` / `-c` | Copy to clipboard instead of file |
| `--stdout` / `-s` | Print to stdout |
| `--skeleton` | Skeletonize all files |
| `--noprompt` | Exclude system prompt |
| `--git-only` | Only git-tracked files |
| `--no-git` | Ignore git status |
| `--code-only` | Skip non-code files |
| `--verbose` / `-v` | Show progress |
| `<TARGET>` | Focus mode: full file for target |

### Roadmap Commands

Warden includes a programmatic roadmap system for tracking features:

| Command | Description |
|---------|-------------|
| `warden roadmap init` | Create a ROADMAP.md template |
| `warden roadmap show` | Display roadmap as tree |
| `warden roadmap tasks` | List all tasks |
| `warden roadmap tasks --pending` | List incomplete tasks |
| `warden roadmap tasks --complete` | List completed tasks |
| `warden roadmap prompt` | Generate AI prompt for roadmap work |
| `warden roadmap apply` | Apply roadmap changes from clipboard |
| `warden roadmap audit` | Verify test anchors exist |

---

## Configuration

### warden.toml

```toml
[rules]
max_file_tokens = 2000           # Law of Atomicity
max_cyclomatic_complexity = 8     # Law of Complexity
max_nesting_depth = 3             # Law of Complexity
max_function_args = 5             # Law of Complexity
max_function_words = 5            # Naming complexity

# Patterns to skip for specific rules
ignore_naming_on = ["tests", "spec"]
ignore_tokens_on = ["lock", ".md"]

[commands]
# Commands run during `warden check` and after `warden apply`
check = [
    "cargo clippy --all-targets -- -D warnings",
    "cargo test"
]
# Commands run during `warden fix`
fix = "cargo fmt"
```

### Strictness Levels

**Strict** (Greenfield projects):
- 1500 tokens per file
- Complexity ≤ 6
- Nesting ≤ 2

**Standard** (Recommended):
- 2000 tokens per file
- Complexity ≤ 8
- Nesting ≤ 3

**Relaxed** (Legacy codebases):
- 3000 tokens per file
- Complexity ≤ 12
- Nesting ≤ 4

### .wardenignore

Like `.gitignore`, but for Warden. Files matching these patterns are excluded from scanning and packing.

```
# .wardenignore
target
node_modules
.git
*.lock
dist/
```

### File-Level Ignores

Add a comment at the top of any file to skip Warden analysis:

```rust
// warden:ignore
```
```python
# warden:ignore
```
```html
<!-- warden:ignore -->
```

---

## Skeleton Mode

For large codebases, sending full file contents may exceed AI context limits. Skeleton mode compresses files to just their signatures:

**Original Rust file:**
```rust
pub fn process_data(input: &str) -> Result<Output, Error> {
    let parsed = parse_input(input)?;
    let validated = validate(parsed)?;
    transform(validated)
}

fn parse_input(s: &str) -> Result<Parsed, Error> {
    // 50 lines of parsing logic
}
```

**Skeletonized:**
```rust
pub fn process_data(input: &str) -> Result<Output, Error> { ... }

fn parse_input(s: &str) -> Result<Parsed, Error> { ... }
```

Use `warden pack --skeleton` to skeletonize everything, or `warden pack src/target.rs` to focus on one file while skeletonizing the rest.

---

## Security

Warden blocks dangerous operations in AI responses:

**Path Traversal:**
- `../` sequences → Blocked
- Absolute paths (`/etc/passwd`, `C:\Windows`) → Blocked

**Sensitive Locations:**
- `.git/` → Blocked
- `.env` → Blocked
- `.ssh/` → Blocked
- `.aws/` → Blocked
- Credentials files → Blocked

**Protected Files:**
- `ROADMAP.md` → Managed programmatically only

**Hidden Files:**
- Dotfiles → Blocked (except `.gitignore`, `.wardenignore`)

**Truncation Detection:**
- `// ...` → Rejected
- `/* ... */` → Rejected
- `# ...` → Rejected
- `"remaining code"` phrases → Rejected
- Empty files → Rejected

---

## Backups

Before applying any changes, Warden creates backups in `.warden_apply_backup/`:

```
.warden_apply_backup/
└── 1699876543/          # Unix timestamp
    ├── src/
    │   └── modified.rs  # Original content
    └── lib/
        └── changed.rs
```

Add `.warden_apply_backup` to your `.gitignore`.

---

## Language Support

| Language | Complexity Analysis | Paranoia Rules | Skeleton |
|----------|-------------------|----------------|----------|
| Rust | ✅ | ✅ (.unwrap/.expect) | ✅ |
| TypeScript | ✅ | — | ✅ |
| JavaScript | ✅ | — | ✅ |
| Python | ✅ | — | ✅ |
| Go | Detection only | — | — |
| Other | Token counting only | — | — |

---

## Principles

1. **Every feature has a verified test** — The roadmap system enforces this
2. **Reject bad input, don't fix it** — Warden is a gatekeeper, not a linter
3. **Git is the undo system** — Don't reinvent version control
4. **Explicit > Magic** — Fail loudly on format violations
5. **Containment over craftsmanship** — Constraints are safety, not style
6. **Self-hosting** — Warden passes its own rules

---

## License

MIT License — See [LICENSE](LICENSE) for details.

---

## Contributing

1. Fork the repository
2. Run `warden` to ensure you start clean
3. Make your changes (Warden will enforce the 3 Laws)
4. Use `warden pack` to generate context for AI assistance
5. Submit a PR

The codebase is its own best documentation—every file follows the constraints it enforces.
