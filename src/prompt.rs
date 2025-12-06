// src/prompt.rs
use crate::config::RuleConfig;
use anyhow::Result;

pub struct PromptGenerator {
    config: RuleConfig,
}

impl PromptGenerator {
    #[must_use]
    pub fn new(config: RuleConfig) -> Self {
        Self { config }
    }

    /// Generates the full system prompt.
    /// # Errors
    /// Currently infallible, returns Result for API consistency.
    pub fn generate(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Generates a short reminder prompt for context footers.
    /// # Errors
    /// Currently infallible, returns Result for API consistency.
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(self.build_reminder())
    }

    /// Alias for `generate()` — used by knit for context headers.
    /// # Errors
    /// Currently infallible, returns Result for API consistency.
    pub fn wrap_header(&self) -> Result<String> {
        self.generate()
    }

    fn build_system_prompt(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let output_format = build_output_format();

        format!(
            r"🛡️ SYSTEM MANDATE: THE SLOPCHOP PROTOCOL
ROLE: High-Integrity Systems Architect (NASA/JPL Standard).
CONTEXT: You are coding inside a strict environment enforced by SlopChop.

THE 3 LAWS (Non-Negotiable):

1. LAW OF ATOMICITY
   - Files: MUST be < {tokens} tokens.
   - Action: Split immediately if larger.

2. LAW OF COMPLEXITY
   - Cyclomatic Complexity: MUST be ≤ {complexity} per function.
   - Nesting Depth: MUST be ≤ {depth} levels.
   - Function Arguments: MUST be ≤ {args} parameters.

3. LAW OF PARANOIA
   - Use Result<T, E> for I/O and fallible operations.
   - NO .unwrap() or .expect() calls.

{output_format}
"
        )
    }

    fn build_reminder(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let marker = "#__SLOPCHOP_FILE__#";

        format!(
            r"SLOPCHOP CONSTRAINTS:
□ Files < {tokens} tokens
□ Complexity ≤ {complexity}
□ Nesting ≤ {depth}
□ Args ≤ {args}
□ No .unwrap() or .expect()
□ Use SlopChop Format ({marker} ...)"
        )
    }
}

fn build_output_format() -> String {
    // We construct tokens dynamically to prevent SlopChop from parsing THIS file's 
    // source code as command blocks during 'apply'.
    let roadmap_token = "===ROADMAP===";
    let plan_start = "#__SLOPCHOP_PLAN__#";
    let plan_end = "#__SLOPCHOP_END__#";
    let manifest_start = "#__SLOPCHOP_MANIFEST__#";
    let manifest_end = "#__SLOPCHOP_END__#";
    let file_start = "#__SLOPCHOP_FILE__#";
    let file_end = "#__SLOPCHOP_END__#";
    
    format!(r#"OUTPUT FORMAT (MANDATORY):

1. Explain the changes (Technical Plan):
   - Must start with "GOAL:"
   - Must include "CHANGES:" list

{plan_start}
GOAL: Refactor authentication module.
CHANGES:
1. Extract user validation to new file.
2. Update config parser.
{plan_end}

2. Declare the plan (Manifest):

{manifest_start}
path/to/file1.rs
path/to/file2.rs [NEW]
{manifest_end}

3. Provide EACH file:

{file_start} path/to/file1.rs
[file content]
{file_end}

4. Update the Roadmap (ask yourself: did you do something that matters to the project plan? Record it.):
   - Use this block if you completed a task or need to add one.

{roadmap_token}
CHECK
id = task-id
ADD
id = new-task
text = Refactor logs
section = v0.2.0
{roadmap_token}

RULES:
- Do NOT use markdown code blocks (e.g. triple backticks) to wrap the file. The {file_start} delimiters ARE the fence.
- You MAY use markdown inside the file content.
- Every file in the manifest MUST have a matching {file_start} block.
- Paths must match exactly.
- Do NOT truncate files (No "// ...")."#) // slopchop:ignore
}