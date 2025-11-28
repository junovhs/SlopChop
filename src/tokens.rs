// src/tokens.rs
use std::sync::LazyLock;
use tiktoken_rs::CoreBPE;

/// The tokenizer encoding (cl100k_base, used by GPT-4/3.5-turbo).
/// Initialization is deferred until first use. If the encoding fails to load
/// (which should never happen with a valid tiktoken-rs installation),
/// token counting will return 0 and log an error.
static BPE: LazyLock<Option<CoreBPE>> = LazyLock::new(|| {
    tiktoken_rs::cl100k_base()
        .map_err(|e| eprintln!("Failed to load cl100k_base tokenizer: {e}"))
        .ok()
});

pub struct Tokenizer;

impl Tokenizer {
    /// Counts the number of tokens in the given text.
    /// Returns 0 if the tokenizer failed to initialize.
    #[must_use]
    pub fn count(text: &str) -> usize {
        BPE.as_ref()
            .map(|bpe| bpe.encode_ordinary(text).len())
            .unwrap_or(0)
    }

    /// Returns true if the text exceeds the token limit.
    #[must_use]
    pub fn exceeds_limit(text: &str, limit: usize) -> bool {
        Self::count(text) > limit
    }

    /// Returns true if the tokenizer is available.
    #[must_use]
    pub fn is_available() -> bool {
        BPE.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer_available() {
        assert!(Tokenizer::is_available());
    }

    #[test]
    fn test_count_basic() {
        let count = Tokenizer::count("hello world");
        assert!(count > 0);
    }

    #[test]
    fn test_exceeds_limit() {
        assert!(!Tokenizer::exceeds_limit("hi", 100));
        assert!(Tokenizer::exceeds_limit("hello world this is a test", 1));
    }
}
