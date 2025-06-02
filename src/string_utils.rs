use ansi_parser::{AnsiParser, Output};
use console::{measure_text_width, strip_ansi_codes};
use print_positions::print_positions;
use unicode_general_category::{get_general_category, GeneralCategory};

/// Defines how ANSI codes should be handled after truncation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiTruncateBehavior {
    /// Preserve all ANSI codes that appear after the truncation point
    PreserveRemaining,
    /// Always add a reset ANSI code (\x1b[0m) after truncation
    ResetAfter,
    /// Do not modify ANSI codes after truncation
    NoModification,
}

/// Check if a string is visually empty (contains only control characters, ANSI codes, or other non-printable characters)
///
/// # Arguments
/// * `s` - The string to check
///
/// # Returns
/// `true` if the string is empty or contains only control characters, ANSI codes, or zero-width characters.
/// `false` if the string contains any visible characters (including spaces, tabs, newlines).
///
/// # Examples
/// ```
/// use titular::string_utils::is_visually_empty;
///
/// assert!(is_visually_empty("")); // Empty string
/// assert!(is_visually_empty("\x1b[31m\x1b[0m")); // Only ANSI codes
/// assert!(is_visually_empty("\u{200B}")); // Zero-width space (format character)
/// assert!(is_visually_empty("\u{FEFF}")); // Zero-width no-break space (format character)
/// assert!(!is_visually_empty("   ")); // Spaces are visually present
/// assert!(!is_visually_empty("\t\n")); // Tabs and newlines are visually present
/// assert!(!is_visually_empty("Hello")); // Has visible text
/// assert!(!is_visually_empty("\x1b[31mHello\x1b[0m")); // Has visible text with ANSI codes
/// ```
pub fn is_visually_empty(s: &str) -> bool {
    // First strip ANSI codes
    let stripped = strip_ansi_codes(s);
    // Then check if what remains is empty or only contains control/format characters
    stripped.chars().all(|c| {
        // A character is visually empty if it's not whitespace AND it's a control/format character
        !c.is_whitespace()
            && matches!(
                get_general_category(c),
                GeneralCategory::Control
                    | GeneralCategory::Format
                    | GeneralCategory::Unassigned
                    | GeneralCategory::PrivateUse
                    | GeneralCategory::Surrogate
            )
    })
}

/// Prints a string with its raw ANSI codes
///
/// # Arguments
/// * `title` - The title of the string
/// * `text` - The string to print
///
/// # Examples
/// ```
/// use titular::string_utils::print_raw_ansi;
///
/// print_raw_ansi("Hello", "\x1b[31mHello\x1b[0m");
/// ```
pub fn print_raw_ansi(title: &str, text: &str) {
    println!("{}: [{}]", title, text.replace("\x1b", "\\x1b"));
}

/// Expands a string to a target width by repeating its content.
/// The width is calculated based on graphemes (print positions), ignoring ANSI codes
/// and non-displayable characters.
///
/// # Arguments
///
/// * `input` - The input string to expand
/// * `target_width` - The target width in graphemes
///
/// # Returns
///
/// A string expanded to the target width
///
/// # Examples
///
/// ```
/// use titular::string_utils::expand_to_width;
///
/// assert_eq!(expand_to_width("X", 2), "XX");
/// assert_eq!(expand_to_width("XY", 3), "XYX");
/// assert_eq!(expand_to_width("🦀", 2), "🦀🦀");
/// assert_eq!(expand_to_width("🦀🌟", 3), "🦀🌟🦀");
/// ```
pub fn expand_to_width(input: &str, target_width: usize) -> String {
    // Collect the "print positions" (user-visible glyphs, including ANSI)
    let positions: Vec<&str> = print_positions(input)
        .map(|(start, end)| &input[start..end])
        .collect();

    // If input is empty or has no visible positions, return as is
    if positions.is_empty() {
        return input.to_string();
    }

    // Calculate current visible width
    let current_width = positions.len();

    // If target_width is 0 or less than or equal to current width, return input as-is
    if target_width == 0 || current_width >= target_width {
        return input.to_string();
    }

    // Repeat positions to build up to target width
    let mut result = String::new();
    let mut i = 0;
    while i < target_width {
        for pos in &positions {
            if i >= target_width {
                break;
            }
            result.push_str(pos);
            i += 1;
        }
    }
    result
}

