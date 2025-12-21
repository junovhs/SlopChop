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
    ///
    /// # Errors
    /// Returns error if prompt generation fails (currently infallible).
    pub fn generate(&self) -> Result<String> {
        Ok(self.build_system_prompt())
    }

    /// Generates the concise reminder prompt.
    ///
    /// # Errors
    /// Returns error if reminder generation fails (currently infallible).
    pub fn generate_reminder(&self) -> Result<String> {
        Ok(self.build_reminder())
    }

    /// Wraps the prompt with header/footer.
    ///
    /// # Errors
    /// Returns error if generation fails.
    pub fn wrap_header(&self) -> Result<String> {
        self.generate()
    }

    fn build_system_prompt(&self) -> String {
        let tokens = self.config.max_file_tokens;
        let complexity = self.config.max_cyclomatic_complexity;
        let depth = self.config.max_nesting_depth;
        let args = self.config.max_function_args;
        let sigil = "XSC7XSC";

        format!(
            r"SYSTEM MANDATE: THE SLOPCHOP PROTOCOL
ROLE: High-Integrity Systems Architect.
CONTEXT: You are coding inside a strict environment enforced by SlopChop.

THE 3 LAWS:
1. LAW OF ATOMICITY: Files MUST be < {tokens} tokens.
2. LAW OF COMPLEXITY: Cyclomatic Complexity <= {complexity}, Nesting <= {depth}, Args <= {args}.
3. LAW OF PARANOIA: No .unwrap() or .expect(). Use Result types for error handling.

OUTPUT FORMAT (MANDATORY):
All responses must use the {sigil} DNA sequence sigil. Do NOT use markdown code blocks.

1. Technical Plan:
{sigil} PLAN {sigil}
GOAL: <summary>
CHANGES: <list>
{sigil} END {sigil}

2. Manifest:
{sigil} MANIFEST {sigil}
path/to/file.rs
path/to/new_file.rs [NEW]
{sigil} END {sigil}

3. File Delivery:
{sigil} FILE {sigil} path/to/file.rs
<raw code content>
{sigil} END {sigil}

RULES:
- No truncation. Provide full file contents.
- To bypass truncation detection on a specific line, append '// slopchop:ignore' to that line.
- No markdown fences around code blocks. The {sigil} markers are the fences."
        )
    }

    fn build_reminder(&self) -> String {
        let sigil = "XSC7XSC";
        format!(
            r"SLOPCHOP CONSTRAINTS:
- File Tokens < {}
- Complexity <= {}
- Use {sigil} Sigil Protocol (PLAN, MANIFEST, FILE)",
            self.config.max_file_tokens, self.config.max_cyclomatic_complexity
        )
    }
}
