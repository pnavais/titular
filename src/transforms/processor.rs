use crate::context_manager::ContextManager;
use crate::error::Result;
use crate::string_utils::{expand_to_width, AnsiTruncateBehavior, Truncate};
use crate::term::TERM_SIZE;
use crate::transforms::Transform;
use console::{measure_text_width, strip_ansi_codes};
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
        let (groups, text_without_pads, avg_padding) = self.extract_padding_groups(content);
        content.to_string()
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
    /// - Number of non-empty padding groups
    fn extract_padding_groups(&self, content: &str) -> (Vec<MatchedGroup>, usize, usize) {
        let stripped_content = strip_ansi_codes(content);
        let (groups, total_group_length, total_content_length, non_empty_count) = PAD_PATTERN
            .captures_iter(&stripped_content)
            .filter_map(|cap| {
                cap.get(0).and_then(|matched| {
                    let pad_content = cap.get(1).map_or("", |m| m.as_str()).to_string();
                    let content_width = measure_text_width(&strip_ansi_codes(&pad_content));
                    let group_length =
                        measure_text_width(&stripped_content[matched.start()..matched.end()]);
                    let is_non_empty = !pad_content.is_empty();

                    Some((
                        MatchedGroup {
                            content: pad_content,
                            start: matched.start(),
                            end: matched.end(),
                        },
                        group_length,
                        content_width,
                        if is_non_empty { 1 } else { 0 },
                    ))
                })
            })
            .fold(
                (Vec::new(), 0, 0, 0),
                |(mut groups, total_group_length, total_content_length, non_empty),
                 (group, group_len, content_len, is_non_empty)| {
                    groups.push(group);
                    (
                        groups,
                        total_group_length + group_len,
                        total_content_length + content_len,
                        non_empty + is_non_empty,
                    )
                },
            );

        let text_without_pads =
            measure_text_width(&stripped_content) - total_group_length + total_content_length;
        let avg_padding = if non_empty_count > 0 {
            let max_width = self.get_width.lock().unwrap()();
            ((max_width - text_without_pads) as f64 / non_empty_count as f64).ceil() as usize
        } else {
            0
        };

        (groups, text_without_pads, avg_padding)
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