/// Expands a string to a target visual width by repeating its content.
/// The width is calculated based on the actual display width of characters,
/// taking into account wide characters like emojis.
///
/// # Arguments
///
/// * `input` - The input string to expand
/// * `target_width` - The target width in display units
///
/// # Returns
///
/// A string expanded to the target visual width
///
/// # Examples
///
/// ```
/// use titular::string_utils::expand_to_visual_width;
///
/// assert_eq!(expand_to_visual_width("X", 2), "XX");
/// assert_eq!(expand_to_visual_width("XY", 3), "XYX");
/// assert_eq!(expand_to_visual_width("📦", 4), "📦📦"); // Each emoji is 2 units wide
/// assert_eq!(expand_to_visual_width("📦🌟", 6), "📦🌟📦"); // Each emoji is 2 units wide
/// ```
pub fn expand_to_visual_width(input: &str, target_width: usize) -> String {
    // If input is empty, return as is
    if input.is_empty() {
        return input.to_string();
    }

    // Calculate current visual width
    let current_width = measure_text_width(input);

    // If target_width is 0 or less than or equal to current width, return input as-is
    if target_width == 0 || current_width >= target_width {
        return input.to_string();
    }

    // Calculate how many times we need to repeat the input
    let repeat_count = (target_width + current_width - 1) / current_width;
    let mut result = String::with_capacity(input.len() * repeat_count);

    // Repeat the input
    for _ in 0..repeat_count {
        result.push_str(input);
    }

    // If we've exceeded the target width, truncate
    if measure_text_width(&result) > target_width {
        let mut truncated = result;
        truncated.truncate_ansi(target_width);
        truncated
    } else {
        result
    }
}

/// Trait for truncating strings while preserving ANSI codes
pub trait Truncate {
    /// Truncates a string to the specified width while preserving ANSI codes in place
    ///
    /// # Arguments
    /// * `width` - The maximum width in characters
    fn truncate_ansi(&mut self, width: usize) {
        self.truncate_ansi_with(width, AnsiTruncateBehavior::NoModification)
    }

    /// Truncates a string to the specified width with configurable ANSI code handling
    ///
    /// # Arguments
    /// * `width` - The maximum width in characters
    /// * `behavior` - How to handle ANSI codes after truncation
    fn truncate_ansi_with(&mut self, width: usize, behavior: AnsiTruncateBehavior);
}

impl Truncate for String {
    /// Truncates a string to the specified width with configurable ANSI code handling
    ///
    /// # Arguments
    /// * `width` - The maximum width in characters
    /// * `behavior` - How to handle ANSI codes after truncation
    ///
    /// # Examples
    ///
    /// ```
    /// use titular::string_utils::{Truncate, AnsiTruncateBehavior};
    ///
    /// let mut s = String::from("\x1b[31mHello\x1b[0m World");
    /// s.truncate_ansi_with(5, AnsiTruncateBehavior::PreserveRemaining);
    /// assert_eq!(s, "\x1b[31mHello\x1b[0m");
    /// ```
    fn truncate_ansi_with(&mut self, width: usize, behavior: AnsiTruncateBehavior) {
        // Get the actual text width without ANSI codes
        let text_without_ansi = strip_ansi_codes(self);
        let text_width = measure_text_width(&text_without_ansi);

        // If text is already within width limit, return it as is
        if text_width <= width {
            return;
        }

        // Find the position where we need to cut the text
        let mut current_width = 0;
        let mut result = String::new();
        let mut current_pos = 0;

        while current_pos < self.len() {
            // Check if we're in an ANSI sequence
            if self[current_pos..].starts_with("\x1b[") {
                if let Some(end) = self[current_pos..].find('m') {
                    // Only include ANSI codes that come before our truncation point
                    if current_width < width {
                        let ansi_seq = &self[current_pos..current_pos + end + 1];
                        result.push_str(ansi_seq);
                    }
                    current_pos += end + 1;
                    continue;
                }
            }

            // Process regular characters
            if let Some(c) = self[current_pos..].chars().next() {
                let char_width = measure_text_width(&c.to_string());
                if current_width + char_width > width {
                    break;
                }
                current_width += char_width;
                result.push(c);
                current_pos += c.len_utf8();
            } else {
                break;
            }
        }

        *self = process_ansi_escapes(&result, self, behavior);
    }
}

