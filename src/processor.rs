use crate::term::TERM_SIZE;
use crate::utils;
use console::strip_ansi_codes;
use once_cell::sync::Lazy;
use regex::Regex;
use unicode_segmentation::UnicodeSegmentation;

/// Represents a matched padding group with its position and width information
struct MatchedGroup {
    content: String,
    start: usize,
    end: usize,
    grapheme_count: usize,
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

        let (padding_groups, occupied_space, non_empty_groups) =
            self.extract_padding_groups(&content, &pad_groups);

        let target_width = (self.get_width)();
        let remaining_space = target_width.saturating_sub(occupied_space);
        let space_per_group = if non_empty_groups > 0 {
            (remaining_space as f64 / non_empty_groups as f64).ceil() as usize
        } else {
            0
        };

        println!("occupied_space: {}", occupied_space);
        println!("width: {}", target_width);
        println!("non_empty_groups: {}", non_empty_groups);
        println!("remaining_space: {}", remaining_space);
        println!("space_per_group: {}", space_per_group);

        // Debug padding groups
        println!("\nPadding Groups:");
        for (i, group) in padding_groups.iter().enumerate() {
            println!("Group {}:", i);
            println!("  Content: '{}'", group.content);
            println!("  Start: {}", group.start);
            println!("  End: {}", group.end);
            println!("  Width: {}", group.grapheme_count);
            println!("  Is Empty: {}", group.content.is_empty());
        }

        content.to_string()
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

        // Find matches in the content
        for group in pad_groups {
            let full_match = group.get(0).unwrap();
            let pad_content = &group[1];

            // Add the length of text between the last match and this one
            if last_end < full_match.start() {
                let between_text = &content[last_end..full_match.start()];
                let stripped_between = strip_ansi_codes(between_text);
                let between_width = stripped_between.graphemes(true).count();
                occupied_space += between_width;
            }

            // Count graphemes in the padding content
            let stripped_pad = strip_ansi_codes(pad_content);
            let content_width = stripped_pad.graphemes(true).count();
            occupied_space += content_width;

            let matched_group = MatchedGroup {
                content: pad_content.to_string(),
                start: full_match.start(),
                end: full_match.end(),
                grapheme_count: content_width,
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
            let remaining_text = &content[last_end..];
            let stripped_remaining = strip_ansi_codes(remaining_text);
            let remaining_width = stripped_remaining.graphemes(true).count();
            occupied_space += remaining_width;
        }

        (padding_groups, occupied_space, non_empty_groups)
    }
}
