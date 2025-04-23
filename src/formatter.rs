use crate::config::TemplateConfig;
use crate::error::*;
use crate::fallback_map::FallbackMap;
use regex::Regex;

#[derive(Debug)]
pub enum VarStyle {
    Pad,
    Fit,
}

#[derive(Debug)]
pub struct VarFormat {
    pub value: String,
    pub suffix: Option<String>,
    pub bg_color: Option<String>,
    pub fg_color: Option<String>,
    pub style: Option<VarStyle>,
}

pub struct TemplateFormatter<'a> {
    context: FallbackMap<'a, str, String>,
}

impl<'a> TemplateFormatter<'a> {
    pub fn new(context: FallbackMap<'a, str, String>) -> Self {
        Self { context }
    }

    /// Formats a template string using the provided context.
    ///
    /// # Arguments
    /// * `template_content` - The template configuration containing the pattern to format
    ///
    /// # Returns
    /// A formatted string
    pub fn format(&self, template_content: &TemplateConfig) -> Result<String> {
        let formatted = template_content.pattern.data.clone();
        let groups = self.extract_groups(&formatted);

        // For now, just return the groups for testing
        println!("Found groups: {:?}", groups);

        Ok(formatted)
    }

    /// Extracts and parses all groups from a template string.
    /// A group is defined as content between ${ and }.
    /// The content can include variable name, style, and color information.
    ///
    /// # Arguments
    /// * `input` - The input string to process
    ///
    /// # Returns
    /// A vector of VarFormat structs containing the parsed information
    fn extract_groups(&self, input: &str) -> Vec<VarFormat> {
        let re = Regex::new(r"\$\{([^}]+)\}").unwrap();
        re.captures_iter(input)
            .filter_map(|cap| cap.get(1).map(|m| self.parse_var_expression(m.as_str())))
            .collect()
    }

