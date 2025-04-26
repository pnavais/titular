use crate::color_manager::ColorManager;
use crate::config::{Defaults, TemplateConfig};
use crate::error::*;
use crate::fallback_map::FallbackMap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt;

static VAR_GROUP_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{([^}]+)\}").unwrap());
static EXPRESSION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(pad|fit|fg\[([^]]+)\]|bg\[([^]]+)\])").unwrap());
static VAR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([^:]+)(?::([^+%]+))?(?:([+%])\[([^]]+)\])?$").unwrap());

#[derive(Debug)]
pub enum VarStyle {
    Pad,
    Fit,
}

#[derive(Debug, Clone)]
pub enum SuffixType {
    Join,     // For + operator: applies the style to the value and the suffix
    Separate, // For % operator: applies the style only to the value
}

impl From<char> for SuffixType {
    fn from(c: char) -> Self {
        match c {
            '+' => SuffixType::Join,
            '%' => SuffixType::Separate,
            _ => panic!("Invalid suffix operator: {}", c),
        }
    }
}

#[derive(Debug)]
pub struct VarFormat {
    pub value: Option<String>,
    pub suffix: Option<String>,
    pub suffix_type: Option<SuffixType>,
    pub bg_color: Option<String>,
    pub fg_color: Option<String>,
    pub style: Option<VarStyle>,
}

pub trait VarFormatter: fmt::Debug {
    /// Formats a variable using the provided context.
    ///
    /// This implementation handles three cases:
    /// 1. Join type (+): Combines value and suffix before styling
    /// 2. Separate type (%): Keeps value and suffix separate for styling
    /// 3. No suffix type: Uses only the value
    ///
    /// The base string is then passed to apply_style for color formatting.
    fn format(&self, context: &FallbackMap<str, String>) -> String;

    /// Applies styling to a base string using the provided context.
    ///
    /// This implementation follows a specific order of operations:
    /// 1. Applies background color if present
    /// 2. Applies foreground color if present
    /// 3. Handles suffix based on suffix type:
    ///    - For Separate type: Appends suffix after styling
    ///    - For Join type: Suffix is already part of the base string
    ///
    /// The method uses Option chaining to handle each styling step,
    /// falling back to the previous result if a style is not present.
    fn apply_style(&self, context: &FallbackMap<str, String>, base: &str) -> String;
}

impl VarFormatter for VarFormat {
    /// Formats a variable using the provided context.
    ///
    /// This implementation handles three cases:
    /// 1. Join type (+): Combines value and suffix before styling
    /// 2. Separate type (%): Keeps value and suffix separate for styling
    /// 3. No suffix type: Uses only the value
    ///
    /// The base string is then passed to apply_style for color formatting.
    fn format(&self, context: &FallbackMap<str, String>) -> String {
        let base = if let Some(value) = &self.value {
            match &self.suffix_type {
                Some(SuffixType::Join) => {
                    format!("{}{}", value, self.suffix.as_deref().unwrap_or(""))
                }
                Some(SuffixType::Separate) => value.to_string(),
                None => value.to_string(),
            }
        } else {
            String::new()
        };

        self.apply_style(context, &base)
    }

    /// Applies styling to a base string using the provided context.
    ///
    /// This implementation follows a specific order of operations:
    /// 1. Applies background color if present
    /// 2. Applies foreground color if present
    /// 3. Handles suffix based on suffix type:
    ///    - For Separate type: Appends suffix after styling
    ///    - For Join type: Suffix is already part of the base string
    ///
    /// The method uses Option chaining to handle each styling step,
    /// falling back to the previous result if a style is not present.
    fn apply_style(&self, context: &FallbackMap<str, String>, base: &str) -> String {
        self.bg_color
            .as_ref()
            .map(|bg| ColorManager::format(context, base, bg, true))
            .or_else(|| Some(base.to_string()))
            .map(|with_bg| {
                self.fg_color
                    .as_ref()
                    .map(|fg| ColorManager::format(context, &with_bg, fg, false))
                    .unwrap_or(with_bg)
            })
            .map(|with_fg| match &self.suffix_type {
                Some(SuffixType::Separate) => {
                    format!("{}{}", with_fg, self.suffix.as_deref().unwrap_or(""))
                }
                _ => with_fg,
            })
            .unwrap_or_else(|| base.to_string())
    }
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
        let groups = self.extract_groups(&template_content.pattern.data);

        println!("Groups: {:?}", groups);

