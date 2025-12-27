# Transport Hardening: Markdown / AI Interface Robustness

**Intent:**
SlopChop must tolerate common AI / markdown rendering artifacts during payload ingestion while **preserving a strict, deterministic trust boundary**.
The transport layer is treated as hostile; the apply model remains exact and non-fuzzy.

This document records the observed failure modes and the proposed, intentionally-scoped mitigations.

---

## Problem Statement

SlopChop payloads are frequently copied through AI chat interfaces and markdown renderers that inject formatting noise. This causes valid transactions to fail **before** semantic validation, even though intent and content are correct.

The goal is to recover valid transactions **without**:

* fuzzy matching
* heuristic patching
* weakening safety guarantees
* relaxing truncation or ambiguity rules

---

## Current Failure Modes

### 1) Sigil lines must start at column 0

**Observed behavior:**
Protocol parsing only recognizes headers and footers when `XSC7XSC` appears at column 0 (e.g. `(?m)^XSC7XSC ...`).

**Failure:**
If the transport adds:

* indentation
* list markers (`-`, `*`, `1.`)
* blockquote prefixes (`>`)

…the parser fails to detect `END` blocks and reaches EOF, resulting in errors like:

> `Unclosed block: FILE at byte ...`

---

### 2) Markdown fences inside FILE/PATCH content

**Observed behavior:**
Validation rejects any non-markdown target file that contains:

* triple backticks (```)
* triple tildes (~~~)

**Failure:**
Many UIs inject code fences automatically when copying. A payload that is otherwise correct is rejected during validation.

This is expected behavior **at the semantic layer**, but problematic at the transport boundary.

---

### 3) PATCH metadata parsing is indentation-sensitive

**Observed behavior:**
The v1 PATCH parser computes `trimmed = line.trim()` but then checks metadata using the original line:

* `line.strip_prefix("BASE_SHA256:")`
* `line.strip_prefix("MAX_MATCHES:")`

**Failure:**
Any indentation introduced by the transport breaks PATCH parsing, even though semantic content is correct.

---

## Design Principle: Deterministic Transport Decontamination

Introduce a **pre-validation transport normalization layer** that runs *before* parsing and safety gates.

This layer must be:

* **Deterministic**
  No similarity matching, inference, or guessing.
* **Narrowly scoped**
  Only addresses known renderer artifacts.
* **Audited**
  Reports what was modified (counts + paths).
* **Configurable**
  Strict mode must remain available.

This layer does **not**:

* repair truncation
* resolve ambiguity
* reorder content
* infer intent

---

## Proposed Mitigations

### A) Sigil Line Normalization (Headers / Footers Only)

Allow protocol block detection even when sigil lines are prefixed by markdown noise.

#### Option A1 — Regex Broadening

Accept optional leading:

* whitespace
* `>` blockquote marker
* list markers (`-`, `*`, `+`)
* ordered list prefixes (`1.`, `1)`)

Applied **only** to protocol headers and footers.

#### Option A2 — Sigil Salvage (Preferred)

Before parsing:

* If a line contains the substring `XSC7XSC`, replace the line with the substring starting at the first `XSC7XSC`.
* Otherwise, leave the line unchanged.

Rationale:

* The sigil is statistically absent from real code.
* Avoids complex regex logic.
* Directly targets hostile transport artifacts.

---

### B) Deterministic Sanitizer for FILE / PATCH Content

Strip known markdown transport wrappers **only** for non-markdown targets.

#### Allowed Transformations

* Remove lines where `trimmed_line` matches:

  * ```
    ```
  * ```lang
    ```
  * ```
    ```
  * ```lang
    ```
* Optionally remove standalone language-label lines injected by some UIs.

#### Explicit Non-Goals

* Do **not** repair truncated content (`...`)
* Do **not** infer missing code
* Do **not** reorder or merge lines

#### Audit Requirements

* Emit message:
  `Sanitized N markdown fence lines from <path>`
* Record event with:

  * file path
  * number of removed lines
  * sanitization mode

**Rationale:**
This removes a deterministic transport wrapper that SlopChop already considers invalid for non-markdown files, without weakening correctness guarantees.

**Insertion Point:**
After content extraction (and PATCH application), but before validation and writing.
`processor::process_input` already provides an appropriate staging point.

---

### C) PATCH Metadata Indentation Tolerance

Correctness fix:

* Parse metadata using `trimmed.strip_prefix(...)` instead of `line.strip_prefix(...)` for:

  * `BASE_SHA256:`
  * `MAX_MATCHES:`

This change aligns PATCH parsing with transport hardening and does not affect semantics.

---

### D) Mode Control: Smart Defaults, Explicit Overrides

#### Default Behavior

* **Clipboard input:** enable sanitization by default
* **File input:** strict mode by default

#### Flags

* `slopchop apply --strict`
  Disable all transport sanitization
* `slopchop apply --sanitize`
  Force sanitization for file/stdin input

This preserves trust-boundary clarity while making the common AI workflow reliable.

---

### E) Self-Diagnosing Error Messages

Enhance parser errors with transport-aware hints.

Examples:

* If sigils are detected but not parsed:

  > “Detected indented XSC7XSC sigil lines (likely copied from a renderer). Try `--sanitize` or enable clipboard sanitization.”

* If markdown fences are rejected:

  > “Detected markdown fencing in non-markdown file. Re-run with `--sanitize` to strip transport artifacts.”

---

## Expected Outcome

With these changes, SlopChop becomes resilient to:

* indented or prefixed sigil lines
* list / blockquote formatting
* UI-injected markdown fences
* indented PATCH metadata

While **still rejecting**:

* truncated payloads (`...`)
* ambiguous patches
* unsafe paths or traversal
* non-deterministic apply

---

**Status:** Design notes only.
**Scope:** UX / transport hardening.
**Impact:** Improves real-world reliability without reopening v1.0 trust guarantees.
