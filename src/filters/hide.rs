use console::{measure_text_width, strip_ansi_codes};
use std::collections::HashMap;
use tera::{Error as TeraError, Value};

/// Type alias for a function that provides an optional string value
pub type ValueProviderFn = fn(&str) -> std::result::Result<Option<String>, TeraError>;

/// Default function that gets a value from ContextManager
fn default_value_provider(key: &str) -> std::result::Result<Option<String>, TeraError> {
    use crate::context_manager::ContextManager;
    let ctx = ContextManager::get()
        .read()
        .map_err(|e| TeraError::msg(e.to_string()))?;
    Ok(ctx.get(key).map(String::from))
}

/// Create a hide filter closure for Tera using the default value provider
pub fn create_hide_filter() -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError>
{
    create_hide_filter_with(None)
}

/// Create a hide filter closure for Tera with a custom value provider
///
/// # Arguments
/// * `value_provider` - Function that provides a string value for a given key. Defaults to using ContextManager if None.
///
/// # Returns
/// A filter that replaces the text with spaces of the same visual width if the hide flag is active.
/// The width is calculated using proper Unicode character width measurement,
/// ensuring correct handling of emojis and other wide characters.
pub fn create_hide_filter_with(
    value_provider: Option<ValueProviderFn>,
) -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError> {
    let get_value = value_provider.unwrap_or(default_value_provider);
    move |value: &Value, _args: &HashMap<String, Value>| {
        let text = tera::try_get_value!("hide", "value", String, value);

        // Check if hide flag is active
        let is_active = get_value("hide")?
            .map(|v| matches!(v.trim().to_lowercase().as_str(), "true" | "1"))
            .unwrap_or(false);

        if is_active {
            // Strip ANSI codes and measure the visual width
            let stripped_text = strip_ansi_codes(&text);
            let visual_width = measure_text_width(&stripped_text);
            let spaces = " ".repeat(visual_width);
            Ok(Value::String(spaces))
        } else {
            Ok(Value::String(text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hide_filter_basic() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("false".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("Hello".to_string());

        // Test when hide is inactive
        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello");

        // Test when hide is active
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "     "); // 5 spaces for "Hello"
    }

    #[test]
    fn test_hide_filter_with_emoji() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("Hello 🦀".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "        "); // 8 spaces (5 for "Hello" + 2 for 🦀 + 1 for space)
    }

    #[test]
    fn test_hide_filter_with_ansi() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("\x1b[31mHello\x1b[0m".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "     "); // 5 spaces for "Hello", ANSI codes are stripped
    }

    #[test]
    fn test_hide_filter_with_ansi_and_emoji() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("\x1b[31mHello 🦀\x1b[0m".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "        "); // 8 spaces (5 for "Hello" + 1 for space + 2 for 🦀)
    }

    #[test]
    fn test_hide_filter_with_mixed_unicode() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("Hello 🌟 世界 🦀".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "                "); // 16 spaces (5 for "Hello" + 1 for space + 2 for 🌟 + 1 for space + 2 for 世界 + 1 for space + 2 for 🦀 + 1 for trailing space)
    }

    #[test]
    fn test_hide_filter_with_zero_width_chars() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("Hello\u{200B}World".to_string()); // Zero-width space between Hello and World

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "          "); // 10 spaces (5 for "Hello" + 5 for "World", zero-width char is ignored)
    }

    #[test]
    fn test_hide_filter_with_combining_chars() {
        let filter = create_hide_filter_with(Some(|key| {
            Ok(match key {
                "hide" => Some("true".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("e\u{0301}".to_string()); // 'e' with combining acute accent

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), " "); // 1 space (combining char doesn't add width)
    }

    #[test]
    fn test_hide_filter_default_provider() {
        use crate::context_manager::ContextManager;

        let filter = create_hide_filter();
        let args = HashMap::new();
        let value = Value::String("Hello".to_string());

        // Test when hide is inactive (default state)
        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "Hello");

        // Test when hide is active
        ContextManager::get()
            .update(|ctx| {
                ctx.insert("hide", "true");
            })
            .unwrap();
        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "     "); // 5 spaces for "Hello"
    }
}
