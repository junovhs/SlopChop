# 🛡️ Warden Protocol

**Structural governance for AI-assisted development.**

> *"The rules are like the seat belts in a car: Initially, using them is perhaps a little uncomfortable, but after a while, it becomes second nature, and not using them is unimaginable."*  
> — Gerard J. Holzmann, NASA/JPL

Warden enforces **NASA "Power of 10" Rules** adapted for modern development. It's not a style linter—it's an architectural MRI that keeps code AI-readable and human-verifiable.

**Languages:** Rust, TypeScript, JavaScript, Python

---

## The Problem

AI-generated code drifts. Functions bloat. Complexity creeps. Context windows overflow.

You paste 50KB into Claude, it generates a 400-line function with 6 levels of nesting, and now you're debugging something neither you nor the AI can reason about.

Warden stops this at the source.

---

## The 3 Laws

### 1. Law of Atomicity
Files must be **< 2000 tokens**.

Small files fit in context windows. Small files are verifiable. When a file grows too large, split it.

### 2. Law of Complexity  
- **Cyclomatic Complexity:** ≤ 10 per function (configurable down to 4 for "Spartan" mode)
- **Nesting Depth:** ≤ 4 levels (no arrow code)
- **Function Arguments:** ≤ 5 parameters (use structs)

If you can't read a function in one breath, it's too complex.

### 3. Law of Paranoia
- No `.unwrap()` or `.expect()` (Rust)
- Fallible operations must return `Result`

The type system is your ally. Don't lie to the compiler.

---

## Installation

```bash
cargo install --path .
```

This installs two binaries: `warden` and `knit`.

---

## The Workflow

Warden isn't just a linter—it's a closed-loop system for AI development.

### 1. Generate Context

```bash
knit --prompt
```

Creates `context.txt` containing:
- Your codebase (filtered, deduplicated)
- The Warden Protocol system prompt
- Output format specification for AI responses
- Token count

### 2. Chat with AI

Drag `context.txt` into Claude/GPT/Gemini. Ask for changes.

The AI will respond with:

```xml
<delivery>
src/lib.rs
src/new_module.rs [NEW]
</delivery>

<file path="src/lib.rs">
// complete file contents
</file>

<file path="src/new_module.rs">
// complete file contents
</file>
```

### 3. Apply Changes

Copy the AI's entire response (Cmd+A, Cmd+C), then:

```bash
warden apply
```

This:
- Parses `<delivery>` manifest and `<file>` blocks
- Validates: Are all declared files provided?
- Creates timestamped backups in `.warden_apply_backup/`
- Writes files atomically
- On failure: generates AI-friendly error message, copies to clipboard

### 4. Verify

```bash
warden
```

Runs structural analysis. If violations exist, exit code is non-zero.

For full verification including your language linter:

```bash
warden run check
```

Runs whatever command is configured in `warden.toml` (e.g., `cargo clippy`, `biome check`).

### 5. Iterate

If `warden apply` fails, the error message is already in your clipboard. Paste it back to the AI, get corrected output, repeat.

---

## Commands

| Command | Description |
|---------|-------------|
| `warden` | Run structural scan |
| `warden --ui` | Interactive TUI dashboard |
| `warden --init` | Create `warden.toml` |
| `warden apply` | Apply AI response from clipboard |
| `warden apply --dry-run` | Validate without writing |
| `warden run check` | Run configured check command |
| `warden run fix` | Run configured fix command |
| `warden prompt` | Print system prompt |
| `warden prompt -c` | Copy system prompt to clipboard |
| `knit` | Generate context.txt |
| `knit --prompt` | Include system prompt in context |
| `knit --stdout` | Output to stdout instead of file |
| `knit --format xml` | Use XML CDATA format |

---

## Configuration

Run `warden --init` to generate `warden.toml`:

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4
max_function_args = 5
max_function_words = 3
ignore_naming_on = ["tests", "spec"]

[commands]
check = "cargo clippy --all-targets -- -D warnings -D clippy::pedantic"
fix = "cargo fmt"
```

### Strict Mode (Spartan)

For maximum discipline:

```toml
[rules]
max_file_tokens = 1500
max_cyclomatic_complexity = 4
max_nesting_depth = 2
max_function_args = 4
```

### TypeScript/Web Projects

```toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4

[commands]
check = "npx @biomejs/biome check src/"
fix = "npx @biomejs/biome check --write src/"
```

**Windows Note:** Use `npx.cmd` instead of `npx`.

---

## TUI Dashboard

```bash
warden --ui
```

| Key | Action |
|-----|--------|
| `j/k` | Navigate files |
| `s` | Cycle sort mode (name/size/errors) |
| `f` | Toggle error filter |
| `q` | Quit |

---

## Output Format (for AI)

When you include `--prompt` in your knit command, the AI is instructed to respond with:

```xml
<delivery>
path/to/file1.rs
path/to/file2.rs [NEW]
path/to/obsolete.rs [DELETE]
</delivery>

<file path="path/to/file1.rs">
[complete file contents - no truncation]
</file>

<file path="path/to/file2.rs">
[complete file contents]
</file>
```

**Rules:**
- Every file in `<delivery>` must have a matching `<file>` block (except `[DELETE]`)
- `[NEW]` marks files being created
- `[DELETE]` marks files to remove
- Files must be complete—no `// ... rest of file` truncation
- Paths are relative to project root

---

## Philosophy

Warden exists because AI-assisted development needs constraints, not suggestions.

The original Power of 10 rules were designed for life-critical systems where bugs kill people. We're not building flight software, but we are building systems that need to remain comprehensible as they evolve through hundreds of AI-human iterations.

Complexity is the enemy. Warden is the checkpoint.

---

## License

MIT
