use console::{measure_text_width, strip_ansi_codes};
use print_positions::print_positions;

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

        *self = process_ansi_escapes(&result, self, current_pos, behavior);
    }
}

/// Process ANSI escape sequences according to the specified behavior
fn process_ansi_escapes(
    truncated: &str,
    original: &str,
    truncate_pos: usize,
    behavior: AnsiTruncateBehavior,
) -> String {
    match behavior {
        AnsiTruncateBehavior::PreserveRemaining => {
            let mut result = String::from(truncated);
            let mut remaining_pos = truncate_pos;

            // Collect all ANSI codes after truncation
            while remaining_pos < original.len() {
                if original[remaining_pos..].starts_with("\x1b[") {
                    if let Some(end) = original[remaining_pos..].find('m') {
                        let ansi_seq = &original[remaining_pos..remaining_pos + end + 1];
                        result.push_str(ansi_seq);
                        remaining_pos += end + 1;
                        continue;
                    }
                }
                // Move to next character boundary
                if let Some(c) = original[remaining_pos..].chars().next() {
                    remaining_pos += c.len_utf8();
                } else {
                    break;
                }
            }
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
    fn test_truncate_ansi() {
        // Test basic ASCII truncation
        let mut s = String::from("Hello World");
        s.truncate_ansi(5);
        println!("Test 1: {}|{}\n------------", s, s.len());
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
        // Test with multiple ANSI codes
        let mut s = String::from("\x1b[31mHello 🦀\x1b[32m World\x1b[0m");
        s.truncate_ansi_with(7, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "\x1b[31mHello \x1b[32m\x1b[0m");

        // Test with nested ANSI codes
        let mut s = String::from("\x1b[1m\x1b[31mBold Red\x1b[32mGreen\x1b[0m");
        s.truncate_ansi_with(5, AnsiTruncateBehavior::PreserveRemaining);
        assert_eq!(s, "\x1b[1m\x1b[31mBold \x1b[32m\x1b[0m");
    }
}
