# SlopChop Scan: Past, Present, and Future

## The Past: v1.0 (The Linter Era)
We essentially built a linter wrapper.
- **Checks**: Cyclomatic complexity, basic nesting, function arguments.
- **Philosophy**: "Clean code is small code."
- **Failure**: Small code can still be architecturally rotten. We missed:
    - State leakage (public fields with getters).
    - Coupling (everything depending on everything).
    - Design flaws (God classes with low cohesion).

## The Present: v1.6 "Vegetable Grade" (The Metrics Era)
We have successfully built the **Structural Analysis Infrastructure**. This was the "hard part".
- **Philosophy**: "Architecture is quantifiable."
- **Infrastructure**: Tree-sitter AST visitor, Call Graphs, Scope Analysis, Locality Algorithm.
- **Key Wins**:
    - **LCOM4**: We can mathematically prove a class is incoherent.
    - **AHF (Tuned)**: We can distinguish DTOs (Data) from Logic Objects (Behavior) and flag state leakage with high precision.
    - **CBO/SFOUT**: We can detect "God Objects" and architectural bottlenecks.
    - **Multilingual Support**: Rust and TypeScript (via tree-sitter integration).

We have "eaten our vegetables". We have the protein and fiber of a solid static analysis engine.

## The Future: v1.6.1 "The Pattern Frontier"
Now that we have the AST/Graph/Scope infrastructure, we can build the **Semantic Analysis Layer**.
This translates the raw power of our engine into detecting specific *behaviors*, not just structural flaws.

### The Immediate Frontier: AST Patterns
We are standing before 35+ specific semantic checks that rely on the deep understanding we just built.

1.  **State Patterns (S01-S05)**
    *   *Why*: Because `static mut` is the root of all evil.
    *   *How*: Use our Scope analysis to detect mutable globals and deep mutations.
    
2.  **Concurrency Patterns (C01-C05)** - **HIGH VALUE**
    *   *Why*: Async race conditions are impossible to debug at runtime.
    *   *How*: Use our AST Visitor to trace variables across `.await` points (the "Async Race Gap").

3.  **Security Patterns (X01-X05)**
    *   *Why*: "Don't roll your own crypto" and "Don't concat SQL".
    *   *How*: Regex + AST structure matching (e.g., `format!("SELECT ... {}", var)`).

4.  **Resource Patterns (R01-R07)**
    *   *Why*: Memory leaks in long-running processes (unclosed handles, listeners).
    *   *How*: Use Variable Lifecycle Analysis (already partially built for Scope) to track creation vs. disposal.

### The Strategy
We will tackle this frontier one category at a time, starting with **State** or **Concurrency**, as these provide the highest "God Tier" value that standard linters miss.

**Status:** Ready to execute. Infrastructure is green. All systems go.
