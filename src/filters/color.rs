use std::collections::HashMap;
use tera::{Error as TeraError, Value};

use crate::color_manager::{ColorManager, StyleFormat, StyleScope};
use crate::context_manager::ContextManager;

/// Create a color filter closure for Tera
///
/// # Returns
/// A closure that can be used with Tera's register_filter
pub fn create_color_filter() -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError>
{
    move |value: &Value, args: &HashMap<String, Value>| {
        let text = tera::try_get_value!("color", "value", String, value);

        // Get the color value and strip the special prefix if present
        let color_value = args
            .get("name")
            .ok_or_else(|| TeraError::msg("Missing name argument"))?
            .as_str()
            .ok_or_else(|| TeraError::msg("Color value must be a string"))?
            .trim_start_matches("raw:")
            .trim_matches('"')
            .to_string();

        // Default to false if is_bg is not provided or not a boolean
        let is_bg = args.get("is_bg").and_then(|v| v.as_bool()).unwrap_or(false);

        let style = StyleFormat {
            fg_color: (!is_bg).then_some(color_value.clone()),
            bg_color: is_bg.then_some(color_value),
            scope: if is_bg {
                StyleScope::BG
            } else {
                StyleScope::FG
            },
        };

        let ctx = ContextManager::get()
            .read()
            .map_err(|e| TeraError::msg(e.to_string()))?;

        Ok(Value::String(ColorManager::format(&ctx, &text, style)))
    }
}
