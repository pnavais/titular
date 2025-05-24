use std::collections::HashMap;
use tera::{Error as TeraError, Value};

use crate::color_manager::{ColorManager, StyleFormat, StyleScope};
use crate::context_manager::ContextManager;

/// Create a style filter closure for Tera
///
/// # Returns
/// A closure that can be used with Tera's register_filter
pub fn create_style_filter() -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError>
{
    move |value: &Value, args: &HashMap<String, Value>| {
        let text = tera::try_get_value!("style", "value", String, value);

        // Get the color values and strip the special prefix if present
        let fg_color = args
            .get("fg_color")
            .and_then(|v| v.as_str())
            .map(|s| s.trim_start_matches("raw:").trim_matches('"').to_string());
        let bg_color = args
            .get("bg_color")
            .and_then(|v| v.as_str())
            .map(|s| s.trim_start_matches("raw:").trim_matches('"').to_string());

        let scope = match (fg_color.as_ref(), bg_color.as_ref()) {
            (Some(_), Some(_)) => StyleScope::BOTH,
            (Some(_), None) => StyleScope::FG,
            (None, Some(_)) => StyleScope::BG,
            (None, None) => return Ok(Value::String(text)), // No colors provided, return original text
        };

        let style = StyleFormat {
            fg_color,
            bg_color,
            scope,
        };

        let ctx = ContextManager::get()
            .read()
            .map_err(|e| TeraError::msg(e.to_string()))?;

        Ok(Value::String(ColorManager::format(&ctx, &text, style)))
    }
}
