# SlopChop Semantic Tree

This document provides a comprehensive summary of every file in the SlopChop codebase, designed to help an AI or developer navigate the system efficiently. SlopChop is a high-integrity code quality governance and transactional change management tool for AI-assisted development.

---

## Root Files

### [AGENT-README.md](file:///home/juno/slopchop/AGENT-README.md)
The SlopChop Autonomous Protocol (SAP) guide for AI agents. It defines the mandatory transactional workflow (Isolate, Act, Verify, Decide) and the "Laws of Physics" (structural metrics like token limits and cognitive complexity) that agents must adhere to. It emphasizes the importance of the `slopchop-work` sandbox and prohibits direct commits to `main`. This is the primary instruction set for any AI operating within this repository to ensure high-integrity changes.

### [Cargo.toml](file:///home/juno/slopchop/Cargo.toml)
The project's manifest file, defining the `slopchop` binary and the `slopchop_core` library. It lists essential dependencies including `anyhow` for errors, `clap` for CLI parsing, `tree-sitter` for AST analysis, and `tiktoken-rs` for tokenization. It configures release profiles for optimization (LTO, stripped symbols) and development profiles for speed. This file establishes the technical foundation, linking the various core modules and external libraries that power the SlopChop engine.

### [slopchop.toml](file:///home/juno/slopchop/slopchop.toml)
The default configuration file for SlopChop rules. it defines threshold values for structural analysis, such as `max_file_tokens` (2000), `max_cognitive_complexity` (25), and `max_nesting_depth` (3). It also configures directory/file ignore patterns, safety requirements for `unsafe` code, and the external commands (like `clippy` and `cargo test`) used during the verification gate. This central configuration allows organizations to customize the governance levels enforced by the SlopChop tool on their codebase.

### [README.md](file:///home/juno/slopchop/README.md)
The general project documentation providing an overview of SlopChop's purpose and usage. (Note: Full content not read yet, but it serves as the standard entry point for developers exploring the repo, typically covering installation, command usage, and the project's philosophy).

---

## src/ Root Files

### [main.rs](file:///home/juno/slopchop/src/main.rs)
A minimal stub currently used for testing purposes. In the final binary architecture, the real entry point is located in `src/bin/slopchop.rs`. This file remains a placeholder or a secondary entry point that currently does not hold the primary logic of the application but exists as part of the standard Rust project structure.

### [lib.rs](file:///home/juno/slopchop/src/lib.rs)
The library entry point for the `slopchop_core` crate. It serves as the central module registry, exporting all major components of the system including `analysis`, `apply`, `cli`, `graph`, `mutate`, and others. By defining the public module surface, it allows the `slopchop` binary and potentially other tools to consume the core logic of the SlopChop protocol in a structured and organized manner.

### [branch.rs](file:///home/juno/slopchop/src/branch.rs)
Implements the git-based transactional workflow. It provides functions to create, reset, and promote the `slopchop-work` branch. It manages safety checks like ensuring a clean working directory before promotion and uses `git merge --squash` to keep the `main` history clean. This file is critical for the "Isolate" and "Decide" phases of the SlopChop loop, ensuring that AI-generated changes are safely staged and reviewed.

### [clean.rs](file:///home/juno/slopchop/src/clean.rs)
Utility for cleaning up the repository environment. It handles the removal of temporary files like `context.txt` and ensures that such files are listed in `.gitignore` to prevent accidental commits of large context payloads. It integrates with the git workflow, potentially committing these cleanup changes. This helps maintain repository hygiene and prevents AI artifacts from polluting the core source code.

### [constants.rs](file:///home/juno/slopchop/src/constants.rs)
Central repository for shared constants and regex patterns used throughout the system. It defines standard directory/file pruning lists (like `.git`, `target`, `node_modules`) and regexes for identifying code files vs. binary artifacts or secrets. These constants ensure consistency across `discovery`, `map`, and `analysis` modules when identifying which files should be included or excluded from the SlopChop governance process.

### [detection.rs](file:///home/juno/slopchop/src/detection.rs)
Implements the `Detector` component for identifying project build systems. It scans for configuration files like `Cargo.toml`, `package.json`, or `pyproject.toml` to determine if a project is Rust, Node, Python, or Go. This detection is used by the `project` module to automatically generate appropriate SlopChop configurations and CLI commands tailored to the specific language environment of the target repository.

### [discovery.rs](file:///home/juno/slopchop/src/discovery.rs)
The file system traversal engine. It uses `walkdir` to recursively search the project, applying heuristics (defined in `constants.rs`) and user-defined patterns to find relevant source code files. It filters out binaries and secrets, and can group discovered files by directory. This module provides the initial list of targets for all subsequent operations like scanning, mapping, or packing context for AI agents.

### [error.rs](file:///home/juno/slopchop/src/error.rs)
A backward-compatibility wrapper for error handling. It re-exports `anyhow` types to provide a unified error interface across the codebase. While newer modules use `anyhow` directly, this file ensures that older segments of the code remain compatible while the system migrates toward a modern, consistent error-handling strategy that simplifies propagation and context-gathering for failed operations.

### [events.rs](file:///home/juno/slopchop/src/events.rs)
Provides a machine-readable audit trail mechanism. It defines the `SlopChopEvent` structure and `EventLogger`, which appends JSON-LD events to `.slopchop/events.jsonl`. Logged events include stage creation, check results, and file modifications. This audit trail is essential for tracking AI actions over time, providing transparency into what changes were applied and whether they passed governance gates.

### [exit.rs](file:///home/juno/slopchop/src/exit.rs)
Defines standardized process exit codes for the SlopChop CLI. It uses the `SlopChopExit` enum to provide descriptive failure statuses such as `InvalidInput`, `SafetyViolation`, and `CheckFailed`. This allows external CI/CD pipelines and scripts to programmatically react to different failure modes, ensuring that the SlopChop gatekeeper can be integrated robustly into larger automation systems.

### [lang.rs](file:///home/juno/slopchop/src/lang.rs)
The language-specific metadata and query registry. It defines the `Lang` enum and associates each supported language (Rust, Python, TypeScript) with its respective tree-sitter grammar and structural queries (for naming, complexity, and skeletonization). This file is the "dictionary" that the `analysis` and `skeleton` modules use to understand the syntax and structure of diverse codebases.

### [map.rs](file:///home/juno/slopchop/src/map.rs)
Generates a tree-style visualization of the repository structure. It calculates file sizes and token counts while optionally visualizing dependency relationships between files. The output is a high-level overview that helps developers and AI agents understand the codebase's layout at a glance. It bridges `discovery`, `tokens`, and `graph` modules to produce a data-rich representation of the system.

### [project.rs](file:///home/juno/slopchop/src/project.rs)
Orchestrates project-level initialization and configuration. It uses `detection.rs` to identify project types and generates a tailored `slopchop.toml` based on user-specified strictness levels (Strict, Standard, Relaxed). By mapping project types to specific linting and formatting commands (like `clippy`, `eslint`, or `ruff`), it simplifies the setup process for applying SlopChop governance to new repositories.

### [prompt.rs](file:///home/juno/slopchop/src/prompt.rs)
Generates system prompts and reminders for AI agents. It translates the rules from `slopchop.toml` into natural language instructions that inform the AI about its structural constraints and the "DNA sequence" sigil protocol (`XSC7XSC`) required for applying changes. This is the primary interface for communicating governance policies directly to the AI model's context window.

### [reporting.rs](file:///home/juno/slopchop/src/reporting.rs)
Handles the human-readable console output for scan results. It formats violations with line numbers, code snippets, and underlined error locations using the `colored` crate. It provides prescriptive suggestions for fixing structural issues (e.g., "Extract methods") and summarizes the overall scan status. This file ensures that developers receive clear, actionable feedback when the SlopChop gatekeeper rejects a change.

