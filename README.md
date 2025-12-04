# Warden

**Turn any LLM chat window into a rigorous coding agent.**

Warden is a CLI tool that bridges the gap between your local codebase and AI chat interfaces (like ChatGPT, Claude, or DeepSeek). It turns the "Copy/Paste" workflow from a chore into a reliable, architectural protocol.

Most AI coding goes like this:
1. You copy code.
2. You paste it into ChatGPT.
3. It gives you back code with markdown errors, truncated lines (`// ... rest of code`), or subtle bugs.
4. You manually fix the files.

**Warden changes the game:**
1. **Pack:** `warden pack` bundles your code into a context-optimized format.
2. **Apply:** You paste the AI's response directly into `warden apply`.
3. **Verify:** Warden parses the response, checks for truncation, enforcing architectural rules (The 3 Laws), runs your tests, and **auto-commits** if green.

If the AI breaks a rule (e.g., creates a massive function, hallucinates a file, or skips code), Warden **rejects the change** and generates a specific error message for you to paste back. The AI fixes it, and you try again.

## Is this an Agent?

**No, it's a protocol for Chat-Driven Development.**

Autonomous agents (like Devin or Aider) try to do everything and often get stuck in loops. Warden keeps **you** in the loop as the pilot, using the AI as the engine.

- **You** control the context (`warden pack src/auth`).
- **You** review the plan (Warden forces the AI to output a PLAN block).
- **You** transport the data (Clipboard).
- **Warden** enforces the quality.

It allows "dumb" chat models to act like "smart" senior engineers by constraining their output to a strict format and set of architectural rules.

---

## Installation

```bash
git clone https://github.com/yourusername/warden.git
cd warden
cargo install --path .
```

Verify:
```bash
warden --version
```

**Requirements:**
- Rust toolchain
- System clipboard utilities (`xclip` or `wl-copy` on Linux)

---

## The Workflow

### 1. Context Generation (Pack)

Don't dump your whole repo. Warden packs exactly what the AI needs.

```bash
# Pack the whole project (smartly filtered)
warden pack

# Focus on specific files (others are skeletonized to save tokens)
warden pack src/auth/login.rs
```

This copies a prompt to your clipboard containing your code + **The System Mandate** (rules about complexity, file size, and the output format).

### 2. The Chat

Paste into ChatGPT/Claude. Ask for your feature. The AI has been instructed by Warden's prompt to reply using the **Warden Protocol**:

```text
#__WARDEN_PLAN__#
GOAL: Refactor login logic
CHANGES: ...