use crate::term::TERM_SIZE;
use console::measure_text_width;
use once_cell::sync::Lazy;
use regex::Regex;

// Regex to match pad() calls, including nested ones
static PAD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // This pattern matches:
    // - pad( followed by any content that doesn't contain unmatched parentheses
    // - The content can include nested pad() calls
    // - Ends with )
    Regex::new(r"pad\((?:[^()]|\([^()]*\))*\)").unwrap()
});

pub struct TextProcessor {
    get_width: Box<dyn Fn() -> usize + Send + Sync>,
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new(Box::new(|| TERM_SIZE.get_term_width()))
    }
}

impl TextProcessor {
    /// Creates a new TextProcessor with a custom width provider
    ///
    /// # Arguments
    /// * `width_provider` - A function that returns the width of the terminal
    ///
    /// # Returns
    /// A new TextProcessor with the specified width provider
    pub fn new(width_provider: Box<dyn Fn() -> usize + Send + Sync>) -> Self {
        Self {
            get_width: width_provider,
        }
    }

    /// Creates a new TextProcessor with a constant width
    /// # Arguments
    /// * `width` - The width to use for the TextProcessor
    ///
    /// # Returns
    /// A new TextProcessor with the specified width
    pub fn with_width(width: usize) -> Self {
        Self::new(Box::new(move || width))
    }

    /// Process the content with padding and line wrapping
    ///
    /// # Arguments
    /// * `content` - The content to process
    ///
    /// # Returns
    /// A string with the processed content
    pub fn process_padding(&self, content: &str) -> String {
        content.to_string()
    }
}
