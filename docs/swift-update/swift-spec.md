This is the **Final Technical Specification** for adding Tier-1 Swift support to SlopChop. It incorporates the architectural constraints of your current codebase, the operational reality of Linux-based iOS development, and the specific corrections identified regarding AST node handling and project detection.

**Note to Implementer:** Do **not** attempt to implement V2 Scope Analysis (LCOM4/CBO) for Swift in this pass. Swift extensions require multi-file scope merging which is out of scope for this milestone.

---

# Specification: Tier-1 Swift Support for SlopChop

## 1. Objectives
1.  **Syntactic Parity:** Enable SlopChop to parse `.swift` files, calculate complexity, and enforce naming conventions using Tree-sitter.
2.  **Law of Paranoia:** Implement Swift-specific bans on unsafe constructs (`!`, `try!`, `as!`) equivalent to Rust's `.unwrap()`.
3.  **Linux-First Workflow:** Configure command generation to support local linting on Linux while deferring build/test logic to CI for Xcode projects.

---

## 2. Dependencies & Discovery

### 2.1 Cargo Dependencies
**File:** `Cargo.toml`
Add `tree-sitter-swift`. **Crucial:** Pin to version `0.3.4` (or closest available compatible version) to ensure compatibility with the project's existing `tree-sitter = "0.20"` lock.

```toml
[dependencies]
# ... existing
tree-sitter-swift = "0.3.4" 
```

### 2.2 File Discovery
**File:** `src/constants.rs`
Update the regex to include `.swift`.

```rust
pub const CODE_EXT_PATTERN: &str = r"(?i)\.(rs|go|py|js|jsx|ts|tsx|java|c|cpp|h|hpp|cs|php|rb|sh|sql|html|css|scss|json|toml|yaml|md|swift)$";
```

---

## 3. Language Definition & Queries

**File:** `src/lang.rs`

### 3.1 Enum Update
Add `Swift` to the `Lang` enum and handle it in `from_ext` and `grammar`.

### 3.2 Tree-Sitter Queries
Add a new entry to the `QUERIES` array (Index 3). Use these specific S-expressions which align with standard Swift grammars.

*   **Naming:** `(function_declaration name: (simple_identifier) @name)`
*   **Complexity:**
    ```scm
    (if_statement) @branch
    (guard_statement) @branch
    (for_statement) @branch
    (while_statement) @branch
    (repeat_while_statement) @branch
    (catch_clause) @branch
    (switch_statement) @branch
    (binary_expression operator: ["&&" "||"]) @branch
    ```
*   **Imports:** `(import_declaration (simple_identifier) @import)`
*   **Defs (Signatures):** Capture `class_declaration`, `struct_declaration`, `enum_declaration`, `protocol_declaration`, `extension_declaration`, and `function_declaration`.
*   **Exports:** Look for modifiers.
    ```scm
    (function_declaration (modifiers) @mod (#match? @mod "public|open")) @export
    (class_declaration (modifiers) @mod (#match? @mod "public|open")) @export
    ```
*   **Skeleton:** `(function_declaration body: (code_block) @body)`, plus `init_declaration` and `deinit_declaration`.

---

## 4. Analysis Logic Updates

### 4.1 AST Analysis & Dispatch
**File:** `src/analysis/ast.rs`

In `run_analysis`, add a dispatch for Swift specifics similar to `check_rust_specifics`.

```rust
if lang == Lang::Swift {
    Self::check_swift_specifics(lang, &ctx, &mut violations);
}
```

Implement `check_swift_specifics` to run the banned checks defined below.

### 4.2 The "Law of Paranoia" (Banned Constructs)
**File:** `src/analysis/checks/banned.rs`

The existing `check_banned` function is tightly coupled to Rust's `identifier == unwrap` logic. Do **not** modify it. Instead, add a new helper:

`pub fn check_banned_query(ctx: &CheckContext, query: &Query, message: &str, out: &mut Vec<Violation>)`

