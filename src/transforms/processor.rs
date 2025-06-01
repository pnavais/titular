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

// Regex to match pad() calls, including nested ones and empty ones
static PAD_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"pad\((?:((?:[^()]|\([^()]*\))*))?\)").unwrap());

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

    /// Process a single line of content with padding and line wrapping.
    /// This is the main entry point for processing a line that may contain pad() calls.
    ///
    /// # Arguments
    /// * `content` - The content to process, which may contain pad() calls
    ///
    /// # Returns
    /// A string with all pad() calls processed:
    /// - Empty pad() calls are removed
    /// - Non-empty pad() calls are expanded to fill available space
    ///
    /// # Examples
    /// ```
    /// use titular::transforms::Transform;
    /// use titular::transforms::TextProcessor;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test empty pad removal
    /// assert_eq!(processor.transform("Hello pad() World").unwrap(), "Hello  World");
    ///
    /// // Test padding expansion
    /// let result = processor.transform("Hello pad(→) World").unwrap();
    /// assert!(result.starts_with("Hello →"));
    /// assert!(result.ends_with("World"));
    /// assert!(result.len() > "Hello → World".len());
    /// ```
    fn process_padding_line(&self, content: &str) -> String {
        let mut result = content.to_string();

        // First remove all empty pad() calls
        self.remove_empty_pads(&mut result);

        // Then process any remaining padding groups
        let (groups, text_without_pads) = self.extract_padding_groups(&result);
        if !groups.is_empty() {
            self.process_padding_groups(&mut result, groups, text_without_pads);
        }

        result
    }

    /// Removes all empty pad() calls from the given string.
    /// This method modifies the string in place by removing any pad() calls
    /// that have no content between the parentheses.
    ///
    /// # Arguments
    /// * `result` - A mutable reference to the string to process. The string will be
    ///             modified in place to remove all empty pad() calls.
    ///
    /// # Examples
    /// ```
    /// use titular::transforms::Transform;
    /// use titular::transforms::TextProcessor;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test empty pad removal
    /// assert_eq!(processor.transform("Hello pad() World").unwrap(), "Hello  World");
    /// assert_eq!(processor.transform("pad() Hello pad() World").unwrap(), " Hello  World");
    ///
    /// // Test that non-empty pads are preserved
    /// let result = processor.transform("Hello pad(→) pad() World").unwrap();
    /// assert!(result.starts_with("Hello →"));
    /// assert!(result.ends_with("World"));
    /// ```
    fn remove_empty_pads(&self, result: &mut String) {
        PAD_PATTERN
            .captures_iter(result)
            .filter_map(|cap| {
                cap.get(0).and_then(|matched| {
                    // If the content group is None or empty, this is an empty pad()
                    if cap.get(1).map_or(true, |m| m.as_str().is_empty()) {
                        Some((matched.start(), matched.end()))
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>() // Collect to avoid borrow issues
            .into_iter()
            .rev() // Process in reverse to maintain correct indices
            .for_each(|(start, end)| {
                result.replace_range(start..end, "");
            });
    }

    /// Process non-empty padding groups in the given string.
    /// This method handles the expansion of pad() calls that contain content,
    /// distributing the available space among all groups.
    ///
    /// # Arguments
    /// * `result` - A mutable reference to the string to process. The string will be
    ///             modified in place to expand pad() calls.
    /// * `groups` - A vector of matched padding groups found in the string
    /// * `text_without_pads` - The width of the text excluding padding groups,
    ///                        used to calculate available space for padding
    ///
    /// # Note
    /// The available space is distributed evenly among all padding groups,
    /// with any remainder being added to the first group. This ensures that
    /// the total width of the line matches the target width while maintaining
    /// proportional padding.
    ///
    /// # Examples
    /// ```
    /// use titular::transforms::Transform;
    /// use titular::transforms::TextProcessor;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test padding distribution
    /// let result = processor.transform("Hello pad(→) pad(←) World").unwrap();
    /// assert!(result.starts_with("Hello →"));
    /// assert!(result.contains("←"));
    /// assert!(result.ends_with("World"));
    /// assert!(result.len() > "Hello → ← World".len());
    /// ```
    fn process_padding_groups(
        &self,
        result: &mut String,
        groups: Vec<MatchedGroup>,
        text_without_pads: usize,
    ) {
        // Filter out empty padding groups
        let non_empty_groups: Vec<_> = groups
            .iter()
            .filter(|g| !strip_ansi_codes(&g.content).is_empty())
            .collect();

        if non_empty_groups.is_empty() {
            return;
        }

        // Calculate total padding needed and remainder
        let max_width = self.get_width.lock().unwrap()();
        let total_padding_needed = max_width.saturating_sub(text_without_pads);
        let base_padding = total_padding_needed / non_empty_groups.len();
        let remainder = total_padding_needed % non_empty_groups.len();

        // Process all groups in reverse order to maintain correct indices
        for (i, group) in non_empty_groups.iter().rev().enumerate() {
            self.expand_padding_group(
                result,
                group,
                if i == 0 {
                    base_padding + remainder
                } else {
                    base_padding
                },
            );
        }
    }

    /// Expands a single padding group with the given width.
    /// This method handles the actual expansion of a pad() call's content,
    /// preserving any ANSI codes while expanding the content to fill the
    /// specified width.
    ///
    /// # Arguments
    /// * `result` - A mutable reference to the string containing the pad() call
    /// * `group` - The padding group to expand, containing the content and its
    ///            position in the string
    /// * `padding_width` - The target width to expand the content to
    ///
    /// # Note
    /// The method preserves any ANSI codes in the content by:
    /// 1. Finding the actual content position within the pad() call
    /// 2. Preserving any ANSI codes before and after the content
    /// 3. Expanding only the content part while maintaining the codes
    ///
    /// # Examples
    /// ```
    /// use titular::transforms::Transform;
    /// use titular::transforms::TextProcessor;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test ANSI code preservation
    /// let result = processor.transform("pad(\x1b[31m→\x1b[0m)").unwrap();
    /// assert!(result.starts_with("\x1b[31m"));
    /// assert!(result.ends_with("\x1b[0m"));
    /// assert!(result.len() > "\x1b[31m→\x1b[0m".len());
    /// ```
    fn expand_padding_group(
        &self,
        result: &mut String,
        group: &MatchedGroup,
        padding_width: usize,
    ) {
        // Expand the stripped content
        let stripped_content = strip_ansi_codes(&group.content);
        let expanded_content = expand_to_visual_width(&stripped_content, padding_width);

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
                    // For empty pad(), content will be None
                    let pad_content = cap.get(1).map_or("", |m| m.as_str()).to_string();

                    // Get the stripped version of the matched group for width calculation
                    let stripped_group = strip_ansi_codes(&content[matched.start()..matched.end()]);
                    let group_length = measure_text_width(&stripped_group);

                    // Only include non-empty groups
                    if !pad_content.is_empty() {
                        Some((
                            MatchedGroup {
                                content: pad_content,
                                start: matched.start(),
                                end: matched.end(),
                            },
                            group_length,
                        ))
                    } else {
                        // For empty pad(), just return the length for total calculation
                        Some((
                            MatchedGroup {
                                content: String::new(),
                                start: matched.start(),
                                end: matched.end(),
                            },
                            0, // Empty pad() contributes 0 to the total length
                        ))
                    }
                })
            })
            .fold(
                (Vec::new(), 0),
                |(mut groups, total_group_length), (group, group_len)| {
                    // Only add non-empty groups to the list
                    if !group.content.is_empty() {
                        groups.push(group);
                    }
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