### [skeleton.rs](file:///home/juno/slopchop/src/skeleton.rs)
Implements structural code reduction (skeletonization). It uses tree-sitter to identify function and class bodies and replaces them with placeholders (e.g., `{ ... }`), leaving only the signatures and structural decorators. This "X-ray" view of the code helps reduce token usage when providing codebase context to an AI, allowing it to understand interfaces without being overwhelmed by implementation details.

### [tokens.rs](file:///home/juno/slopchop/src/tokens.rs)
The core tokenization component. It uses the `tiktoken-rs` library with the `cl100k_base` encoding (compatible with GPT-4) to accurately count tokens in source text. It provides functions to check if content exceeds specific limits, which is fundamental to enforcing the "Law of Atomicity." This component ensures that file sizes remain manageable for both human reviewers and AI context windows.

### [types.rs](file:///home/juno/slopchop/src/types.rs)
Defines the central data structures used for analysis and reporting. This includes `Violation`, `FileReport`, and `ScanReport`, which encapsulate the details of structural issues found during a scan. It also contains `CheckReport`, which aggregates scan results with external command outcomes. These shared types facilitate data flow between the `analysis` engine and the `reporting` and `cli` layers.

### [utils.rs](file:///home/juno/slopchop/src/utils.rs)
Provides general-purpose utility functions, specifically focusing on SHA256 hashing with normalized line endings. This ensures that file-integrity checks remain consistent across different operating systems (Windows vs. Unix) by handling line-ending differences (CRLF vs. LF) before computing hashes. This is crucial forSlopChop's transactional tracking and patching mechanisms where exact content matching is required.

---

## src/bin/

### [slopchop.rs](file:///home/juno/slopchop/src/bin/slopchop.rs)
The primary CLI entry point. It handles command-line argument parsing via `clap` and dispatches work to the `cli::dispatch` module. It also manages the top-level process exit state by converting results into `SlopChopExit` codes. This is the user-facing interface that ties together all the core library components into the `slopchop` command-line tool.

---

## src/analysis/

### [mod.rs](file:///home/juno/slopchop/src/analysis/mod.rs)
The module entry point for the analysis engine. it organizes submodules for AST analysis, safety checks, metrics, and the newer V2 analysis system. It exports the `RuleEngine`, which serves as the high-level coordinator for scanning files against the SlopChop ruleset. This file acts as the gateway to the primary intelligence layer of the tool.

