use crate::context::Context;
use crate::error::Result;
use crate::string_utils::{expand_to_width, AnsiTruncateBehavior, Truncate};
use crate::term::TERM_SIZE;
use crate::transforms::Transform;
use console::strip_ansi_codes;
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{Arc, Mutex};
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
    get_width: Arc<Mutex<Box<dyn Fn() -> usize + Send + Sync>>>,
}

impl Default for TextProcessor {
    fn default() -> Self {
        Self::new(Self::default_width())
    }
}

/// TextProcessor is a transform that processes the text with padding groups.
/// It is used to process functions that need global line width information for applying
/// operations like padding and line wrapping.
impl TextProcessor {
    /// Returns the default width function
    fn default_width() -> Box<dyn Fn() -> usize + Send + Sync> {
        Box::new(|| TERM_SIZE.get_term_width())
    }

    /// Creates a new TextProcessor with a custom width provider
    ///
    /// # Arguments
    /// * `width_provider` - A function that returns the width of the terminal
    ///
    /// # Returns
    /// A new TextProcessor with the specified width provider
    pub fn new(width_provider: Box<dyn Fn() -> usize + Send + Sync>) -> Self {
        Self {
            get_width: Arc::new(Mutex::new(width_provider)),
        }
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
        let pad_groups: Vec<_> = PAD_PATTERN.captures_iter(content).collect();
        if pad_groups.is_empty() {
            return content.to_string();
        }

        let (padding_groups, occupied_space, non_empty_groups) =
            self.extract_padding_groups(content, &pad_groups);
        let target_width = self.get_width.lock().unwrap()();
        let space_per_group = if non_empty_groups > 0 {
            (target_width.saturating_sub(occupied_space) as f64 / non_empty_groups as f64).ceil()
                as usize
        } else {
            0
        };

        let mut result = content.to_string();
        padding_groups.iter().rev().for_each(|group| {
            result.replace_range(
                group.start..group.end,
                &format!(
                    "{}{}",
                    group.content,
                    expand_to_width(&group.content, space_per_group)
                ),
            );
        });

        result.truncate_ansi_with(target_width, AnsiTruncateBehavior::ResetAfter);
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
        let mut last_end = 0;
        let mut occupied_space = 0;

        let padding_groups: Vec<MatchedGroup> = pad_groups
            .iter()
            .map(|group| {
                let full_match = group.get(0).unwrap();
                let pad_content = &group[1];

                // Add space for text between matches
                if last_end < full_match.start() {
                    let between_text = &content[last_end..full_match.start()];
                    let stripped_between = strip_ansi_codes(between_text);
                    occupied_space += stripped_between.graphemes(true).count();
                }

                // Count graphemes in padding content
                let stripped_pad = strip_ansi_codes(pad_content);
                occupied_space += stripped_pad.graphemes(true).count();

                // Update last_end for next iteration
                last_end = full_match.end();

                MatchedGroup {
                    content: pad_content.to_string(),
                    start: full_match.start(),
                    end: full_match.end(),
                }
            })
            .collect();

        // Add space for remaining text
        if last_end < content.len() {
            let remaining_text = &content[last_end..];
            let stripped_remaining = strip_ansi_codes(remaining_text);
            occupied_space += stripped_remaining.graphemes(true).count();
        }

        let non_empty_groups = padding_groups
            .iter()
            .filter(|g| !g.content.is_empty())
            .count();

        (padding_groups, occupied_space, non_empty_groups)
    }
}

impl Transform for TextProcessor {
    fn transform(&self, context: Arc<Context>, text: &str) -> Result<String> {
        // Check if context has a width parameter
        if let Some(width) = context.get("width").and_then(|w| w.parse::<u8>().ok()) {
            *self.get_width.lock().unwrap() = Box::new(move || {
                let term_width = Self::default_width()();
                (term_width * width as usize) / 100
            });
        }
        Ok(self.process_padding(text))
    }
}
