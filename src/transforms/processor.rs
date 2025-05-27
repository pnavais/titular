use crate::context_manager::ContextManager;
use crate::error::Result;
use crate::string_utils::expand_to_visual_width;
use crate::term::TERM_SIZE;
use crate::transforms::Transform;
use console::{measure_text_width, strip_ansi_codes};
use once_cell::sync::Lazy;
use regex::Regex;
use std::sync::{Arc, Mutex};

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
    ///
    /// # Note
    /// When using wide characters (like emojis) for padding, the final width might be slightly
    /// less than the target width due to quantization. This happens because:
    /// 1. Wide characters must be displayed as complete units (can't be split)
    /// 2. The terminal width is fixed, but our padding units are "quantized" by the width of the characters
    /// 3. When there's a remainder that's less than the width of a character, we must round down
    ///    to avoid exceeding the target width
    ///
    /// For example, with a target width of 164 and emojis (2 units wide):
    /// - If we need 127 units of padding for 2 groups
    /// - First group gets 64 units (32 emojis)
    /// - Second group gets 63 units (31 emojis + 1 unit remainder)
    /// - The 1 unit remainder can't be filled without exceeding the target width
    fn process_padding_line(&self, content: &str) -> String {
        let (groups, text_without_pads) = self.extract_padding_groups(content);
        let mut result = content.to_string();

        if !groups.is_empty() {
            // Filter out empty padding groups
            let non_empty_groups: Vec<_> = groups
                .iter()
                .filter(|g| !strip_ansi_codes(&g.content).is_empty())
                .collect();

            if !non_empty_groups.is_empty() {
                // Calculate total padding needed and remainder
                let max_width = self.get_width.lock().unwrap()();
                let total_padding_needed = max_width.saturating_sub(text_without_pads);
                let base_padding = total_padding_needed / non_empty_groups.len();
                let remainder = total_padding_needed % non_empty_groups.len();

                // Process all groups in reverse order to maintain correct indices
                for (i, group) in groups.iter().rev().enumerate() {
                    if strip_ansi_codes(&group.content).is_empty() {
                        // Remove empty padding groups
                        result.replace_range(group.start..group.end, "");
                    } else {
                        // Calculate padding for non-empty groups
                        let padding_width = if i == 0 {
                            base_padding + remainder
                        } else {
                            base_padding
                        };

                        // Expand the stripped content
                        let stripped_content = strip_ansi_codes(&group.content);
                        let expanded_content =
                            expand_to_visual_width(&stripped_content, padding_width);

                        // Find the actual content position in the original string
                        let content_start = group
                            .content
                            .find(&stripped_content.to_string())
                            .unwrap_or(0);
                        let content_end = content_start + stripped_content.len();

                        // Extract ANSI codes before and after the content
                        let prefix = &group.content[..content_start];
                        let suffix = &group.content[content_end..];

                        // Combine the ANSI codes with the expanded content
                        let final_content = format!("{}{}{}", prefix, expanded_content, suffix);

                        // Replace the entire pad() structure with the expanded content
                        result.replace_range(group.start..group.end, &final_content);
                    }
                }
            }
        }

        result
    }

    /// Extract padding groups from the content and calculate their information
    ///
    /// # Arguments
    /// * `content` - The content to process
    ///
    /// # Returns
    /// A tuple containing:
    /// - Vector of padding group information
    /// - Total occupied space (outside text + padding content)
    fn extract_padding_groups(&self, content: &str) -> (Vec<MatchedGroup>, usize) {
        let stripped_content = strip_ansi_codes(content);
        let stripped_width = measure_text_width(&stripped_content);

        let (groups, total_group_length) = PAD_PATTERN
            .captures_iter(content) // Use original content for matching
            .filter_map(|cap| {
                cap.get(0).and_then(|matched| {
                    let pad_content = cap.get(1).map_or("", |m| m.as_str()).to_string();

                    // Get the stripped version of the matched group for width calculation
                    let stripped_group = strip_ansi_codes(&content[matched.start()..matched.end()]);
                    let group_length = measure_text_width(&stripped_group);

                    Some((
                        MatchedGroup {
                            content: pad_content,
                            start: matched.start(),
                            end: matched.end(),
                        },
                        group_length,
                    ))
                })
            })
            .fold(
                (Vec::new(), 0),
                |(mut groups, total_group_length), (group, group_len)| {
                    groups.push(group);
                    (groups, total_group_length + group_len)
                },
            );

        // Calculate the width of the text without padding groups
        let text_without_pads = stripped_width - total_group_length;

        (groups, text_without_pads)
    }
}

impl Transform for TextProcessor {
    fn transform(&self, text: &str) -> Result<String> {
        let ctx = ContextManager::get().read()?;
        // Check if context has a width parameter
        if let Some(width) = ctx.get("width").and_then(|w| w.parse::<u8>().ok()) {
            *self.get_width.lock().unwrap() = Box::new(move || {
                let term_width = Self::default_width()();
                (term_width * width as usize) / 100
            });
        }
        Ok(self.process_padding(text))
    }
}
