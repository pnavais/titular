use crate::error::Result;
use crate::transforms::Transform;
use ansi_parser::{AnsiParser, Output};
use std::collections::VecDeque;

const RESET_CODE: &str = "\x1b[0m";

pub struct AnsiFormatter;

impl AnsiFormatter {
    pub fn new() -> Self {
        Self
    }

    fn process_ansi_codes(&self, input: &str) -> String {
        let mut result = String::new();
        let mut stack = VecDeque::<String>::new();

        for output in input.ansi_parse() {
            match output {
                Output::TextBlock(text) => {
                    result.push_str(text);
                }
                Output::Escape(escape) => {
                    let escape_str = escape.to_string();
                    // Always push the escape string to the result
                    result.push_str(&escape_str);

                    if escape_str == RESET_CODE {
                        // Full reset - pop last element
                        if let Some(_) = stack.pop_back() {
                            // After reset, reapply remaining stack in order
                            for code in stack.iter() {
                                result.push_str(code);
                            }
                        }
                    } else {
                        // Non-reset code - move into stack
                        stack.push_back(escape_str);
                    }
                }
            }
        }

        // At the end, apply any remaining ANSI codes in reverse insertion order
        while let Some(code) = stack.pop_back() {
            result.push_str(&code);
        }

        result
    }
}

impl Transform for AnsiFormatter {
    fn transform(&self, text: &str) -> Result<String> {
        Ok(self.process_ansi_codes(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_colors() {
        let formatter = AnsiFormatter::new();
        let input = "\x1b[31mRed\x1b[32mGreen\x1b[0mBack to Red";
        let result = formatter.transform(input).unwrap();
        assert_eq!(
            result,
            "\x1b[31mRed\x1b[32mGreen\x1b[0m\x1b[31mBack to Red\x1b[31m"
        );
    }

    #[test]
    fn test_multiple_resets() {
        let formatter = AnsiFormatter::new();
        let input = "\x1b[31mRed\x1b[32mGreen\x1b[0mBack to Red\x1b[0mNormal";
        let result = formatter.transform(input).unwrap();
        assert_eq!(
            result,
            "\x1b[31mRed\x1b[32mGreen\x1b[0m\x1b[31mBack to Red\x1b[0mNormal"
        );
    }

    #[test]
    fn test_no_ansi_codes() {
        let formatter = AnsiFormatter::new();
        let input = "Normal text";
        let result = formatter.transform(input).unwrap();
        assert_eq!(result, "Normal text");
    }

    #[test]
    fn test_remaining_codes() {
        let formatter = AnsiFormatter::new();
        let input = "\x1b[31mRed\x1b[32mGreen";
        let result = formatter.transform(input).unwrap();
        assert_eq!(result, "\x1b[31mRed\x1b[32mGreen\x1b[32m\x1b[31m");
    }
}