        Ok(groups
            .into_iter()
            .map(|group| group.format(&self.context))
            .collect::<Vec<String>>()
            .join(""))
    }

    /// Extracts and parses all groups from a template string.
    /// A group is defined as content between ${ and }.
    /// The content can include variable name, style, and color information.
    ///
    /// # Arguments
    /// * `input` - The input string to process
    ///
    /// # Returns
    /// A vector of VarFormatter implementations containing the parsed information
    fn extract_groups(&self, input: &str) -> Vec<Box<dyn VarFormatter>> {
        VAR_GROUP_REGEX
            .captures_iter(input)
            .filter_map(|cap| cap.get(1).map(|m| self.parse_var_expression(m.as_str())))
            .map(|format| Box::new(format) as Box<dyn VarFormatter>)
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
        // Parse the content using regex
        let captures = VAR_REGEX.captures(content).unwrap();
        let var_name = captures.get(1).unwrap().as_str();
        let style_expr = captures.get(2).map(|m| m.as_str());
        let suffix_info = captures.get(3).map(|op| {
            let suffix_type = SuffixType::from(op.as_str().chars().next().unwrap());
            let suffix = captures
                .get(4)
                .map(|m| {
                    let captured = m.as_str();
                    if captured.starts_with('$') {
                        self.context.get(&captured[1..]).map(String::from)
                    } else {
                        Some(captured.to_string())
                    }
                })
                .flatten()
                .unwrap_or_default();
            (suffix_type, suffix)
        });

        // Get the base value from context
        let value = Some(self.process_fillers(var_name));
        // Parse style expression
        let (style, fg_color, bg_color) = self.parse_style(style_expr);

        VarFormat {
            value,
            suffix: suffix_info.as_ref().map(|(_, s)| s.to_string()),
            suffix_type: suffix_info.map(|(t, _)| t),
            bg_color,
            fg_color,
            style,
        }
    }

    /// Processes special filler variables.
    /// For variable named "f", it follows this priority:
    /// 1. Value from context if exists
    /// 2. Value from "defaults.fill_char" if exists
    /// 3. Default value from Defaults struct
    ///
    /// # Arguments
    /// * `var_name` - The name of the variable to process
    ///
    /// # Returns
    /// The resolved value for the variable
    fn process_fillers(&self, var_name: &str) -> String {
        if var_name == "f" {
            self.context
                .get(var_name)
                .or_else(|| self.context.get("defaults.fill_char"))
                .unwrap_or(&Defaults::default().fill_char)
                .to_string()
        } else {
            self.context
                .get(var_name)
                .unwrap_or(&String::new())
                .to_string()
        }
    }

    /// Parses the style from the expression variable.
    ///
    /// # Arguments
    /// * `expressions` - The expression variable to parse
    ///
    /// # Returns
    /// A tuple containing the style, foreground color, and background color
    /// If no style is found, it returns None for the style.
    fn parse_style(
        &self,
        expressions: Option<&str>,
    ) -> (Option<VarStyle>, Option<String>, Option<String>) {
        let mut style = None;
        let mut fg_color = None;
        let mut bg_color = None;

        if let Some(exprs) = expressions {
            for cap in EXPRESSION_REGEX.captures_iter(exprs) {
                match cap.get(1).unwrap().as_str() {
                    "pad" => style = Some(VarStyle::Pad),
                    "fit" => style = Some(VarStyle::Fit),
                    color if color.starts_with("fg[") || color.starts_with("bg[") => {
                        let is_fg = color.starts_with("fg[");
                        let color_name = cap.get(if is_fg { 2 } else { 3 }).unwrap().as_str();
                        let resolved_color = if color_name.starts_with('$') {
                            self.context.get(&color_name[1..]).map(String::from)
                        } else {
                            Some(color_name.to_string())
                        };
                        if is_fg {
                            fg_color = resolved_color;
                        } else {
                            bg_color = resolved_color;
                        }
                    }
                    _ => {}
                }
            }
        }

        (style, fg_color, bg_color)
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
        vars.insert("f".to_string(), "#".to_string());
        vars.insert("defaults.fill_char".to_string(), "-".to_string());
        TestContext { vars }
    });

    static TEST_FORMATTER: Lazy<TemplateFormatter> =
        Lazy::new(|| TemplateFormatter::new(FallbackMap::from(&*TEST_CONTEXT)));

    #[test]
    fn test_basic_variable_parsing() {
        let result = TEST_FORMATTER.parse_var_expression("name");

        assert_eq!(result.value, Some("John".to_string()));
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_variable_with_style() {
        let result = TEST_FORMATTER.parse_var_expression("name:pad");

        assert_eq!(result.value, Some("John".to_string()));
        assert!(result.suffix.is_none());
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_variable_with_literal_colors() {
        let result = TEST_FORMATTER.parse_var_expression("name:fg[red]:bg[blue]");

        assert_eq!(result.value, Some("John".to_string()));
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert_eq!(result.bg_color, Some("blue".to_string()));
    }

    #[test]
    fn test_variable_with_context_colors() {
        let result = TEST_FORMATTER.parse_var_expression("name:fg[$my_fg_color]:bg[$my_bg_color]");

        assert_eq!(result.value, Some("John".to_string()));
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert_eq!(result.bg_color, Some("blue".to_string()));
    }

    #[test]
    fn test_variable_with_additional_string() {
        let result = TEST_FORMATTER.parse_var_expression("name+[!]");

        assert_eq!(result.value, Some("John".to_string()));
        assert_eq!(result.suffix, Some("!".to_string()));
        assert!(result.style.is_none());
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_variable_with_all_options() {
        let result =
            TEST_FORMATTER.parse_var_expression("name:pad:fg[$my_fg_color]:bg[$my_bg_color]+[!]");

        assert_eq!(result.value, Some("John".to_string()));
        assert_eq!(result.suffix, Some("!".to_string()));
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert_eq!(result.fg_color, Some("red".to_string()));
        assert_eq!(result.bg_color, Some("blue".to_string()));
    }

    #[test]
    fn test_variable_with_additional_string_before_expressions() {
        let result = TEST_FORMATTER.parse_var_expression("name+[!]:pad:fg[$my_fg_color]");

        assert_eq!(result.value, Some("John".to_string()));
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
        assert_eq!(groups[0].format(&TEST_FORMATTER.context), "John");
        assert!(groups[0].format(&TEST_FORMATTER.context).contains("John"));
        assert!(groups[0].format(&TEST_FORMATTER.context).contains("red"));
        assert!(groups[0].format(&TEST_FORMATTER.context).contains("blue"));

        // Check second variable
        assert_eq!(groups[1].format(&TEST_FORMATTER.context), "12:00");
        assert!(groups[1].format(&TEST_FORMATTER.context).contains("12:00"));
        assert!(groups[1].format(&TEST_FORMATTER.context).contains("fit"));
        assert!(groups[1].format(&TEST_FORMATTER.context).contains("blue"));

        // Check third variable
        assert_eq!(groups[2].format(&TEST_FORMATTER.context), "2024-03-20");
        assert!(groups[2]
            .format(&TEST_FORMATTER.context)
            .contains("2024-03-20"));
        assert!(groups[2].format(&TEST_FORMATTER.context).contains("!"));
        assert!(groups[2].format(&TEST_FORMATTER.context).contains("John"));
    }

    #[test]
    fn test_unknown_variable() {
        let result = TEST_FORMATTER.parse_var_expression("unknown:pad");

        assert_eq!(result.value, Some("".to_string()));
        assert!(result.suffix.is_none());
        assert!(matches!(result.style, Some(VarStyle::Pad)));
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_unknown_color_variable() {
        let result = TEST_FORMATTER.parse_var_expression("name:fg[$unknown_color]");

        assert_eq!(result.value, Some("John".to_string()));
        assert!(result.suffix.is_none());
        assert!(result.style.is_none());
        assert!(result.fg_color.is_none());
        assert!(result.bg_color.is_none());
    }

    #[test]
    fn test_process_fillers_with_direct_value() {
        let result = TEST_FORMATTER.process_fillers("f");
        assert_eq!(result, "#");
    }

    #[test]
    fn test_process_fillers_with_defaults_fill_char() {
        let mut vars = HashMap::new();
        vars.insert("defaults.fill_char".to_string(), "-".to_string());
        let context = TestContext { vars };
        let formatter = TemplateFormatter::new(FallbackMap::from(&context));

        let result = formatter.process_fillers("f");
        assert_eq!(result, "-");
    }

    #[test]
    fn test_process_fillers_with_default_value() {
        let context = TestContext {
            vars: HashMap::new(),
        };
        let formatter = TemplateFormatter::new(FallbackMap::from(&context));

        let result = formatter.process_fillers("f");
        assert_eq!(result.as_str(), &Defaults::default().fill_char);
    }

    #[test]
    fn test_process_fillers_with_non_filler() {
        let result = TEST_FORMATTER.process_fillers("name");
        assert_eq!(result, "John");
    }

    #[test]
    fn test_process_fillers_with_unknown_variable() {
        let result = TEST_FORMATTER.process_fillers("unknown");
        assert_eq!(result, "");
    }

    #[test]
    fn test_process_fillers_priority() {
        let mut vars = HashMap::new();
        vars.insert("f".to_string(), "#".to_string());
        vars.insert("defaults.fill_char".to_string(), "-".to_string());
        let context = TestContext { vars };
        let formatter = TemplateFormatter::new(FallbackMap::from(&context));

        let result = formatter.process_fillers("f");
        assert_eq!(result, "#"); // Should use direct value over defaults.fill_char
    }
}