### [ast.rs](file:///home/juno/slopchop/src/analysis/ast.rs)
The core AST analysis driver. It uses tree-sitter to parse source files and runs various structural checks, including naming conventions, complexity metrics (via the `CognitiveAnalyzer`), and language-specific rules (like Rust's banned `.unwrap()` calls). It coordinates with the `checks` module to identify violations and build a detailed `AnalysisResult` for each file processed.

### [checks.rs](file:///home/juno/slopchop/src/analysis/checks.rs)
A registry and context provider for individual structural checks. It defines the `CheckContext` (bundling source, AST root, and config) and re-exports modules for `banned` constructs, `complexity`, `naming`, and `syntax`. This file provides the infrastructure for the AST-based analysis system to execute functional, modular rules against the tree-sitter tree.

### [engine.rs](file:///home/juno/slopchop/src/analysis/engine.rs)
Orchestrates the analysis of multiple files. It wraps the `Config` and provides the `scan()` and `scan_with_progress()` entry points. It delegates the heavy lifting to `analysis::logic`, providing a clean API for the CLI to initiate scans while managing progress callbacks. This is the main interface used by the rest of the application to interact with the rule engine.

### [file_analysis.rs](file:///home/juno/slopchop/src/analysis/file_analysis.rs)
The per-file analysis pipeline. It manages file reading, ignore-directive detection, and token counting. Notably, it implements the "Systems Profile" auto-detection, which relaxes structural limits for files containing low-level constructs like `unsafe`, `no_std`, or `Atomic`. This ensures that specialized systems code isn't unfairly penalized by standard application-layer rules.

### [logic.rs](file:///home/juno/slopchop/src/analysis/logic.rs)
The parallel execution layer for scanning. It uses the `rayon` crate to analyze files across multiple CPU cores, greatly improving performance on large codebases. It combines the results of per-file analysis with the global V2 deep analysis (LCOM4/CBO) to produce a unified `ScanReport`. This module ensures that SlopChop remains fast and responsive during heavy use.

### [metrics.rs](file:///home/juno/slopchop/src/analysis/metrics.rs)
Low-level metrics calculation utilities. It provides functions to walk the AST and compute nesting depth, McCabe cyclomatic complexity, and function arity (parameter counts). These pure functions are used by the `checks` and `ast` modules to quantify the structural "slop" in a function, forming the basis for the Law of Complexity enforcement.

### [safety.rs](file:///home/juno/slopchop/src/analysis/safety.rs)
Enforces the "Law of Paranoia" specifically for Rust. it scans for `unsafe` blocks and validates that they are either prohibited by configuration or accompanied by a mandatory `// SAFETY:` justification comment. It includes logic for navigating the AST and checking neighboring comment nodes, ensuring that critical safety-related documentation is present where required by project policy.

---

## src/analysis/checks/

### [banned.rs](file:///home/juno/slopchop/src/analysis/checks/banned.rs)
Implements checks for prohibited code constructs. Currently focused on the "Law of Paranoia," it scans for `.unwrap()` and `.expect()` calls in non-test files. It provides detailed violations with suggestions to use the `?` operator or proper error handling. This helps enforce robust error management practices by preventing the introduction of potential runtime panics in stable production code.

### [complexity.rs](file:///home/juno/slopchop/src/analysis/checks/complexity.rs)
Focuses on arity and nesting depth violations. It scans function definitions and uses the `metrics` module to ensure that parameters and conditional nesting stay within configured limits. While cognitive complexity is handled in V2, this module provides the baseline structural enforcement that prevents the creation of "God functions" with excessively deep or wide interfaces.

### [naming.rs](file:///home/juno/slopchop/src/analysis/checks/naming.rs)
Enforces function naming rules. It counts words in function names (handling both camelCase and snake_case) and alerts on names that are excessively long or verbose. It provides automated suggestions for shorter, more concise names. This encourages developers to favor clear, meaningful identifiers while avoiding descriptive names that might indicate a function is trying to do too much.

### [syntax.rs](file:///home/juno/slopchop/src/analysis/checks/syntax.rs)
The "Law of Integrity" validator. It traverses the AST specifically looking for `ERROR` or `MISSING` nodes produced by the tree-sitter parser. This ensures that SlopChop only accepts code that is syntactically valid according to the target language's grammar, preventing malformed code from passing the governance gate and potentially breaking builds or confusing other tools.

---

## src/analysis/v2/

### [mod.rs](file:///home/juno/slopchop/src/analysis/v2/mod.rs)
The entry point for the V2 analysis subsystem. V2 introduces "Deep Analysis," which looks at global structural metrics like cohesion (LCOM4) and coupling (CBO/SFOUT). This module exports the `ScanEngineV2` and provides constants for "small codebase" detection, which skips these advanced metrics when they would be too noisy for tiny projects.

### [aggregator.rs](file:///home/juno/slopchop/src/analysis/v2/aggregator.rs)
A data collector for deep analysis. It "ingests" results from individual file workers, mapping global scopes (like structs and methods) to their respective file paths. This centralized repository allows the deep analyzer to see the "big picture" of the codebase, enabling it to calculate metrics that require knowledge of relationships between different parts of the system.

### [cognitive.rs](file:///home/juno/slopchop/src/analysis/v2/cognitive.rs)
Implements the Cognitive Complexity metric based on the SonarSource specification. Unlike cyclomatic complexity, this metric measures how hard code is for a human to understand, penalizing nesting and branching more heavily. It uses a tree-sitter visitor to assess control flow structures and logic gates, providing a more intuitive measure of "tangled" logic than simple branch counting.

### [deep.rs](file:///home/juno/slopchop/src/analysis/v2/deep.rs)
The runner for global structural analysis. It iterates through the global scopes gathered by the `Aggregator` and uses the `Inspector` to find violations. It maps these "deep" violations back to their original file locations. This module bridges the gap between local per-file parsing and the cross-file metrics that define SlopChop V2's advanced governance capabilities.

### [engine.rs](file:///home/juno/slopchop/src/analysis/v2/engine.rs)
The main execution controller for V2 deep scans. It implements the "small codebase" logic, skipping advanced metrics if the project has fewer than 10 source files. It orchestrates the two-phase process: Phase 1 (parallel local analysis to collect scopes) and Phase 2 (global inspection for cross-file structural violations). This ensures SlopChop scales effectively while remaining relevant for projects of all sizes.

### [inspector.rs](file:///home/juno/slopchop/src/analysis/v2/inspector.rs)
The rule enforcer for deep metrics. It examines `Scope` objects to check for cohesion (LCOM4), encapsulation (AHF), and coupling (CBO/SFOUT). It includes logic to skip checks on "data structs" (e.g., those deriving `Serialize`) where high visibility is expected. This module provides the high-level logic that translates raw metrics into prescriptive violations for developers.

### [metrics.rs](file:///home/juno/slopchop/src/analysis/v2/metrics.rs)
The specialized calculator for V2 structural metrics. It contains the logic for LCOM4 (calculating connected components of methods based on field access), CBO (counting unique external dependencies), SFOUT (max external calls), and AHF (percentage of private fields). These algorithms provide the mathematical foundation for assessing the modular integrity and maintainability of the codebase.

### [rust.rs](file:///home/juno/slopchop/src/analysis/v2/rust.rs)
The Rust-specific structure extractor. It uses tree-sitter to find `struct`, `enum`, and `type` definitions, extracting their fields and attributes (like `derive` macros). It coordinates with `rust_impl.rs` to associate methods with their parent types. This module is essential for building the "Scope Map" that V2 needs to perform its behavior-aware structural analysis on Rust codebases.

### [rust_impl.rs](file:///home/juno/slopchop/src/analysis/v2/rust_impl.rs)
Extracts method-level behavior from Rust `impl` blocks. It recursively walks function bodies to identify field accesses (`self.field`), internal calls (`self.method()`), and external calls. It also calculates cognitive complexity for each method. By separating this from structural extraction, SlopChop manages complexity and prevents "God file" issues in its own implementation.

### [scope.rs](file:///home/juno/slopchop/src/analysis/v2/scope.rs)
Defines the core data models for V2 analysis. The `Scope` struct represents a class, struct, or enum, while `Method` and `FieldInfo` capture their internal details. These models are designed to store not just names, but relationships (like which method accesses which field), enabling the complex graph-based metrics that SlopChop V2 uses to enforce high-level design principles.

### [visitor.rs](file:///home/juno/slopchop/src/analysis/v2/visitor.rs)
The AST visitor for the V2 system. It serves as a generic interface that dispatches to language-specific extractors (currently `RustExtractor`). This pattern allows SlopChop to easily extend its deep analysis capabilities to other languages in the future by adding new extractors that fulfill the same high-level interface for structural data gathering.

### [worker.rs](file:///home/juno/slopchop/src/analysis/v2/worker.rs)
The per-file worker for V2 analysis. it handles the I/O of reading a file and running both regex-based and AST-based pattern detection. It then uses the `AstVisitor` to extract the high-level structural scopes needed for deep analysis. This worker is the basic unit of execution that the V2 engine distributes across threads during a scan.

---

## src/analysis/v2/patterns/

### [mod.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/mod.rs)
The orchestrator for the V2 pattern detection subsystem. It defines `detect_all`, which runs a battery of specialized checkers (concurrency, performance, logic, security, etc.) against the tree-sitter root of a file. It also provides helper utilities like `get_capture_node` to simplify data extraction from tree-sitter query matches across the various sub-modules.

### [concurrency.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/concurrency.rs)
A gateway module for concurrency-related checks. It re-exports and aggregates detections for deadlocks (C03) and undocumented synchronization (C04). This provides a single entry point for the `patterns::mod` to invoke all concurrency-focused rules, ensuring that shared state and threading issues are consistently identified during analysis.

### [concurrency_lock.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/concurrency_lock.rs)
Implements detection for the C03 violation: holding a `MutexGuard` across an `.await` point. This is a critical check for async Rust code, as holding a standard `std::sync::Mutex` across an await can lead to deadlocks or thread starvation. The module identifies `async fn` bodies and tracks lock acquisition relative to await points to flag potential safety issues.

### [concurrency_sync.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/concurrency_sync.rs)
Focuses on the C04 violation: undocumented synchronization primitives. It flags struct fields using `Arc<Mutex<T>>` or `Arc<RwLock<T>>` that lack a triple-slash `///` or double-slash `//` documentation comment. This enforces a policy that shared mutable state must be explained, helping maintainers understand what data is being protected and why.

### [db_patterns.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/db_patterns.rs)
Scans for the P03 anti-pattern: N+1 database queries. It identifies database-related method calls (e.g., `fetch_one`, `query`, `load`) occurring within loops that use a loop variable as part of the call. This identifies a major performance bottleneck where a single operation executes multiple separate queries instead of using a batched approach or a JOIN.

### [idiomatic.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/idiomatic.rs)
Enforces idiomatic programming practices (I01, I02). it flags manual `From` implementations that could be simplified with `#[derive(From)]` and identifies duplicate match arm bodies that should be combined (e.g., `A | B => body`). This ensures that the codebase remains clean, concise, and follows established community best practices for the target language.

### [logic.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/logic.rs)
Detects dangerous logic patterns, including L02 (boundary ambiguity) and L03 (unchecked indexing). it flags suspicious uses of `<=`/`>=` with `.len()` which often indicate off-by-one errors, and identifies unchecked `[0]` or `.first().unwrap()` calls that risk runtime panics on empty collections. It includes logic for identifying "guards" that might make such code safe.

### [performance.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/performance.rs)
A comprehensive performance checker (P01, P02, P04, P06). It flags hoistable clones and allocations inside loops, nested loops (quadratic complexity), and linear searches (`.find()`) inside loops (O(n*m)). By identifying these common bottlenecks early, it prevents significant performance degradation as the codebase scales, enforcing efficient resource usage during the gated check process.

### [resource.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/resource.rs)
Implements resource management checks, specifically R07: missing `flush()` calls for `BufWriter`. It identifies instances where a buffered writer is created but not explicitly flushed before the function ends, which can lead to data loss if the writer is dropped while data remains in the internal buffer. This ensures that I/O operations are completed reliably and safely.

### [security.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/security.rs)
The security sentinel (X01, X02, X03). it scans for SQL injection risks (using `format!` instead of parameters), command injection (unvalidated variables in `Command::new`), and potential hardcoded secrets (variable names like `KEY` or `TOKEN` assigned to string literals). This module provides a basic but essential security baseline that prevents common vulnerabilities from being introduced by AI agents.

### [semantic.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/semantic.rs)
Enforces semantic naming and behavior consistency (M03, M04, M05). it flags "getters" (`get_*`, `is_*`) or "calculators" (`compute_*`) that take `&mut self`, as these names imply read-only or pure operations. It also ensures that boolean-prefixed functions (`is_*`, `has_*`) actually return booleans. This ensures that the code's behavior matches its naming, reducing cognitive load for developers.

### [state.rs](file:///home/juno/slopchop/src/analysis/v2/patterns/state.rs)
Monitors global state patterns (S01, S02, S03). it flags dangerous `static mut` declarations, exported public statics that may lead to implicit coupling, and suspicious global containers (like `lazy_static! { Mutex<HashMap> }`). By discouraging global state, it pushes developers toward more functional, testable, and dependency-injected architectures that are less prone to subtle state-related bugs and data races.

---

## src/apply/

### [mod.rs](file:///home/juno/slopchop/src/apply/mod.rs)
The entry point for the "Apply" subsystem, responsible for processing AI-generated code changes. it orchestrates the reading of input from various sources (clipboard, stdin, or file) and dispatches it to the `processor` for parsing and execution. This module connects the CLI commands to the underlying transactional logic that safely stages and verifies changes on the work branch.

### [advisory.rs](file:///home/juno/slopchop/src/apply/advisory.rs)
A developer-experience utility that monitors edit volume. It triggers an "Advisory" message if it detects more than three modified files on the current work branch, encouraging the user to commit their progress. This serves as a nudge toward the high-integrity practice of making small, incremental, and easily verifiable checkpoints rather than large, monolithic batches of changes.

### [backup.rs](file:///home/juno/slopchop/src/apply/backup.rs)
Provides safety through automatic file backups and rollbacks. Before any files are overwritten or deleted during an `apply` operation, this module creates a timestamped backup in `.slopchop_apply_backup`. If an error occurs during the transaction, it can perform a full rollback to restore the repository to its original state. It also handles automatic cleanup of old backups based on configured retention limits.

### [blocks.rs](file:///home/juno/slopchop/src/apply/blocks.rs)
The block construction and sanitization layer for the XSC7XSC protocol. it handles the creation of typed blocks (`PLAN`, `MANIFEST`, `FILE`, `PATCH`) and implements robust logic for stripping transport-level prefixes (like markdown blockquote symbols `>`) from the content. It also ensures that file paths provided in blocks are not reserved keywords, preventing protocol-level injection attacks.

### [executor.rs](file:///home/juno/slopchop/src/apply/executor.rs)
The heart of the transactional apply logic. It manages the `slopchop-work` branch, ensuring it is initialized and checked out before writing files. It orchestrates the write-verify-commit sequence, including saving the "pending goal" for later promotion. This module bridges the gap between the `writer` and the `verification` pipeline, ensuring that every applied change is safely staged and logged as an event.

### [manifest.rs](file:///home/juno/slopchop/src/apply/manifest.rs)
Parses the delivery manifest which lists the files to be modified and their operations (NEW, DELETE, or UPDATE). it uses regex to clean up common markdown list markers and extracts the final paths and operations. This manifest acts as the "contract" for the apply operation, defining exactly which files from the AI's payload should be acted upon.

### [messages.rs](file:///home/juno/slopchop/src/apply/messages.rs)
The communication layer for the apply process. It handles printing outcomes (success, promotion, failure) to the console with color-coded status indicators. Notably, it generates and formats the AI feedback message when verification fails, including truncated logs and error summaries. It also implements utilities for handling UTF-8 character boundaries during log truncation to prevent malformed output.

### [parser.rs](file:///home/juno/slopchop/src/apply/parser.rs)
The strict parser for the XSC7XSC protocol. It uses a stateful approach to identify and extract typed blocks from a raw input stream. Key features include support for multi-line blocks with varied markdown prefixes and a design that prevents "signature spoofing" where block headers could be hidden inside content. This parser ensures the integrity of the communication channel between the AI and the tool.

### [patch.rs](file:///home/juno/slopchop/src/apply/patch.rs)
A legacy file containing surgical patch application logic. While it supports V1 (context-anchored) and V0 (search/replace) formats, it is currently DEPRECATED in favor of full-file atomicity. The file remains in the tree as a historical artifact but its features are explicitly bypassed by the modern `processor` in favor of whole-file `FILE` blocks which offer better reliability and simpler AI integration.

### [process_runner.rs](file:///home/juno/slopchop/src/apply/process_runner.rs)
Provides a robust interface for running external sub-processes (like `cargo test` or `clippy`) with streaming output. It manages separate threads for capturing stdout/stderr and includes a "heartbeat" mechanism to keep the CLI spinner alive during long-running commands. It also handles failure reporting, summarizing large outputs and automatically copying error summaries to the system clipboard for the user.

### [processor.rs](file:///home/juno/slopchop/src/apply/processor.rs)
The high-level orchestrator for the `apply` command. it ties together the `parser`, `validator`, and `executor`. It handles plan review (requiring user confirmation of the AI's intent), performs "markdown fence" sanitization to strip accidental AI formatting, and extracts the commit message from the plan. This module ensures that the entire apply workflow follows the SlopChop transactional protocol.

### [report_writer.rs](file:///home/juno/slopchop/src/apply/report_writer.rs)
Generates the comprehensive `slopchop-report.txt` file after a verification run. It creates a dashboard summarizing cognitive complexity and file sizes, lists all structural violations, and appends full logs from external tools. This persistent report provides a detailed record of the verification gate, helping developers debug complex failures that might not be fully visible in the summarized console output.

### [types.rs](file:///home/juno/slopchop/src/apply/types.rs)
Defines the core data models for the apply subsystem. This includes the `Block` and `Operation` enums, `ManifestEntry`, and the `ApplyContext` which carries configuration and CLI flags throughout the transaction. Centralizing these types ensures that the various apply modules have a consistent language for describing the state and outcomes of code-change operations.

### [validator.rs](file:///home/juno/slopchop/src/apply/validator.rs)
The security and integrity gatekeeper for the apply process. It performs multiple checks: verifying paths against blocked directories (like `.git`) and protected files, ensuring manifest consistency, and checking for common AI mistakes like code truncation (e.g., `// ...`) or accidental markdown fences. It even performs a syntax check using tree-sitter to ensure applied code is paraseable before it hits the disk.

### [writer.rs](file:///home/juno/slopchop/src/apply/writer.rs)
The file-system interaction layer. It implements atomic file writing by first writing to a temporary file and then performing a rename, which minimizes the window for data corruption. It also includes "Symlink Escape" protection, verifying that every file operation stays within the repository root. This module ensures that the final step of the apply transaction is both safe and durable.

---

## src/apply/verification/

### [mod.rs](file:///home/juno/slopchop/src/apply/verification/mod.rs)
Orchestrates the verification pipeline that runs after an apply operation. it manages the sequence of external checks (lints, tests) and internal SlopChop scans (structural analysis, locality). It integrates with the `SpinnerClient` for progress reporting and coordinates with the `report_writer` to persist results. This module is the final gate that decides if a change is "Done" according to project policy.

### [report_display.rs](file:///home/juno/slopchop/src/apply/verification/report_display.rs)
The console formatting logic for verification reports. It prints a high-level summary of check statuses, followed by detailed structural statistics (total tokens, complexity) and a breakdown of violations by type. It also identifies "top offenders"—files with the most violations—helping developers prioritize their refactoring efforts. This module ensures that the verification results are communicated clearly and impactfully.

---

## src/cli/

### [mod.rs](file:///home/juno/slopchop/src/cli/mod.rs)
The module entry point for the command-line interface handlers. It declares and organizes the various subcommands of SlopChop, including the apply engine, mutation testing, and the interactive configuration TUI. By aggregating these handlers, it provides a clean API for the main binary to dispatch commands while keeping the business logic separated from argument parsing. It essentially serves as the table of contents for how the CLI interacts with the core library functions.

### [apply_handler.rs](file:///home/juno/slopchop/src/cli/apply_handler.rs)
The command handler for the `apply` operation, bridging CLI-specific arguments to the transactional application engine. It handles determining the source of changes—whether via stdin, a specific file, or the system clipboard—and configures the application context with flags for dry-runs, safety sanitization, and automatic branch promotion. It maps the internal results of the apply process back to standardized CLI exit codes, ensuring consistent feedback for automated workflows.

### [args.rs](file:///home/juno/slopchop/src/cli/args.rs)
Defines the structure of the Command Line Interface using the `clap` crate. This file is the source of truth for all available subcommands, flags, and arguments, such as `pack`, `scan`, `check`, and `apply`. It implements the parser and provides the strongly-typed structures that the rest of the CLI subsystem uses to understand user intent. It also contains the `ApplyArgs` structure used for configuring change application.

### [audit.rs](file:///home/juno/slopchop/src/cli/audit.rs)
An orphaned or legacy handler for codebase consolidation audits. It contains logic for orchestrating checks that find dead code and duplicate patterns, intended to help developers clean up technical debt. While it provides detailed progress reporting and can copy AI-friendly summaries to the clipboard, it is currently disconnected from the main `dispatch.rs` logic. It sits as a standalone utility for identifying deep structural optimization opportunities.

### [dispatch.rs](file:///home/juno/slopchop/src/cli/dispatch.rs)
The central routing hub for the SlopChop CLI. It takes the abstract commands parsed in `args.rs` and dispatches them to their corresponding specialized handlers across the `analysis`, `apply`, and `git_ops` modules. By isolating the match logic here, the main entry point is kept minimal and focused. It manages the high-level execution flow, from running structural scans to performing GIT branch management and interactive configuration editing.

### [git_ops.rs](file:///home/juno/slopchop/src/cli/git_ops.rs)
Implements the CLI handlers for Git-based workflow actions, specifically managing the `slopchop-work` branch. It provides user-facing interfaces for initializing new work branches, promoting completed changes back to the main branch with automated commit messaging, and aborting current work. These handlers wrap the low-level process executions found in the library, providing consistent console output and status updates for the developer during the AI-driven development cycle.

### [locality.rs](file:///home/juno/slopchop/src/cli/locality.rs)
The CLI interface for the "Law of Locality" enforcement system. It orchestrates the end-to-end locality check process: loading configuration, discovering files, extracting imports, building the dependency graph, and finally validating it against architectural rules. It outputs detailed health reports to the console, identifying circular dependencies and sideways imports. This file serves as the primary entry point for developers to verify the topological integrity and "cache-locality" of their codebase.

### [mutate_handler.rs](file:///home/juno/slopchop/src/cli/mutate_handler.rs)
The handler for the experimental mutation testing command. It translates CLI flags like worker counts and test timeouts into mutation engine configurations. It initiates the mutation run across the codebase, identifying "survived" mutants that indicate gaps in test coverage. The handler ensures that failures in the mutation pipeline translate to the correct CLI exit codes, allowing it to be integrated into CI/CD pipelines to guard against regressions in test quality.

---

## src/cli/config_ui/

### [mod.rs](file:///home/juno/slopchop/src/cli/config_ui/mod.rs)
The module entry point for the interactive configuration TUI. It exports the primary `run_config_editor` function, which allows users to modify their `slopchop.toml` file through a terminal interface. It serves as a glue layer for the editor's internal state management, rendering logic, and event handling, providing a high-level API for the rest of the CLI to invoke the configuration editing experience.

### [editor.rs](file:///home/juno/slopchop/src/cli/config_ui/editor.rs)
Manages the state and lifecycle of the interactive configuration editor. It defines the `ConfigEditor` structure, which tracks the current configuration, the selected menu item, and whether changes have been modified. This file contains the high-level logic for initializing the TUI and persisting changes back to disk after a user saves their edits. It acts as the "Controller" in the TUI's internal architecture, coordinating between the data and the rendering logic.

### [items.rs](file:///home/juno/slopchop/src/cli/config_ui/items.rs)
The definition of all editable configuration fields within the SlopChop ecosystem. It lists all metrics (like max tokens, complexity, and nesting) and preferences (like auto-copy or locality mode) that can be adjusted via the TUI. Each item is associated with logic to retrieve its value, cycle through enum states, or toggle boolean flags. It provides the metadata required for the TUI to render readable labels and interactive checkboxes or numeric inputs.

### [logic.rs](file:///home/juno/slopchop/src/cli/config_ui/logic.rs)
Implements the event loop and terminal interaction logic for the configuration editor using the `crossterm` crate. It handles raw terminal mode, keyboard events (navigation, saving, quitting), and the specialized logic for "adaptive stepping" when editing numeric values. This file bridge the gap between user keypresses and state changes in the `ConfigEditor`, managing the alternate screen and providing the interactive "feel" of the TUI application.

### [render.rs](file:///home/juno/slopchop/src/cli/config_ui/render.rs)
The "View" layer of the configuration TUI, responsible for drawing the interface on the terminal. It handles formatting the header, listing configuration items with their current values and selection pointers, and displaying a helpful footer with keyboard shortcuts. Using ANSI colors and box-drawing characters, it creates a premium-looking terminal interface. It ensures that the display is efficiently updated in response to state changes during the interactive session.

---

## src/cli/handlers/

### [mod.rs](file:///home/juno/slopchop/src/cli/handlers/mod.rs)
A collection of orchestration logic for primary CLI operations like `scan`, `check`, `pack`, and `signatures`. It coordinates between the rule engine, file discovery, and reporting systems. Notably, it implements the progress reporting for long-running scans using a spinner-based UI. This module acts as the implementation layer for `dispatch.rs`, ensuring that complex command sequences—like running the full verification pipeline or generating repo maps—are handled in a centralized, testable location.

### [scan_report.rs](file:///home/juno/slopchop/src/cli/handlers/scan_report.rs)
Specialized in formatting and displaying the final summary of a codebase scan. It provides a visual breakdown of violations, file sizes, and top complexity scores, using color-coded output to highlight areas of concern. It includes logic to detect "small codebases" where certain structural metrics are skipped for better signal-to-noise. This file ensures that the output of SlopChop is not just a list of errors, but an actionable architectural health dashboard.

---

## src/graph/

### [mod.rs](file:///home/juno/slopchop/src/graph/mod.rs)
The entry point for the dependency and symbol graph subsystem. It declares sub-modules for import extraction, symbol definition, ranking, and the topological "Law of Locality." This module organizes the tools required to transform raw source code into a structured graph of relationships, which is fundamental to SlopChop's ability to understand project architecture, identify critical files, and enforce boundary rules.

### [imports.rs](file:///home/juno/slopchop/src/graph/imports.rs)
Responsible for extracting raw import statements from source files. It uses `tree-sitter` and language-specific queries (defined in `lang.rs`) to identify module imports, requires, and uses across Rust, Python, and TypeScript. By parsing the AST rather than using regex, it reliably finds imported modules while ignoring comments and strings. This data forms the "edges" of the dependency graph used for ranking and locality analysis.

### [resolver.rs](file:///home/juno/slopchop/src/graph/resolver.rs)
Bridges the gap between abstract import strings (like `use crate::config` or `import "./utils"`) and actual file paths on disk. It implements language-specific resolution logic, such as Rust's `mod.rs` vs file patterns and TypeScript's complex path mapping. This file is critical for building an accurate dependency graph, as it understands the filesystem conventions of different build systems to find the target code being referenced.

### [tsconfig.rs](file:///home/juno/slopchop/src/graph/tsconfig.rs)
A dedicated parser for `tsconfig.json` and `jsconfig.json` files, specifically focused on `baseUrl` and `paths` mappings. It handles stripping comments from JSON, resolving path aliases, and checking for index files. This allows the graph subsystem to accurately resolve "aliased" imports (like `@/components/Button`) in modern JavaScript and TypeScript projects, ensuring that locality checks work correctly in complex web applications.

---

## src/graph/defs/

### [mod.rs](file:///home/juno/slopchop/src/graph/defs/mod.rs)
The module entry for the symbol definition extraction system. It coordinates the use of `tree-sitter` to identify where functions, structs, classes, and other symbols are defined within the codebase. By exposing these definitions, it allows the ranking and search systems to understand the "surface area" of each file, enabling cross-referencing between where a symbol is defined and where it is imported.

### [extract.rs](file:///home/juno/slopchop/src/graph/defs/extract.rs)
Implements the core logic for querying the AST to find symbol definitions. It maps language-specific tree-sitter nodes (like `struct_item` or `class_definition`) into a unified `Definition` structure containing the symbol name, kind, line number, and signature. This extraction is a key part of the graph construction pipeline, providing the "targets" for the edges identified by the import extractor. It supports multiple languages and ensures symbol kind consistency.

### [queries.rs](file:///home/juno/slopchop/src/graph/defs/queries.rs)
A utility for managing and compiling the `tree-sitter` queries used for definition extraction. It centralizes the heavy lifting of preparing the AST patterns for different languages, ensuring they are compiled correctly before being used by the extractor. This separation keeps the extraction logic focused on processing results while this file handles the low-level tree-sitter configuration.

---

## src/graph/rank/

### [mod.rs](file:///home/juno/slopchop/src/graph/rank/mod.rs)
Orchestrates the codebase ranking system, which identifies the most important files in a repository. It provides the `GraphEngine` to build a `RepoGraph` and compute file importance using PageRank. It also supports "focus ranking," where importance is calculated relative to a specific "anchor" file. This system is used by SlopChop to prioritize analysis and help users navigate large codebases by highlighting central "hubs."

### [builder.rs](file:///home/juno/slopchop/src/graph/rank/builder.rs)
Constructs the actual dependency graph by matching symbol definitions against references across all files. It walks the codebase, extracts "tags" (defines and refs), and builds an adjacency matrix (edges) where weights represent the frequency of interaction. It handles the mapping of abstract symbol names to files, creating the topological foundation required for the ranking algorithms and locality validation to operate.

### [graph.rs](file:///home/juno/slopchop/src/graph/rank/graph.rs)
Defines the `RepoGraph` structure, the core data container for the project's dependency network. It stores symbol definitions, references, and the computed importance scores for every file. The struct provides a high-level API for querying "neighbors" (bi-directional), "dependencies" (fan-out), and "dependents" (fan-in). It also identifies "hubs"—files that act as central infrastructure based on their high number of incoming dependencies.

### [pagerank.rs](file:///home/juno/slopchop/src/graph/rank/pagerank.rs)
An implementation of the PageRank algorithm tailored for file dependencies. It treats the codebase as a network where imports are directed links. Files that are frequently imported by other important files receive higher scores. It includes support for "personalization" (anchoring), which boosts the importance of files topologically close to a specific starting point. This algorithm powers SlopChop's ability to automatically identify critical architectural components.

### [queries.rs](file:///home/juno/slopchop/src/graph/rank/queries.rs)
Implements the query logic for the `RepoGraph`, providing easy access to topological information. It contains functions to collect list of dependencies and dependents for any given file by traversing the internal definition and reference maps. This file translates the raw graph data into actionable lists of related files, which is used for generating repo maps and identifying structural smells like isolated code or God modules.

### [tags.rs](file:///home/juno/slopchop/src/graph/rank/tags.rs)
Contains the basic data structures for symbol "tags." A tag represents either a Definition (where code is located) or a Reference (where code is used). These simple structs carry the essential metadata—file path, symbol name, line number, and signature—required by the graph builder to link different parts of the codebase together into a unified dependency model.

---

## src/graph/locality/

### [mod.rs](file:///home/juno/slopchop/src/graph/locality/mod.rs)
The central module for the "Law of Locality" enforcement system. It exports the tools needed to measure "topological distance" and enforce architectural boundaries. This system is designed to prevent "sideways dependencies" (unrelated modules talking directly) and "upward dependencies" (violating layers). It coordinates between classifiers, distance calculators, and layer inferrers to judge whether any given code relationship is architecturally sound.

### [classifier.rs](file:///home/juno/slopchop/src/graph/locality/classifier.rs)
Categorizes files into specific architectural roles based on their coupling metrics. Files with high fan-in and low fan-out are classified as "Stable Hubs," while those with high fan-out and low fan-in are "Volatile Leafs." It also identifies "God Modules" (monoliths) and "Deadwood" (isolated files). These identities are critical for the locality validator, as certain roles (like Hubs) are permitted to have far-reaching dependencies that would be forbidden for others.

### [coupling.rs](file:///home/juno/slopchop/src/graph/locality/coupling.rs)
Calculates Afferent coupling (fan-in) and Efferent coupling (fan-out) for all nodes in the dependency graph. These metrics measure how many files depend on a module versus how many it depends on itself. This data is the primary input for the node classifier and for determining the "Instability Index" of code, helping SlopChop identify fragile or overly central areas of the architecture.

### [cycles.rs](file:///home/juno/slopchop/src/graph/locality/cycles.rs)
Implements architectural cycle detection using a Depth-First Search (DFS) algorithm. It identifies sets of files that depend on each other in a loop, which is a major source of technical debt and build complexity. In SlopChop's locality system, a cycle is considered a "hard error" that must be resolved to restore topological integrity, as cycles break the hierarchical layering of a well-structured system.

### [distance.rs](file:///home/juno/slopchop/src/graph/locality/distance.rs)
Computes the "architectural distance" between two files using their positions in the directory tree. It finds the Lowest Common Ancestor (LCA) and calculates the steps required to navigate from one file to the other. This "directory distance" serves as a proxy for locality—files within the same folder are "close" (Distance 2), while files in distant sibling directories are "far" (Distance 4+). This metric is the core of the locality validation algorithm.

### [edges.rs](file:///home/juno/slopchop/src/graph/locality/edges.rs)
The pre-processor for locality analysis, responsible for collecting dependency "edges" from the filesystem. it orchestrates the reading of files, extraction of imports, and resolution to file paths, producing a list of normalized (from, to) pairs. This abstracts the raw IO and parsing away from the validation logic, ensuring that the locality system works on a clean model of project-relative file relationships.

### [exemptions.rs](file:///home/juno/slopchop/src/graph/locality/exemptions.rs)
Implements "smart exemptions" for the locality system, specifically understanding Rust module patterns. It auto-exempts structural relationships that are technically "far" but architecturally necessary, such as `main.rs` importing everything, or `mod.rs` re-exporting children. By recognizing these vertical routing patterns, it reduces false positives and ensures the locality analysis focused on genuine sideways architectural leaks.

### [layers.rs](file:///home/juno/slopchop/src/graph/locality/layers.rs)
Infers an architectural layering system from the dependency graph. It automatically groups files into hierarchical levels (L0, L1, etc.), where lower levels are foundation code and higher levels are specialized logic. It identifies "Upward Dependencies"—where a lower-level library depends on a higher-level feature—which is a major architectural violation. This automated layer inference allows SlopChop to enforce unidirectional dependency flow without manual configuration.

### [report.rs](file:///home/juno/slopchop/src/graph/locality/report.rs)
Generates rich, human-readable reports on codebase topology. It visualizes the layer architecture, identifies "God Modules" with high violation counts, and flags "Hub Candidates" that are frequently imported but not yet officially recognized. The report includes a "Topological Health" score and "Entropy" metric, providing developers with a high-level dashboard of how well-structured their project is according to the Law of Locality.

### [types.rs](file:///home/juno/slopchop/src/graph/locality/types.rs)
Defines the core data structures for the locality system. This includes `Coupling` metrics (fan-in/fan-out), `NodeIdentity` (Stable Hub, Deadwood, etc.), and the `LocalityEdge` which carries distance and skew metrics. It also defines the `EdgeVerdict` and `PassReason` enums used to explain why a dependency was either approved or flagged as a violation.

### [validator.rs](file:///home/juno/slopchop/src/graph/locality/validator.rs)
The implementation of the "Universal Locality Algorithm." It judges every dependency edge against a set of rules: Is it "close" enough (Distance < 4)? Is it vertical routing to a shared "Hub"? Does it violate the inferred "Layering"? It combines distance metrics, node roles, and structural exemptions to produce a `ValidationReport`. This file is the "judge" that determines whether the codebase's topology has been compromised.

---

## src/graph/locality/analysis/

### [mod.rs](file:///home/juno/slopchop/src/graph/locality/analysis/mod.rs)
The entry point for deep topological analysis. It takes raw validation results and produces higher-level insights by categorizing violations and identifying systemic architectural issues. It coordinates the identification of God Modules, Hub Candidates, and excessive Module Coupling, turning a simple list of "bad imports" into actionable architectural advice for the developer.

### [metrics.rs](file:///home/juno/slopchop/src/graph/locality/analysis/metrics.rs)
Calculates aggregate topological metrics from a validation report. It identifies "God Modules" as files that are responsible for a high number of outbound violations, and "Hub Candidates" as files with very high fan-in that should likely be promoted to official infrastructure. It also computes "Module Coupling" strength between different top-level directories, highlighting tight-coupling between theoretically separate modules.

### [violations.rs](file:///home/juno/slopchop/src/graph/locality/analysis/violations.rs)
Categorizes failing dependencies into recognizable architectural smells like "Encapsulation Breach" (importing internals), "Missing Hub" (frequently used file), or "Upward Dependency" (layer violation). For each category, it provides a specific, actionable fix suggestion (e.g., "Expose API from mod.rs" or "Move to lower layer"). This makes SlopChop's feedback much more meaningful than a generic "import error."

---

## src/mutate/

### [mod.rs](file:///home/juno/slopchop/src/mutate/mod.rs)
The coordinator for the experimental cross-language mutation testing system. It manages the end-to-end lifecycle of a mutation run: discovering target source files, identifying mutable AST nodes, and orchestrating worker threads to run tests against mutated code. It supports multiple languages (Rust, Python, TypeScript) and provides both human-readable terminal reports and machine-parsable JSON output, making it suitable for both local development and CI/CD integration.

### [discovery.rs](file:///home/juno/slopchop/src/mutate/discovery.rs)
Implements the mutation discovery engine using `tree-sitter` for precise AST analysis. It walks the syntax tree of source files to identify "mutable points"—operators and literals that can be safely modified to test the strength of the test suite. By targeting specific node kinds like `binary_operator` or `boolean_literal`, it ensures that mutations are syntactically valid and architecturally meaningful, avoiding "junk mutations" that would lead to unparsable code.

### [mutations.rs](file:///home/juno/slopchop/src/mutate/mutations.rs)
Defines the core library of mutation patterns and the logic for applying them. It categorizes mutations into logical groups such as `Comparison` (`==` to `!=`), `Logical` (`&&` to `||`), `Boolean` (`true` to `false`), and `Arithmetic` (`+` to `-`). This file contains the "rules of transformation" that dictate how a specific piece of source code should be modified to create a mutant, providing the atomic operations used by the runner.

### [report.rs](file:///home/juno/slopchop/src/mutate/report.rs)
The presentation layer for mutation testing results. it handles real-time progress updates, showing which mutants were "Killed" (caught by tests) or "Survived." It generates a final dashboard featuring a "Mutation Score"—a critical metric for test suite effectiveness—and provides a detailed breakdown of "test gaps" where mutations passed unnoticed. This module ensures that the complex data from a mutation run is turned into actionable insights for the developer.

### [runner.rs](file:///home/juno/slopchop/src/mutate/runner.rs)
Orchestrates the execution of the project's test suite against mutated source code. It handles the delicate task of applying a mutation to a file, running the detected test command (e.g., `cargo test` or `pytest`), and then restoring the original file to its state. It manages process timeouts to handle infinite loops caused by mutations and collects aggregate statistics on run durations and outcomes, although it is currently limited to serial execution in v1.

---

## src/pack/

### [mod.rs](file:///home/juno/slopchop/src/pack/mod.rs)
The high-level orchestrator for the repository "packing" system. it coordinates the discovery of files, the application of "foveal" and "peripheral" focus rules, and the generation of structured context for AI consumption. It ties together the rule engine (to inject current violations), the tokenizer (for token counting), and output handlers (file, clipboard, or stdout). This module is the engine behind SlopChop's ability to "crunch" a large codebase into a concise, AI-ready package.

### [docs.rs](file:///home/juno/slopchop/src/pack/docs.rs)
A specialized parser for extracting documentation comments from various programming languages. It understands the nuances between Python's docstrings and C-style or Rust's triple-slash comments. This extraction logic is used specifically for the "Spec" output format, allowing SlopChop to generate high-level technical specifications that focus on public APIs and developer intent while ignoring the implementation details of the code.

### [focus.rs](file:///home/juno/slopchop/src/pack/focus.rs)
Implements the "Holographic Focus" algorithm used to manage token budgets. Based on the project's dependency graph, it identifies which files are "foveal" (central to the current task and provided in full) and which are "peripheral" (topologically related but only provided as skeletons). This automated expansion of the context window ensures the AI has all the necessary structural information while keeping the total token count as low as possible.

### [formats.rs](file:///home/juno/slopchop/src/pack/formats.rs)
Defines the standard text-based output format for packed repository context. It uses unique "sigil" markers to delimit files, providing the AI with a clear structure to parse the codebase. This module also handles "skeletonization"—stripping function bodies to reduce token counts—and generates the "Holographic Spec" format. It represents the primary way SlopChop serializes a project's state for external consumption by LLMs.

### [xml_format.rs](file:///home/juno/slopchop/src/pack/xml_format.rs)
An alternative output generator that produces structured XML context. It maps the project's file hierarchy and focus-mode data into standard XML tags, utilizing CDATA sections to safely wrap original source code. This format is designed for maximum compatibility with modern AI models that excel at parsing structured documents, often providing better results than plain text for complex multi-file reasoning tasks.

---

## src/signatures/

### [mod.rs](file:///home/juno/slopchop/src/signatures/mod.rs)
The generator for SlopChop's holographic signature map. It performs a wide-scale scan of the codebase's "type surface," extracting public exports and their associated documentation. Using the repository graph, it orders these signatures topologically so that foundational types are defined before their dependents. It also assigns "tiers" (CORE, HIGH, etc.) based on PageRank scores, giving the AI an immediate sense of which files are architecturally central to the project.

### [docs.rs](file:///home/juno/slopchop/src/signatures/docs.rs)
Focuses on preserving developer intent during signature extraction. it identifies doc-comment prefixes (like `///` or `/**`) and attributes (like `#[must_use]`) to ensure they are captured alongside the symbols they document. This module expands the raw symbol ranges to include the preceding context, transforming a dry list of types into a rich, human-readable API map that retains critical usage instructions and architectural notes.

### [ordering.rs](file:///home/juno/slopchop/src/signatures/ordering.rs)
Implements the architectural sorting logic for the signature map. It primarily uses Kahn's algorithm for topological sorting, ensuring that base libraries and utility modules appear before the high-level logic that depends on them. For complex cyclic structures or disconnected files, it falls back to ordering by PageRank importance. This bottom-up presentation strategy is designed to help AI models build a robust mental model of the codebase's type system layer-by-layer.

---

## src/spinner/

### [mod.rs](file:///home/juno/slopchop/src/spinner/mod.rs)
The entry point for the Head-Up Display (HUD) progress reporting system. It provides a simple interface to spawn a high-performance background spinner that doesn't block the main application thread. By splitting the interface into a `client` (for updates) and a `controller` (for lifecycle management), it allows concurrent operations—like codebase scans—to provide rich visual feedback while ensuring a clean terminal exit upon completion.

### [client.rs](file:///home/juno/slopchop/src/spinner/client.rs)
The primary API for sending updates to the active HUD. It provides high-level methods to set the "macro" pipeline step (e.g., [1/4]), update the "micro" status message, and push log lines to a rolling buffer. This thread-safe client is designed to be passed into worker threads and external processors, allowing them to report progress without needing to worry about the underlying TUI rendering logic.

### [controller.rs](file:///home/juno/slopchop/src/spinner/controller.rs)
Manages the lifecycle and terminal state of the background spinner. Held by the thread that initiated the process, it provides the logic to gracefully stop the spinner and join the background thread. It captures the final execution status (success or failure) and ensures that the final summary message is printed to the terminal after the HUD interface has been cleared.

### [handle.rs](file:///home/juno/slopchop/src/spinner/handle.rs)
The low-level thread management layer for the spinner subsystem. It is responsible for spawning the dedicated rendering thread and providing an atomic "running" flag to control its execution loop. This separation prevents the visual update frequency (e.g., the spinner spinning at 12fps) from being tied to the work-loop performance, ensuring a smooth UI even during heavy CPU-bound analysis tasks.

### [render.rs](file:///home/juno/slopchop/src/spinner/render.rs)
The visual engine of the HUD system. It uses `crossterm` and ANSI escape sequences to draw a multi-part terminal interface in real-time. The interface includes a blue-themed pipeline header, a primary progress bar, a yellow status line for granular updates, and a dimmed rolling log buffer. It handles terminal width detection, string truncation, and the final transition from the interactive HUD back to standard console output.

### [safe_hud.rs](file:///home/juno/slopchop/src/spinner/safe_hud.rs)
A thread-safe wrapper around the shared HUD state. It encapsulates a `Mutex<HudState>` and provides a clean `modify` pattern for atomic state updates. By hiding the mutex implementation from consumers, it reduces architectural coupling and prevents lock-contention patterns, ensuring that multiple concurrent processes can report their progress safely and efficiently.

### [state.rs](file:///home/juno/slopchop/src/spinner/state.rs)
Defines the data structures that track the current progress and status of a SlopChop operation. It stores the pipeline title, step counts, active log lines, and start times. Notably, it includes "status extraction" logic that can automatically derive meaningful status updates from raw log lines (e.g., seeing "Compiling..." in a log and updating the HUD status automatically), providing a more intelligent progress-tracking experience.

---

## src/clipboard/

### [mod.rs](file:///home/juno/slopchop/src/clipboard/mod.rs)
The high-level manager for SlopChop's specialized clipboard operations. It implements "smart copying" logic that automatically decides whether to copy content as raw text (for small payloads) or as a temporary file handle (for large payloads > 2000 tokens). This ensures that massive AI-generated contexts or large source-code blocks are never truncated and can be pasted seamlessly as file attachments in modern developer tools.

### [linux.rs](file:///home/juno/slopchop/src/clipboard/linux.rs)
Provides robust Linux clipboard integration with specialized support for WSL (Windows Subsystem for Linux). It leverages `wl-copy` (Wayland) and `xclip` (X11) for native Linux environments and coordinates with PowerShell and `clip.exe` when running inside WSL. This ensures that Linux users have first-class support for both text and file-handle copying, regardless of their specific desktop environment or virtualization setup.

### [macos.rs](file:///home/juno/slopchop/src/clipboard/macos.rs)
Implements native macOS clipboard support using the `pbcopy` and `pbpaste` utilities. It also utilizes `osascript` to perform specialized "POSIX file" object copying, which allows the AI's output to be pasted directly into chat interfaces as a real file attachment rather than just a massive wall of text. This module is essential for the smooth "Holographic" workflow on Apple hardware.

### [platform.rs](file:///home/juno/slopchop/src/clipboard/platform.rs)
A cross-platform facade for the clipboard subsystem. Using conditional compilation (`cfg`), it exports a unified `platform_impl` module that points to the correct OS-specific implementation (Linux, macOS, or Windows). This allows the rest of the SlopChop codebase to perform complex clipboard operations—like copying file handles—without needing to know the low-level details of each operating system's API.

### [temp.rs](file:///home/juno/slopchop/src/clipboard/temp.rs)
Manages the lifecycle of temporary files generated during large clipboard operations. It handles the secure creation of uniquely-named context files in the system's temporary directory and provides a "garbage collection" routine that automatically purges any SlopChop temporary artifacts older than 15 minutes. This prevents the tool from cluttering the developer's temporary storage over time.

### [utils.rs](file:///home/juno/slopchop/src/clipboard/utils.rs)
Contains internal utilities for piping data into the stdin of external clipboard managers. It abstracts away the complexity of spawning subprocesses and managing process streams, providing a reliable `pipe_to_cmd` function used by all platform implementations. This focus on stdin piping allows SlopChop to handle very large text payloads without exhausting environment variable limits or command-line length restrictions.

### [windows.rs](file:///home/juno/slopchop/src/clipboard/windows.rs)
Implements Windows-specific clipboard support using the native `clip` command for simple text and PowerShell's `Set-Clipboard` for file handles. Just like the macOS and Linux implementations, it ensures that Windows developers can easily copy large blocks of code or whole-project contexts as file attachments, maintaining the tool's transactional integrity across all major operating systems.

---

## src/config/

### [mod.rs](file:///home/juno/slopchop/src/config/mod.rs)
The centralized configuration entry point for the entire application. It provides the `Config` struct and the high-level `load` logic that aggregates project-specific settings from multiple sources. It serves as the primary gateway for other subsystems to query global rules, architectural constraints, and user preferences, ensuring that every SlopChop action is performed according to the project's unique "Laws of Physics."

### [io.rs](file:///home/juno/slopchop/src/config/io.rs)
Handles the persistence and retrieval of configuration data from the filesystem. It manages the reading of `slopchop.toml` and `.slopchopignore` files, and includes logic to automatically detect the project type (Rust, Node, etc.) to apply sensible default check and fix commands. This module also provides the serialization logic used by the interactive TUI to save user-edited settings back to disk.

### [locality.rs](file:///home/juno/slopchop/src/config/locality.rs)
Defines the specialized configuration parameters for the Law of Locality enforcement system. This includes strict thresholds for topological "distance", hub classification criteria, and god-module limits. By decoupling these architecture-specific constants into their own module, SlopChop makes it easy for teams to fine-tune their topological integrity rules without modifying the core validation engine.

### [types.rs](file:///home/juno/slopchop/src/config/types.rs)
The "source of truth" for SlopChop's data model. It defines the structured TOML schemas for rules, preferences, and command mappings. This file also contains the hard-coded default values for critical codebase health metrics—such as the 2000-token file limit and 25-cognitive-complexity cap—which act as the fallback "physics" for any project without a custom configuration.

---

## Final Entry Points

### [src/lib.rs](file:///home/juno/slopchop/src/lib.rs)
The root of the `slopchop_core` library. It defines the entire module hierarchy of the project, orchestrating the relationships between analysis engines, application logic, and utility subsystems. As the primary library entry point, it provides external consumers with a unified interface to SlopChop’s full suite of tools, from holographic repository packing to Law of Locality enforcement. Consistent with the LCOM4 principle, it serves as the high-level map that links the diverse components of the codebase into a cohesive toolkit.

### [src/bin/slopchop.rs](file:///home/juno/slopchop/src/bin/slopchop.rs)
The main executable entry point for the SlopChop command-line interface. It is a thin wrapper that uses `clap` to parse incoming commands and arguments, delegating the actual execution to the library's CLI dispatcher. By mapping library-level results to standardized exit codes, it provides a stable contract for automation and CI/CD pipelines. This file ensures that the powerful codebase management logic of the core library is accessible through a high-performance, user-friendly terminal interface.

### [src/main.rs](file:///home/juno/slopchop/src/main.rs)
A minimal, vestigial entry point located at the source root. In the current architecture, SlopChop’s main execution logic has been moved to the `src/bin/` directory to support clear separation between library and executable. Consequently, this file remains as a placeholder or relic of earlier development phases and does not contribute to the production binary. It is effectively "deadwood" in the current project structure, though it serves to mark the root source directory.