/// Process ANSI escape sequences according to the specified behavior
///
/// # Arguments
/// * `truncated` - The truncated string
/// * `original` - The original string
/// * `behavior` - The behavior to apply to the ANSI codes
///
/// # Returns
/// A string with the ANSI codes processed according to the behavior
fn process_ansi_escapes(truncated: &str, original: &str, behavior: AnsiTruncateBehavior) -> String {
    match behavior {
        AnsiTruncateBehavior::PreserveRemaining => {
            let mut result = String::from(truncated);

            // Get the remaining part by stripping the truncated string from the beginning
            let remaining = original.strip_prefix(truncated).unwrap_or("");

            // Collect remaining ANSI codes using ansi-parser
            let codes: String = remaining
                .ansi_parse()
                .into_iter()
                .filter_map(|block| match block {
                    Output::Escape(seq) => Some(seq.to_string()),
                    _ => None,
                })
                .collect();

            result.push_str(&codes);
            result
        }
        AnsiTruncateBehavior::ResetAfter => {
            if truncated.contains("\x1b[") {
                format!("{}\x1b[0m", truncated.trim_end_matches("\x1b[0m"))
            } else {
                truncated.to_string()
            }
        }
        AnsiTruncateBehavior::NoModification => truncated.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_to_width() {
        // Test basic ASCII characters
        assert_eq!(expand_to_width("X", 0), "X");
        assert_eq!(expand_to_width("X", 1), "X");
        assert_eq!(expand_to_width("X", 2), "XX");
        assert_eq!(expand_to_width("X", 3), "XXX");

        // Test multi-character strings
        assert_eq!(expand_to_width("XY", 0), "XY");
        assert_eq!(expand_to_width("XY", 1), "XY");
        assert_eq!(expand_to_width("XY", 2), "XY");
        assert_eq!(expand_to_width("XY", 3), "XYX");
        assert_eq!(expand_to_width("XY", 4), "XYXY");

        // Test emojis
        assert_eq!(expand_to_width("🦀", 0), "🦀");
        assert_eq!(expand_to_width("🦀", 1), "🦀");
        assert_eq!(expand_to_width("🦀", 2), "🦀🦀");
        assert_eq!(expand_to_width("🦀🦀", 2), "🦀🦀");
        assert_eq!(expand_to_width("🦀🌟", 3), "🦀🌟🦀");
        assert_eq!(expand_to_width("🦀🌟", 4), "🦀🌟🦀🌟");

        // Test Mixed characters
        assert_eq!(expand_to_width("🦀-🦀é", 8), "🦀-🦀é🦀-🦀é");

        // Test Unicode characters
        assert_eq!(expand_to_width("é", 0), "é");
        assert_eq!(expand_to_width("é", 1), "é");
        assert_eq!(expand_to_width("é", 2), "éé");
        assert_eq!(expand_to_width("éè", 3), "éèé");
        assert_eq!(expand_to_width("éè", 4), "éèéè");

        // Test Japanese characters
        assert_eq!(expand_to_width("こ", 0), "こ");
        assert_eq!(expand_to_width("こ", 1), "こ");
        assert_eq!(expand_to_width("こ", 2), "ここ");
        assert_eq!(expand_to_width("こ", 3), "こここ");
        assert_eq!(expand_to_width("こに", 4), "こにこに");

        // Test Korean characters
        assert_eq!(expand_to_width("안녕", 0), "안녕");
        assert_eq!(expand_to_width("안녕", 1), "안녕");
        assert_eq!(expand_to_width("안녕", 2), "안녕");
        assert_eq!(expand_to_width("안녕", 3), "안녕안");
        assert_eq!(expand_to_width("안녕", 4), "안녕안녕");

        // Test ANSI escape codes
        assert_eq!(expand_to_width("\x1b[31mH\x1b[0m", 0), "\x1b[31mH\x1b[0m");
        assert_eq!(expand_to_width("\x1b[31mH\x1b[0m", 1), "\x1b[31mH\x1b[0m");
        assert_eq!(
            expand_to_width("\x1b[31mH\x1b[0m", 2),
            "\x1b[31mH\x1b[0m\x1b[31mH\x1b[0m"
        );
    }

    #[test]
    fn test_expand_to_visual_width() {
        // Test basic ASCII characters
        assert_eq!(expand_to_visual_width("X", 0), "X");
        assert_eq!(expand_to_visual_width("X", 1), "X");
        assert_eq!(expand_to_visual_width("X", 2), "XX");
        assert_eq!(expand_to_visual_width("X", 3), "XXX");

        // Test multi-character strings
        assert_eq!(expand_to_visual_width("XY", 0), "XY");
        assert_eq!(expand_to_visual_width("XY", 1), "XY");
        assert_eq!(expand_to_visual_width("XY", 2), "XY");
        assert_eq!(expand_to_visual_width("XY", 3), "XYX");
        assert_eq!(expand_to_visual_width("XY", 4), "XYXY");

        // Test emojis (each emoji is 2 units wide)
        assert_eq!(expand_to_visual_width("📦", 0), "📦");
        assert_eq!(expand_to_visual_width("📦", 1), "📦");
        assert_eq!(expand_to_visual_width("📦", 2), "📦");
        assert_eq!(expand_to_visual_width("📦", 3), "📦");
        assert_eq!(expand_to_visual_width("📦", 4), "📦📦");
        assert_eq!(expand_to_visual_width("📦", 5), "📦📦");
        assert_eq!(expand_to_visual_width("📦", 6), "📦📦📦");

        // Test mixed characters
        assert_eq!(expand_to_visual_width("📦-", 4), "📦-");
        assert_eq!(expand_to_visual_width("📦-", 5), "📦-📦"); // Visual width 3, needs to repeat to reach 5
        assert_eq!(expand_to_visual_width("📦-", 6), "📦-📦-");

        // Test ANSI escape codes
        assert_eq!(
            expand_to_visual_width("\x1b[31mH\x1b[0m", 0),
            "\x1b[31mH\x1b[0m"
        );
        assert_eq!(
            expand_to_visual_width("\x1b[31mH\x1b[0m", 1),
            "\x1b[31mH\x1b[0m"
        );
        assert_eq!(
            expand_to_visual_width("\x1b[31mH\x1b[0m", 2),
            "\x1b[31mH\x1b[0m\x1b[31mH\x1b[0m"
        );
    }

    #[test]
    fn test_truncate_ansi() {
        // Test basic ASCII truncation
        let mut s = String::from("Hello World");
        s.truncate_ansi(5);
        assert_eq!(s, "Hello");

        // Test truncation with ANSI colors
        let mut s = String::from("\x1b[31mHello\x1b[0m World");
        s.truncate_ansi(5);
        assert_eq!(s, "\x1b[31mHello");

        // Test truncation with emojis
        let mut s = String::from("Hello 🦀 World");
        s.truncate_ansi(8);
        assert_eq!(s, "Hello 🦀");
        let mut s = String::from("Hello 🦀 World");
        s.truncate_ansi(7);
        assert_eq!(s, "Hello ");

        // Test truncation with ANSI and emojis
        let mut s = String::from("\x1b[31mHello 🦀\x1b[0m World");
        s.truncate_ansi(8);
        assert_eq!(s, "\x1b[31mHello 🦀");
        let mut s = String::from("\x1b[31mHello 🦀\x1b[0m World");
        s.truncate_ansi(7);
        assert_eq!(s, "\x1b[31mHello ");

        // Test truncation with no change needed
        let mut s = String::from("Hello");
        s.truncate_ansi(10);
        assert_eq!(s, "Hello");

        // Test truncation to zero
        let mut s = String::from("Hello");
        s.truncate_ansi(0);
        assert_eq!(s, "");
    }

    #[test]
    fn test_truncate_ansi_with_reset_after() {
        // Test basic ASCII truncation
        let mut s = String::from("Hello World");
        s.truncate_ansi_with(5, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "Hello");

        // Test truncation with ANSI colors
        let mut s = String::from("\x1b[31mHello\x1b[0m World");
        s.truncate_ansi_with(5, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "\x1b[31mHello\x1b[0m");

        // Test truncation with emojis
        let mut s = String::from("Hello 🦀 World");
        s.truncate_ansi_with(8, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "Hello 🦀");
        let mut s = String::from("Hello 🦀 World");
        s.truncate_ansi_with(7, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "Hello ");

        // Test truncation with ANSI and emojis
        let mut s = String::from("\x1b[31mHello 🦀\x1b[0m World");
        s.truncate_ansi_with(8, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "\x1b[31mHello 🦀\x1b[0m");
        let mut s = String::from("\x1b[31mHello 🦀\x1b[0m World");
        s.truncate_ansi_with(7, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "\x1b[31mHello \x1b[0m");

        // Test truncation with no change needed
        let mut s = String::from("Hello");
        s.truncate_ansi_with(10, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "Hello");

        // Test truncation to zero
        let mut s = String::from("Hello");
        s.truncate_ansi_with(0, AnsiTruncateBehavior::ResetAfter);
        assert_eq!(s, "");
    }

    #[test]
    fn test_truncate_ansi_with_preserve_remaining() {
        // Test basic ASCII truncation
        let mut s = String::from("Hello World");
        s.truncate_ansi_with(5, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "Hello");

        // Test truncation with ANSI colors
        let mut s = String::from("\x1b[31mHello\x1b[0m World");
        s.truncate_ansi_with(5, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "\x1b[31mHello\x1b[0m");

        // Test truncation with emojis
        let mut s = String::from("Hello 🦀 World");
        s.truncate_ansi_with(8, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "Hello 🦀");

        // Test with multiple ANSI codes and emojis
        let mut s = String::from("\x1b[31mHello 🦀\x1b[32m World\x1b[0m");
        s.truncate_ansi_with(7, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "\x1b[31mHello \x1b[32m\x1b[0m");

        // Test with nested ANSI codes
        let mut s = String::from("\x1b[1m\x1b[31mBold Red\x1b[32mGreen\x1b[0m");
        s.truncate_ansi_with(5, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "\x1b[1m\x1b[31mBold \x1b[32m\x1b[0m");
    }

    #[test]
    fn test_is_visually_empty() {
        // Test empty strings
        assert!(is_visually_empty(""));
        assert!(!is_visually_empty("   ")); // Spaces are visually present
        assert!(!is_visually_empty("\t\n\r")); // Tabs and newlines are visually present

        // Test ANSI codes
        assert!(is_visually_empty("\x1b[31m\x1b[0m")); // Red color code
        assert!(is_visually_empty("\x1b[1m\x1b[0m")); // Bold
        assert!(is_visually_empty("\x1b[1;31m\x1b[0m")); // Bold red

        // Test control characters
        assert!(is_visually_empty("\u{200B}")); // Zero-width space
        assert!(is_visually_empty("\u{FEFF}")); // Zero-width no-break space
        assert!(is_visually_empty("\u{200D}")); // Zero-width joiner
        assert!(is_visually_empty("\u{200C}")); // Zero-width non-joiner
        assert!(is_visually_empty("\u{200E}")); // Left-to-right mark
        assert!(is_visually_empty("\u{200F}")); // Right-to-left mark
        assert!(is_visually_empty("\u{202A}")); // Left-to-right embedding
        assert!(is_visually_empty("\u{202B}")); // Right-to-left embedding
        assert!(is_visually_empty("\u{202C}")); // Pop directional formatting
        assert!(is_visually_empty("\u{202D}")); // Left-to-right override
        assert!(is_visually_empty("\u{202E}")); // Right-to-left override

        // Test mixed control characters
        assert!(is_visually_empty("\u{200B}\u{FEFF}\u{200D}"));
        assert!(!is_visually_empty("\u{200B} \u{FEFF}\t\u{200D}\n")); // Mixed with whitespace

        // Test non-empty strings
        assert!(!is_visually_empty("Hello"));
        assert!(!is_visually_empty("Hello World"));
        assert!(!is_visually_empty("\x1b[31mHello\x1b[0m"));
        assert!(!is_visually_empty("\x1b[31mHello World\x1b[0m"));
        assert!(!is_visually_empty("Hello\u{200B}World")); // Zero-width space between text
        assert!(!is_visually_empty("\u{200B}Hello\u{200B}")); // Zero-width space around text
        assert!(!is_visually_empty("\x1b[31mHello\u{200B}World\x1b[0m")); // Mixed ANSI and control
    }
}