    /// Parses a single variable expression into a VarFormat struct.
    /// Supports the following patterns:
    /// - var_name:expression_list
    /// - var_name+[additional_string]
    /// - var_name:expression_list+[additional_string]
    /// - var_name+[additional_string]:expression_list
    ///
    /// Where expression_list can contain:
    /// - pad or fit (style)
    /// - fg[color_name] (foreground color)
    /// - bg[color_name] (background color)
    ///
    /// # Arguments
    /// * `content` - The content to parse
    /// * `context` - The context containing variable values
    ///
    /// # Returns
    /// A VarFormat struct containing the parsed information
    fn parse_var_expression(&self, content: &str) -> VarFormat {
        // First, split the content into base part and additional string if present
        let (base, additional) = if let Some(idx) = content.find('+') {
            let (base, rest) = content.split_at(idx);
            // Find the end of the additional string (either at the next ':' or end of string)
            let additional_end = rest.find(':').unwrap_or(rest.len());
            let additional = rest[1..additional_end].trim_matches(|c| c == '[' || c == ']');
            let remaining = if additional_end < rest.len() {
                &rest[additional_end..]
            } else {
                ""
            };
            (base.to_string() + remaining, Some(additional))
        } else {
            (content.to_string(), None)
        };

        // Split the base part into variable name and expressions
        let (var_name, expressions) = if let Some(idx) = base.find(':') {
            let (name, exprs) = base.split_at(idx);
            (name, Some(exprs.trim_start_matches(':')))
        } else {
            (base.as_str(), None)
        };

        // Get the base value from context
        let value = self
            .context
            .get(var_name)
            .unwrap_or(&String::new())
            .to_string();

        // Parse expressions
        let mut style = None;
        let mut fg_color = None;
        let mut bg_color = None;

        if let Some(exprs) = expressions {
            for expr in exprs.split(':') {
                match expr.trim() {
                    "pad" => style = Some(VarStyle::Pad),
                    "fit" => style = Some(VarStyle::Fit),
                    fg if fg.starts_with("fg[") && fg.ends_with(']') => {
                        let color_name = &fg[3..fg.len() - 1];
                        // If color name starts with $, resolve from context, otherwise use as is
                        fg_color = if color_name.starts_with('$') {
                            self.context.get(&color_name[1..]).map(String::from)
                        } else {
                            Some(color_name.to_string())
                        };
                    }
                    bg if bg.starts_with("bg[") && bg.ends_with(']') => {
                        let color_name = &bg[3..bg.len() - 1];
                        // If color name starts with $, resolve from context, otherwise use as is
                        bg_color = if color_name.starts_with('$') {
                            self.context.get(&color_name[1..]).map(String::from)
                        } else {
                            Some(color_name.to_string())
                        };
                    }
                    _ => {}
                }
            }
        }

        VarFormat {
            value,
            suffix: additional.map(String::from),
            bg_color,
            fg_color,
            style,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fallback_map::{FallbackMap, MapProvider};
    use once_cell::sync::Lazy;
    use std::collections::HashMap;

    struct TestContext {
        vars: HashMap<String, String>,
    }

    impl MapProvider<str, String> for TestContext {
        fn contains(&self, key: &str) -> bool {
            self.vars.contains_key(key)
        }

        fn resolve(&self, key: &str) -> Option<&String> {
            self.vars.get(key)
        }

        fn is_active(&self, _key: &str) -> bool {
            true
        }
    }

    static TEST_CONTEXT: Lazy<TestContext> = Lazy::new(|| {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "John".to_string());
        vars.insert("time".to_string(), "12:00".to_string());
        vars.insert("date".to_string(), "2024-03-20".to_string());
        vars.insert("my_fg_color".to_string(), "red".to_string());
        vars.insert("my_bg_color".to_string(), "blue".to_string());
        TestContext { vars }
    });

    static TEST_FORMATTER: Lazy<TemplateFormatter> =
        Lazy::new(|| TemplateFormatter::new(FallbackMap::from(&*TEST_CONTEXT)));

    #[test]
    fn test_basic_variable_parsing() {
        let result = TEST_FORMATTER.parse_var_expression("name");

        assert_eq!(result.value, "John");
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_variable_with_style() {
        let result = TEST_FORMATTER.parse_var_expression("name:pad");

        assert_eq!(result.value, "John");
        assert!(result.suffix.is_none());
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_variable_with_literal_colors() {
        let result = TEST_FORMATTER.parse_var_expression("name:fg[red]:bg[blue]");

        assert_eq!(result.value, "John");
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert_eq!(result.bg_color, Some("blue".to_string()));
    }

    #[test]
    fn test_variable_with_context_colors() {
        let result = TEST_FORMATTER.parse_var_expression("name:fg[$my_fg_color]:bg[$my_bg_color]");

        assert_eq!(result.value, "John");
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert_eq!(result.bg_color, Some("blue".to_string()));
    }

    #[test]
    fn test_variable_with_additional_string() {
        let result = TEST_FORMATTER.parse_var_expression("name+[!]");

        assert_eq!(result.value, "John");
        assert_eq!(result.suffix, Some("!".to_string()));
        assert!(result.style.is_none());
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_variable_with_all_options() {
        let result =
            TEST_FORMATTER.parse_var_expression("name:pad:fg[$my_fg_color]:bg[$my_bg_color]+[!]");

        assert_eq!(result.value, "John");
        assert_eq!(result.suffix, Some("!".to_string()));
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert_eq!(result.bg_color, Some("blue".to_string()));
    }

    #[test]
    fn test_variable_with_additional_string_before_expressions() {
        let result = TEST_FORMATTER.parse_var_expression("name+[!]:pad:fg[$my_fg_color]");

        assert_eq!(result.value, "John");
        assert_eq!(result.suffix, Some("!".to_string()));
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_multiple_variables_in_string() {
        let input = "Hello ${name:pad:fg[$my_fg_color]}! The time is ${time:fit:bg[$my_bg_color]} and the date is ${date+[!]}";
        let groups = TEST_FORMATTER.extract_groups(input);

        assert_eq!(groups.len(), 3);

        // Check first variable
        assert_eq!(groups[0].value, "John");
        assert!(groups[0].suffix.is_none());
        assert!(matches!(groups[0].style, Some(VarStyle::Pad)));
        assert_eq!(groups[0].fg_color, Some("red".to_string()));
        assert!(groups[0].bg_color.is_none());

        // Check second variable
        assert_eq!(groups[1].value, "12:00");
        assert!(groups[1].suffix.is_none());
        assert!(matches!(groups[1].style, Some(VarStyle::Fit)));
        assert!(groups[1].fg_color.is_none());
        assert_eq!(groups[1].bg_color, Some("blue".to_string()));

        // Check third variable
        assert_eq!(groups[2].value, "2024-03-20");
        assert_eq!(groups[2].suffix, Some("!".to_string()));
        assert!(groups[2].style.is_none());
        assert!(groups[2].fg_color.is_none());
        assert!(groups[2].bg_color.is_none());
    }

    #[test]
    fn test_unknown_variable() {
        let result = TEST_FORMATTER.parse_var_expression("unknown:pad");

        assert_eq!(result.value, "");
        assert!(result.suffix.is_none());
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_unknown_color_variable() {
        let result = TEST_FORMATTER.parse_var_expression("name:fg[$unknown_color]");

        assert_eq!(result.value, "John");
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }
}
