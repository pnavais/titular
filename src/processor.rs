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
static PAD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // This pattern matches:
    // - pad( followed by any content that doesn't contain unmatched parentheses
    // - The content can include nested pad() calls
    // - Ends with )
    // - Captures the content inside the parentheses
    Regex::new(r"pad\(((?:[^()]|\([^()]*\))*)\)").unwrap()
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

        let (padding_groups, occupied_space, non_empty_groups) =
            self.extract_padding_groups(content, &pad_groups);
        let remaining_space = (self.get_width)().saturating_sub(occupied_space);
        let space_per_group =
            ((remaining_space as f64) / (non_empty_groups as f64)).ceil() as usize;
        let is_exact_division = (remaining_space as f64) % (non_empty_groups as f64) == 0.0;

        // Replace each padding group with its content followed by its expanded content
        let mut result = content.to_string();
        for (i, group) in padding_groups.iter().rev().enumerate() {
            let start = group.start;
            let end = group.end;
            let replacement = if !group.content.is_empty() {
                // If this is the first group (from right) and division isn't exact
                let space = if i == 0 && !is_exact_division {
                    let base_space = space_per_group - 1;
                    // For single graphemes (emojis or other single characters),
                    // ensure the space is a multiple of their width
                    if group.content.graphemes(true).count() == 1 {
                        let width = measure_text_width(&group.content);
                        base_space + (width - (base_space % width))
                    } else {
                        base_space
                    }
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
