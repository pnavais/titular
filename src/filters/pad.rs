use crate::constants::padding;
use std::collections::HashMap;
use tera::{Error as TeraError, Value};

/// Create a pad filter closure for Tera
///
/// The pad filter surrounds the text with non-visible Unicode markers
/// to identify padding groups that can be extracted later.
///
/// # Arguments
/// * `value` - The input string to process
/// * `args` - A HashMap containing the filter arguments (not used yet)
///
/// # Returns
/// A closure that can be used with Tera's register_filter
pub fn create_pad_filter() -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError> {
    move |value: &Value, _args: &HashMap<String, Value>| {
        let text = tera::try_get_value!("pad", "value", String, value);

        // Surround the text with non-visible markers
        Ok(Value::String(format!(
            "{}{}{}",
            padding::START,
            text,
            padding::END
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_filter_basic() {
        let filter = create_pad_filter();
        let args = HashMap::new();
        let value = Value::String("hello".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(
            result.as_str().unwrap(),
            format!("{}hello{}", padding::START, padding::END)
        );
    }

    #[test]
    fn test_pad_filter_with_ansi() {
        let filter = create_pad_filter();
        let args = HashMap::new();
        let value = Value::String("\x1b[31mhello\x1b[0m".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(
            result.as_str().unwrap(),
            format!("{}\x1b[31mhello\x1b[0m{}", padding::START, padding::END)
        );
    }

    #[test]
    fn test_pad_filter_with_emoji() {
        let filter = create_pad_filter();
        let args = HashMap::new();
        let value = Value::String("hello 🦀".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(
            result.as_str().unwrap(),
            format!("{}hello 🦀{}", padding::START, padding::END)
        );
    }

    #[test]
    fn test_pad_filter_with_ansi_and_emoji() {
        let filter = create_pad_filter();
        let args = HashMap::new();
        let value = Value::String("\x1b[31mhello 🦀\x1b[0m".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(
            result.as_str().unwrap(),
            format!("{}\x1b[31mhello 🦀\x1b[0m{}", padding::START, padding::END)
        );
    }
}
