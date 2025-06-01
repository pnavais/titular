use crate::error::Result;

/// Trait for text transformations in the formatter chain
///
/// This trait defines the core behavior for all text transformations in the system.
/// Each transform is responsible for a specific aspect of text processing, such as:
/// - Template rendering
/// - Text padding and line wrapping
/// - Line ending handling
/// - ANSI color formatting
///
/// # Examples
///
/// ```
/// use titular::transforms::{Transform, AnsiFormatter};
///
/// let formatter = AnsiFormatter::new();
/// let result = formatter.transform("\x1b[31mRed\x1b[0m").unwrap();
/// assert_eq!(result, "\x1b[31mRed\x1b[0m");
/// ```
pub trait Transform: Send + Sync {
    /// Transforms the input text using the global context
    ///
    /// # Arguments
    /// * `text` - The text to transform
    ///
    /// # Returns
    /// The transformed text or an error if transformation fails
    ///
    /// # Examples
    ///
    /// ```
    /// use titular::transforms::{Transform, AnsiFormatter};
    ///
    /// let formatter = AnsiFormatter::new();
    /// let result = formatter.transform("Hello").unwrap();
    /// assert_eq!(result, "Hello");
    /// ```
    fn transform(&self, text: &str) -> Result<String>;
}
