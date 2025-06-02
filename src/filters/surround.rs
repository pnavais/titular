use crate::context_manager::ContextManager;
use std::collections::HashMap;
use tera::{Error as TeraError, Value};

/// Type alias for a function that provides an optional string value
pub type ValueProviderFn = fn(&str) -> std::result::Result<Option<String>, TeraError>;

/// Default function that gets a value from ContextManager
fn default_value_provider(key: &str) -> std::result::Result<Option<String>, TeraError> {
    let ctx = ContextManager::get()
        .read()
        .map_err(|e| TeraError::msg(e.to_string()))?;
    Ok(ctx.get(key).map(String::from))
}

/// Create a surround filter closure for Tera using the default value provider
pub fn create_surround_filter(
) -> impl Fn(&Value, &HashMap<String, Value>) -> std::result::Result<Value, TeraError> {
    create_surround_filter_with(None)
}

/// Create a surround filter closure for Tera with a custom value provider
///
/// # Arguments
/// * `value_provider` - Function that provides a string value for a given key. Defaults to using ContextManager if None.
///
/// # Returns
/// A filter that surrounds the input text with start and end strings. The strings are obtained from:
/// 1. The primary key (surround_start/end) if available
/// 2. The defaults key (defaults.surround_start/end) as fallback
/// 3. An empty string if neither is available
pub fn create_surround_filter_with(
    value_provider: Option<ValueProviderFn>,
) -> impl Fn(&Value, &HashMap<String, Value>) -> std::result::Result<Value, TeraError> {
    let get_value = value_provider.unwrap_or(default_value_provider);
    move |value: &Value, _: &HashMap<String, Value>| {
        let text = tera::try_get_value!("surround", "value", String, value);

        // Get start string with fallback chain
        let start = get_value("surround_start")?
            .or(get_value("defaults.surround_start")?)
            .unwrap_or_default();

        // Get end string with fallback chain
        let end = get_value("surround_end")?
            .or(get_value("defaults.surround_end")?)
            .unwrap_or_default();

        Ok(Value::String(format!("{}{}{}", start, text, end)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surround_filter_empty_provider() {
        let filter = create_surround_filter_with(Some(|_| Ok(None)));
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "test");
    }

    #[test]
    fn test_surround_filter_with_mock_provider() {
        let filter = create_surround_filter_with(Some(|key| {
            Ok(match key {
                "surround_start" => Some("<".to_string()),
                "surround_end" => Some(">".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "<test>");
    }

    #[test]
    fn test_surround_filter_only_start() {
        let filter = create_surround_filter_with(Some(|key| {
            Ok(if key == "surround_start" {
                Some("<".to_string())
            } else {
                None
            })
        }));
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "<test");
    }

    #[test]
    fn test_surround_filter_only_end() {
        let filter = create_surround_filter_with(Some(|key| {
            Ok(if key == "surround_end" {
                Some(">".to_string())
            } else {
                None
            })
        }));
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "test>");
    }

    #[test]
    fn test_surround_filter_only_defaults() {
        let filter = create_surround_filter_with(Some(|key| {
            Ok(match key {
                "defaults.surround_start" => Some("<".to_string()),
                "defaults.surround_end" => Some(">".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "<test>");
    }

    #[test]
    fn test_surround_filter_mixed_provider() {
        let filter = create_surround_filter_with(Some(|key| {
            Ok(match key {
                "surround_start" => Some("<".to_string()),
                "defaults.surround_end" => Some(">".to_string()),
                _ => None,
            })
        }));
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "<test>");
    }

    #[test]
    fn test_surround_filter_default_provider() {
        let filter = create_surround_filter();
        let args = HashMap::new();
        let value = Value::String("test".to_string());

        let result = filter(&value, &args).unwrap();
        assert_eq!(result.as_str().unwrap(), "test");
    }
}
