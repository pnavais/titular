use crate::string_utils::is_visually_empty;
use std::collections::HashMap;
use tera::{Error as TeraError, Value};

/// Create an append filter closure for Tera
///
/// # Returns
/// A filter that appends a string literal to the input text if provided.
/// The string to append is passed as the "text" named argument.
/// If the input text is visually empty (contains only whitespace, ANSI codes, or other non-printable characters),
/// it is returned unchanged regardless of the append text.
/// If the append text is visually empty, it is not appended.
///
/// # Examples
/// ```tera
/// {{ "hello" | append(text=" world") }} # Using named argument
/// {{ "hello" | append(text="\x1b[31m\x1b[0m") }} # Visually empty text (only ANSI codes)
/// {{ "\x1b[31m\x1b[0m" | append(text=" world") }} # Visually empty input, returns unchanged
/// ```
pub fn create_append_filter() -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError>
{
    move |value: &Value, args: &HashMap<String, Value>| {
        let text = tera::try_get_value!("append", "value", String, value);

        // If input is visually empty, return it unchanged, otherwise append if append text is not empty
        Ok(Value::String(if is_visually_empty(&text) {
            text
        } else {
            let append_text = args
                .get("text")
                .and_then(|v| v.as_str())
                .filter(|s| !is_visually_empty(s))
                .unwrap_or("");
            format!("{}{}", text, append_text)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_filter_with_text() {
        let filter = create_append_filter();
        let mut args = HashMap::new();
        args.insert("text".to_string(), Value::String(" world".to_string()));
        let value = Value::String("hello".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_append_filter_without_args() {
        let filter = create_append_filter();
        let args = HashMap::new();
        let value = Value::String("hello".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_append_filter_with_empty_text() {
        let filter = create_append_filter();
        let mut args = HashMap::new();
        args.insert("text".to_string(), Value::String("".to_string()));
        let value = Value::String("hello".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello");
    }

    #[test]
    fn test_append_filter_with_ansi_text() {
        let filter = create_append_filter();
        let mut args = HashMap::new();
        args.insert(
            "text".to_string(),
            Value::String("\x1b[31m\x1b[0m".to_string()),
        ); // Red color code
        let value = Value::String("hello".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello"); // Should not append visually empty text
    }

    #[test]
    fn test_append_filter_with_ansi_and_text() {
        let filter = create_append_filter();
        let mut args = HashMap::new();
        args.insert(
            "text".to_string(),
            Value::String("\x1b[31m world\x1b[0m".to_string()),
        ); // Red " world"
        let value = Value::String("hello".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "hello\x1b[31m world\x1b[0m"); // Should append text with ANSI codes
    }

    #[test]
    fn test_append_filter_with_empty_input() {
        let filter = create_append_filter();
        let mut args = HashMap::new();
        args.insert("text".to_string(), Value::String(" world".to_string()));
        let value = Value::String("\x1b[31m\x1b[0m".to_string()); // Visually empty input

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "\x1b[31m\x1b[0m"); // Should return input unchanged
    }

    #[test]
    fn test_append_filter_with_whitespace_input() {
        let filter = create_append_filter();
        let mut args = HashMap::new();
        args.insert("text".to_string(), Value::String(" world".to_string()));
        let value = Value::String(" ".to_string()); // Single space input

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "  world"); // Two spaces: one from input + one from " world"
    }
}
