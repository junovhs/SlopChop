# Warden

**Bring rigor to your conversations with AI.**

Warden is a command-line tool designed for developers who love the conversational workflow of AI (ChatGPT, Claude, DeepSeek) but hate the friction of moving code back and forth.

It doesn't try to replace the Chat UI with a headless agent. It respects that **chatting is thinking**. The back-and-forth is where the architecture happens.

The problem isn't the conversation; it's the **delivery**.
- You paste code, and it breaks because of bad Markdown escaping.
- The AI writes a 300-line "God Function" that works but ruins your architecture.
- The AI gets lazy and gives you `// ... rest of implementation`.
- You lose track of what you've actually finished vs. what you just talked about.

**Warden bridges the gap between the chat window and your compiler.** It ensures that when you decide to apply the AI's advice, it is **Deterministic**, **Atomic**, and **Safe**.

---

## The Philosophy: Chat-Driven Development

We believe that **slowing down** to articulate your problem to an AI is a feature, not a bug. It forces you to think. But once the thinking is done, the *doing* should be instant and error-free.

Warden acts as a **Gatekeeper**.

1. **Pack** your context (smartly filtered) to the clipboard.
2. **Converse** with the AI in your browser.
3. **Apply** the result via a rigorous protocol.

If the AI produces garbage, Warden rejects it and tells the AI *exactly why*.

---

## How Warden Changes the Game

### 1. The Protocol (Certainty in Transport)
Markdown code blocks (` ```rust `) are fragile. AIs mess them up constantly.

Warden teaches the AI a specific, delimiter-based protocol (`#__WARDEN_FILE__#`). When you run `warden apply`, it doesn't just regex-match text; it parses a structured manifest.

**Result:** No more copy-paste errors. No more "files ending in the middle." If the protocol is valid, the file lands.

### 2. The Three Laws (Enforced Modularity)
AIs love to write spaghetti code. They don't have to maintain it—you do. Warden enforces architectural discipline at the gate.

If an AI tries to give you a complex function:
> **Warden:** "Error: Function `process_data` has a Cyclomatic Complexity of 12 (Max: 8). Refactor."

You paste that error back to the chat. The AI apologizes and breaks the function into three smaller, testable helpers.

**Result:** You don't just get working code; you get **clean, atomic code**.

### 3. The Anti-Lazy Defense (No Truncation)
We've all seen it:
```rust
fn complicated_logic() {
    // ... existing logic ... // warden:ignore
    new_logic();
}
```
If you paste this, you delete your source code.

Warden detects these "lazy markers" (comments, placeholders, ellipses) and **rejects the file immediately**, forcing the AI to provide the complete, compile-ready source.

### 4. Shared Memory (The Roadmap)
Chats are ephemeral. Context windows forget.

Warden maintains a `ROADMAP.md` that serves as the project's long-term memory. The AI can read it to know where we are, and issue commands to update it:

```
===ROADMAP===
CHECK "auth-login"
ADD "auth-logout" AFTER "auth-login"
===ROADMAP===
```

**Result:** One paste updates your code **and** your project status.

---

## A Typical Session

**You:** "I need to fix the login bug. Let's look at the auth module."
```bash
warden pack src/auth/
# Context is now on your clipboard.
```

**You (in ChatGPT):** [Paste Context] "The login handler is failing when..."

**AI:** "I see the issue. Here is the fix."
*[AI outputs a Warden Protocol block]*

**You:** [Copy Response]
```bash
warden apply
```

**Warden:**
```text
❌ Validation Failed
- src/auth/login.rs: High Complexity: Score is 10 (Max: 8).
- src/auth/login.rs: Detected lazy truncation marker: '// ...'.
```
*[Warden copies this error report to your clipboard]*

**You (in ChatGPT):** [Paste Error]

**AI:** "Apologies. I will refactor to reduce complexity and provide the full file."
*[AI outputs corrected code]*

**You:** [Copy Response]
```bash
warden apply
```

**Warden:**
```text
✅ Apply successful!
   ✓ src/auth/login.rs
   ✓ src/auth/helpers.rs [NEW]
   
running tests... passed.
git committing... done.
```

You just refactored a module without writing a line of code, and you have **certainty** that it meets your quality standards.

---

## Installation

```bash
cargo install --path .
```

(Requires Rust toolchain. Supports Linux, macOS, and Windows via WSL.)

## Configuration (`warden.toml`)

Warden is opinionated, but you can negotiate.

```toml
[rules]
max_file_tokens = 2000              # Keep files small
max_cyclomatic_complexity = 8       # Keep logic simple
max_nesting_depth = 3               # Keep indentation flat
max_function_args = 5               # Keep interfaces clean

[commands]
# Warden runs these before committing. If they fail, the apply is rejected.
check = ["cargo test", "npm test"]
fix = "cargo fmt"
```

## "Is this an Agent?"

No. Agents (like Devin) try to be the pilot. **Warden keeps you as the pilot.**

Warden is the navigation system and the safety interlocks. It allows you to use the most powerful LLMs available (which are currently chat-based) without sacrificing the integrity of your local codebase.

---

*MIT License*