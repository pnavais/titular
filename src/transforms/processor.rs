use crate::prelude::*;
use crate::string_utils::expand_to_visual_width;
use crate::term::TERM_SIZE;
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

// Regex to match content between our non-visible markers
static PAD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        r"{start}(.*?){end}",
        start = regex::escape(&padding::START.to_string()),
        end = regex::escape(&padding::END.to_string())
    ))
    .unwrap()
});

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
    /// use titular::constants::padding;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test empty pad removal
    /// let input = format!("Hello {}{} World", padding::START, padding::END);
    /// assert_eq!(processor.transform(&input).unwrap(), "Hello  World");
    ///
    /// // Test padding expansion
    /// let input = format!("Hello {}→{} World", padding::START, padding::END);
    /// let result = processor.transform(&input).unwrap();
    /// assert!(result.starts_with("Hello →"));
    /// assert!(result.ends_with("World"));
    /// assert!(result.len() > "Hello → World".len());
    /// ```
    fn process_padding_line(&self, content: &str) -> String {
        let mut result = content.to_string();

        // First remove all empty padding groups from the string
        self.remove_empty_pads(&mut result);

        // Extract and process padding groups
        let (groups, text_without_pads) = self.extract_padding_groups(&result);
        if !groups.is_empty() {
            self.process_padding_groups(&mut result, groups, text_without_pads);
        }

        result
    }

    /// Removes all empty padding groups from the given string.
    /// This method modifies the string in place by removing any padding groups
    /// that have no content between the markers.
    fn remove_empty_pads(&self, result: &mut String) {
        // Use regex to find all padding groups
        let pattern = format!(
            r"{}(.*?){}",
            regex::escape(&padding::START.to_string()),
            regex::escape(&padding::END.to_string())
        );
        let re = Regex::new(&pattern).unwrap();

        // First collect all matches that need to be removed
        let to_remove: Vec<(usize, usize)> = re
            .captures_iter(&result)
            .filter_map(|cap| {
                let matched = cap.get(0)?;
                let content = cap.get(1)?;

                // If the content is empty after stripping ANSI codes, mark for removal
                if strip_ansi_codes(content.as_str()).is_empty() {
                    Some((matched.start(), matched.end()))
                } else {
                    None
                }
            })
            .collect();

        // Then remove them in reverse order to maintain correct indices
        for (start, end) in to_remove.into_iter().rev() {
            result.replace_range(start..end, "");
        }
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

                    // Include all groups, empty or not
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
                    // Add all groups to the list
                    groups.push(group);
                    (groups, total_group_length + group_len)
                },
            );

        // Calculate the width of the text without padding groups
        let text_without_pads = stripped_width - total_group_length;

        (groups, text_without_pads)
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
    /// use titular::constants::padding;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test padding distribution
    /// let input = format!("Hello {}→{} {}←{} World",
    ///     padding::START, padding::END, padding::START, padding::END);
    /// let result = processor.transform(&input).unwrap();
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
    /// use titular::constants::padding;
    /// use std::sync::Mutex;
    /// use std::sync::Arc;
    ///
    /// // Create a processor with a fixed width for testing
    /// let processor = TextProcessor::new(Box::new(|| 20));
    ///
    /// // Test ANSI code preservation
    /// let input = format!("{}\x1b[31m→\x1b[0m{}", padding::START, padding::END);
    /// let result = processor.transform(&input).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_padding_groups_basic() {
        let processor = TextProcessor::default();
        let input = format!(
            "{}hello{} world {}foo{}",
            padding::START,
            padding::END,
            padding::START,
            padding::END
        );
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].content, "hello");
        assert_eq!(groups[1].content, "foo");
        assert_eq!(text_width, measure_text_width(" world "));
    }

    #[test]
    fn test_extract_padding_groups_empty() {
        let processor = TextProcessor::default();
        let input = format!("{}{}", padding::START, padding::END);
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(
            groups.len(),
            1,
            "extract_padding_groups should extract all groups, including empty ones"
        );
        assert_eq!(
            groups[0].content, "",
            "empty group's content should be an empty string"
        );
        assert_eq!(
            text_width, 0,
            "text_width (outside text) should be zero (no text outside the group)"
        );
    }

    #[test]
    fn test_extract_padding_groups_with_ansi() {
        let processor = TextProcessor::default();
        let input = format!(
            "{}\x1b[31mhello\x1b[0m{} \x1b[32mworld\x1b[0m {}foo{}",
            padding::START,
            padding::END,
            padding::START,
            padding::END
        );
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].content, "\x1b[31mhello\x1b[0m");
        assert_eq!(groups[1].content, "foo");
        assert_eq!(text_width, measure_text_width(" world "));
    }

    #[test]
    fn test_extract_padding_groups_with_emoji() {
        let processor = TextProcessor::default();
        let input = format!(
            "{}hello 🦀{} world {}foo{}",
            padding::START,
            padding::END,
            padding::START,
            padding::END
        );
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].content, "hello 🦀");
        assert_eq!(groups[1].content, "foo");
        assert_eq!(text_width, measure_text_width(" world "));
    }

    #[test]
    fn test_extract_padding_groups_with_ansi_and_emoji() {
        let processor = TextProcessor::default();
        let input = format!(
            "{}\x1b[31mhello 🦀\x1b[0m{} \x1b[32mworld\x1b[0m {}foo{}",
            padding::START,
            padding::END,
            padding::START,
            padding::END
        );
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].content, "\x1b[31mhello 🦀\x1b[0m");
        assert_eq!(groups[1].content, "foo");
        assert_eq!(text_width, measure_text_width(" world "));
    }

    #[test]
    fn test_extract_padding_groups_no_markers() {
        let processor = TextProcessor::default();
        let input = "hello world";
        let (groups, text_width) = processor.extract_padding_groups(input);
        assert_eq!(groups.len(), 0);
        assert_eq!(text_width, measure_text_width(input));
    }

    #[test]
    fn test_extract_padding_groups_unmatched_start() {
        let processor = TextProcessor::default();
        let input = format!("{}hello world", padding::START);
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(groups.len(), 0);
        assert_eq!(text_width, measure_text_width(&input));
    }

    #[test]
    fn test_extract_padding_groups_unmatched_end() {
        let processor = TextProcessor::default();
        let input = format!("hello world{}", padding::END);
        let (groups, text_width) = processor.extract_padding_groups(&input);
        assert_eq!(groups.len(), 0);
        assert_eq!(text_width, measure_text_width(&input));
    }

    #[test]
    fn test_process_padding_line() {
        let processor = TextProcessor::new(Box::new(|| 20));
        let input = format!("Hello {}→{} World", padding::START, padding::END);
        let result = processor.process_padding_line(&input);
        assert!(result.starts_with("Hello →"));
        assert!(result.ends_with("World"));
        assert!(result.len() > "Hello → World".len());
    }

    #[test]
    fn test_process_padding_line_with_ansi() {
        let processor = TextProcessor::new(Box::new(|| 20));
        let input = format!(
            "Hello {}\x1b[31m→\x1b[0m{} World",
            padding::START,
            padding::END
        );
        let result = processor.process_padding_line(&input);
        assert!(result.starts_with("Hello \x1b[31m→"));
        assert!(result.ends_with("\x1b[0m World"));
        assert!(result.len() > "Hello \x1b[31m→\x1b[0m World".len());
    }

    #[test]
    fn test_remove_empty_pads() {
        let processor = TextProcessor::default();
        let mut input = format!(
            "Hello {}{} World {}{} Test",
            padding::START,
            padding::END,
            padding::START,
            padding::END
        );
        processor.remove_empty_pads(&mut input);
        assert_eq!(input, "Hello  World  Test");

        // Test with non-empty padding
        let mut input = format!(
            "Hello {}content{} World {}{} Test",
            padding::START,
            padding::END,
            padding::START,
            padding::END
        );
        processor.remove_empty_pads(&mut input);
        assert_eq!(
            input,
            format!(
                "Hello {}content{} World  Test",
                padding::START,
                padding::END
            )
        );
    }
}
