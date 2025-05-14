use crate::term::TERM_SIZE;
use crate::utils;
use console::measure_text_width;
use once_cell::sync::Lazy;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

/// Represents a matched padding group with its position and width information
struct MatchedGroup {
    content: String,
    start: usize,
    end: usize,
}

// Regex to match pad() calls, including nested ones
static PAD_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pad\(((?:[^()]|\([^()]*\))*)\)").unwrap());

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
    /// * `content` - The content to process, can be multiline
    ///
    /// # Returns
    /// A string with the processed content
    pub fn process_padding(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| self.process_padding_line(line))
            .collect::<Vec<String>>()
            .join("\n")
    }

    /// Process a single line of content with padding and line wrapping
    ///
    /// # Arguments
    /// * `content` - The content to process
    ///
    /// # Returns
    /// A string with the processed content
    fn process_padding_line(&self, content: &str) -> String {
        // Find all pad() groups
        let pad_groups: Vec<_> = PAD_PATTERN.captures_iter(content).collect();

        // If no padding groups, return the content as is
        if pad_groups.is_empty() {
            return content.to_string();
        }

        // Recursively process the content inside each pad() group
        let mut processed = content.to_string();
        for cap in PAD_PATTERN.captures_iter(content) {
            if let Some(m) = cap.get(1) {
                let inner = self.process_padding_line(m.as_str());
                let full = cap.get(0).unwrap();
                processed.replace_range(full.start()..full.end(), &format!("pad({})", inner));
            }
        }

        // Now extract the pad() groups again from the processed string
        let pad_groups: Vec<_> = PAD_PATTERN.captures_iter(&processed).collect();
        let (padding_groups, occupied_space, non_empty_groups) =
            self.extract_padding_groups(&processed, &pad_groups);
        let remaining_space = (self.get_width)().saturating_sub(occupied_space);
        let space_per_group = remaining_space / non_empty_groups;
        let extra_space = remaining_space % non_empty_groups;
        let mut result = processed.to_string();
        for (i, group) in padding_groups.iter().rev().enumerate() {
            let start = group.start;
            let end = group.end;
            let replacement = if !group.content.is_empty() {
                // For the last group, add any extra space from the remainder
                let space = if i == non_empty_groups - 1 {
                    space_per_group + extra_space
                } else {
                    space_per_group
                };
                let expanded = utils::expand_to_display_width(&group.content, space);
                format!("{}{}", group.content, expanded)
            } else {
                String::new()
            };
            result.replace_range(start..end, &replacement);
        }
        result
    }

    /// Extract padding groups from the content and calculate their information
    ///
    /// # Arguments
    /// * `content` - The content to process
    /// * `pad_groups` - The regex matches for padding groups
    ///
    /// # Returns
    /// A tuple containing:
    /// - Vector of padding group information
    /// - Total occupied space (outside text + padding content)
    /// - Number of non-empty padding groups
    fn extract_padding_groups(
        &self,
        content: &str,
        pad_groups: &[regex::Captures],
    ) -> (Vec<MatchedGroup>, usize, usize) {
        let mut padding_groups = Vec::new();
        let mut occupied_space = 0;
        let mut non_empty_groups = 0;
        let mut last_end = 0;

        for group in pad_groups {
            let full_match = group.get(0).unwrap();
            let pad_content = &group[1];

            // Add the length of text between the last match and this one
            // Use the full_match's byte range to safely slice the string
            if last_end < full_match.start() {
                occupied_space += measure_text_width(&content[last_end..full_match.start()]);
            }

            let content_width = measure_text_width(pad_content);
            occupied_space += content_width;

            let matched_group = MatchedGroup {
                content: pad_content.to_string(),
                start: full_match.start(),
                end: full_match.end(),
            };

            if !matched_group.content.is_empty() {
                non_empty_groups += 1;
            }

            padding_groups.push(matched_group);

            // Update last_end for the next iteration
            last_end = full_match.end();
        }

        // Add the length of any remaining text after the last match
        if last_end < content.len() {
            occupied_space += measure_text_width(&content[last_end..]);
        }

        (padding_groups, occupied_space, non_empty_groups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to debug test output
    fn debug_output(result: &str) {
        println!("\nDebug output for: {}", result);
        println!("Total width: {}", measure_text_width(result));
        println!("Character by character:");
        for (i, c) in result.char_indices() {
            println!(
                "  {}: '{}' (width: {})",
                i,
                c,
                measure_text_width(&c.to_string())
            );
        }
    }

    #[test]
    fn test_process_padding_no_padding() {
        let processor = TextProcessor::with_width(80);
        let input = "Hello, World!";
        assert_eq!(processor.process_padding(input), input);
    }

    #[test]
    fn test_process_padding_single_padding() {
        let processor = TextProcessor::with_width(20);
        let input = "Hello pad(+) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello ++++++++ World");
        assert_eq!(measure_text_width(&result), 20);
    }

    #[test]
    fn test_process_padding_multiple_padding() {
        let processor = TextProcessor::with_width(30);
        let input = "Hello pad(+) pad(-) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello +++++++++ -------- World");
        assert_eq!(measure_text_width(&result), 30);
    }

    #[test]
    fn test_process_padding_with_emoji() {
        let processor = TextProcessor::with_width(20);
        let input = "Hello pad(🦀) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello 🦀 World");
        assert_eq!(measure_text_width(&result), 14);
    }

    #[test]
    fn test_process_padding_with_family_emoji() {
        let processor = TextProcessor::with_width(30);
        let input = "Hello pad(👨‍👩‍👧‍👦) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello 👨‍👩‍👧‍👦 World");
        assert_eq!(measure_text_width(&result), 14);
    }

    #[test]
    fn test_process_padding_multiline() {
        let processor = TextProcessor::with_width(20);
        let input = "Hello pad(+)\nWorld pad(-)";
        let result = processor.process_padding(input);
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
        debug_output(lines[0]);
        debug_output(lines[1]);
        assert_eq!(lines[0], "Hello +");
        assert_eq!(lines[1], "World -");
        assert_eq!(measure_text_width(lines[0]), 7);
        assert_eq!(measure_text_width(lines[1]), 7);
    }

    #[test]
    fn test_process_padding_empty_content() {
        let processor = TextProcessor::with_width(20);
        let input = "pad()";
        let result = processor.process_padding(input);
        assert_eq!(result, "");
    }

    #[test]
    fn test_process_padding_nested_padding() {
        let processor = TextProcessor::with_width(30);
        let input = "Hello pad(pad(+)) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello + World");
        assert_eq!(measure_text_width(&result), 13);
    }

    #[test]
    fn test_process_padding_exact_division() {
        let processor = TextProcessor::with_width(20);
        let input = "Hello pad(+) pad(+) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello + + World");
        assert_eq!(measure_text_width(&result), 15);
    }

    #[test]
    fn test_process_padding_mixed_content() {
        let processor = TextProcessor::with_width(30);
        let input = "Hello pad(🦀) pad(+) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello 🦀 + World");
        assert_eq!(measure_text_width(&result), 16);
    }

    #[test]
    fn test_process_padding_with_rainbow_flag() {
        let processor = TextProcessor::with_width(30);
        let input = "Hello pad(🏳️‍🌈) World";
        let result = processor.process_padding(input);
        debug_output(&result);
        assert_eq!(result, "Hello 🏳️‍🌈 World");
        assert!(measure_text_width(&result) >= 14); // Rainbow flag width can vary
    }
}