Use this helper in `check_swift_specifics` to ban:
1.  **Force Unwrap:** `(force_unwrapping_expression) @bang`
2.  **Force Try:** `(try_expression) @try` (Requires checking if the child node text contains `!`).
3.  **Force Cast:** `(as_expression) @cast` (Requires checking if the operator is `as!`).

**Violation Message:** "Unsafe construct detected (`!`). Use `if let`, `guard let`, `try?`, or `as?` instead."

### 4.3 Metrics Compatibility
**File:** `src/analysis/metrics.rs`

Update `count_arguments` to recognize Swift's parameter nodes.
*   Current: matches `parameters` | `formal_parameters`.
*   Update: Add `parameter_clause` (standard Swift naming).

---

## 5. Project Detection & Commands

**File:** `src/project.rs`

### 5.1 Project Type Enum
Split Swift into two types to handle the Linux/macOS dichotomy.
```rust
pub enum ProjectType {
    // ... existing
    SwiftSpm,   // Linux compatible
    SwiftXcode, // macOS/CI only
}
```

### 5.2 Detection Logic
1.  **SwiftSpm:** If `Package.swift` exists in root.
2.  **SwiftXcode:** Use `WalkDir` (max depth 3) to find `*.xcodeproj` or `*.xcworkspace`.

### 5.3 Default Commands
*   **SwiftSpm:**
    *   Check: `swiftlint lint --strict`, `swift test`
    *   Fix: `swift-format format -i -r .`
*   **SwiftXcode:**
    *   Check: `swiftlint lint --strict`
    *   *Crucial:* Do **not** default to `xcodebuild`.
    *   Add a dummy command or echo: `echo "⚠️ Logic checks passed. Push to CI for iOS Simulator tests and UI Snapshots."`

---

## 6. Prompt Engineering

**File:** `src/prompt.rs`

Update `build_system_prompt` to include Swift safety in the "LAW OF PARANOIA" section.

*   *Current:* "LAW OF PARANOIA: No .unwrap() or .expect(). Use Result types."
*   *New:* "LAW OF PARANOIA: No .unwrap(), .expect(), or Swift Force Unwraps (`!`, `as!`, `try!`). Use Result/Option/Optional binding."

---

## 7. Verification Strategy (Tests)

You must verify the Tree-sitter queries work, or SlopChop will panic/fail silently.

1.  Create `tests/fixtures/swift/basic.swift`:
    ```swift
    import Foundation
    public class User {
        func parse(data: Data) {
            let x = try! JSONSerialization.jsonObject(with: data) // Violation
            if x == nil { return } // Complexity
        }
    }
    ```
2.  Create a unit test in `src/lang.rs` (or a new test module) that:
    *   Parses this fixture using `Lang::Swift`.
    *   Compiles the Complexity, Naming, and Imports queries.
    *   Asserts matches are found (e.g., verifies `parse` is found as a function name).

---

## 8. Exclusions (V2 Metrics)

**File:** `src/analysis/v2/worker.rs`

Ensure the existing guard clause remains:
```rust
if lang != Lang::Rust { return empty; }
```
This ensures we do not attempt to run LCOM4/CBO on Swift files, which would produce false positives due to extensions.

---

## Summary of Work
1.  **Dep:** `tree-sitter-swift`.
2.  **Lang:** Enum + Queries.
3.  **AST:** New `check_swift_specifics` + generic banned query helper.
4.  **Metrics:** Support `parameter_clause`.
5.  **Project:** Detect Xcode vs SPM; set commands appropriate for Linux.
6.  **Prompt:** Warn AI about `!`.
7.  **Tests:** Fixture verification.

Here is the updated **Section 9** to be appended to the Technical Spec. This details exactly how to verify the implementation using your specific **Linux-local / CI-remote** setup.

---

## 9. Verification & Testing Plan

This section defines how to prove SlopChop works for Swift once the code changes are applied.

### 9.1 Phase 1: rust-level Unit Tests (The "It Compiles" Check)
*Goal: Ensure SlopChop's Rust binary can parse Swift without crashing.*

**Action:** Create a new test file `tests/swift_grammar.rs` (or add to `src/lang.rs`).

```rust
#[test]
fn test_swift_paranoia_detection() {
    // 1. Setup Swift source with a violation
    let source = r#"
        func riskyBusiness() {
            let data = try! loadData() // Banned: try!
            let val = data! // Banned: Force Unwrap
        }
    "#;

    // 2. Parse
    let lang = Lang::Swift;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(lang.grammar()).unwrap();
    let tree = parser.parse(source, None).unwrap();

    // 3. Run Checks (Mocking context)
    // Note: You will need to expose a helper to run specific checks for testing
    // or run the full analysis stack if your test harness supports it.
    
    // ASSERT: Logic should find 2 violations (try! and !)
}
```

**Run:** `cargo test` on Linux.
**Success Criteria:** Tests pass, confirming Tree-sitter is correctly identifying the nodes.

---

### 9.2 Phase 2: Local "Smoke Test" (Linux)
*Goal: Verify `slopchop scan` works on Linux even without the Swift toolchain.*

1.  **Setup Sandbox:**
    ```bash
    mkdir -p /tmp/slop-swift/src
    cd /tmp/slop-swift
    # Create a Package.swift so SlopChop thinks it's a SwiftSpm project
    touch Package.swift 
    ```

2.  **Create "Bad" Swift:**
    Create `/tmp/slop-swift/src/Bad.swift`:
    ```swift
    // Trigger "Max Args" (assuming config limit is 5)
    func godFunction(a: Int, b: Int, c: Int, d: Int, e: Int, f: Int) {
        // Trigger "Law of Paranoia"
        let danger = someOptional!
    }
    ```

3.  **Run SlopChop:**
    Run your modified SlopChop binary against this folder.
    ```bash
    /path/to/your/target/debug/slopchop scan
    ```

4.  **Success Criteria:**
    *   Output should show **2 Violations**:
        1.  `Max args exceeded (6 > 5)`
        2.  `Unsafe construct detected (!)`
    *   If you see these, the **Rust side is done.**

---

### 9.3 Phase 3: The Toolchain Test (Linux + SwiftLint)
*Goal: Verify `slopchop check` runs the linter locally.*

1.  **Install SwiftLint on Linux:**
    (If you haven't already, this is required for local checking).
    *   *Option A:* Download binary from [SwiftLint Releases](https://github.com/realm/SwiftLint/releases).
    *   *Option B:* If that's too annoying, mock it for now by creating a dummy script at `/usr/local/bin/swiftlint` that just echoes "Linting...".

2.  **Run Check:**
    ```bash
    /path/to/your/target/debug/slopchop check
    ```

3.  **Success Criteria:**
    *   SlopChop detects `ProjectType::SwiftSpm`.
    *   It executes the command `swiftlint lint --strict`.
    *   It reports the output.

---

### 9.4 Phase 4: The "Full Loop" (CI Handshake)
*Goal: Verify the Linux -> GitHub -> macOS loop.*

1.  **Repo Setup:**
    Create a new private repo (or use a branch on your existing one). Push the `.github/workflows/ios.yml` defined in the previous steps.

2.  **The Test Commit:**
    On Linux, commit a Swift file that causes a compile error (e.g., `import NonExistentLibrary`).

3.  **Push:**
    `git push origin main`

4.  **Observe:**
    *   Go to GitHub Actions tab.
    *   Watch `iOS UI & Logic` job pick up the change.
    *   **Success:** The job fails during the `xcodebuild test` step.

5.  **The Fix:**
    *   Correct the code on Linux.
    *   Push again.
    *   **Success:** The job turns Green.

---

## 10. Summary Checklist for "Done"

- [ ] `cargo test` passes with new Swift grammar enabled.
- [ ] `slopchop scan` correctly identifies `!` unwraps in a `.swift` file on Linux.
- [ ] `slopchop pack` generates a prompt that mentions "No Force Unwraps".
- [ ] GitHub Actions workflow successfully boots a macOS runner and runs `xcodebuild`.
