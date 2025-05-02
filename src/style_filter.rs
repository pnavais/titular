use std::collections::HashMap;
use std::sync::Arc;
use tera::{Error as TeraError, Value};

use crate::color_manager::{ColorManager, StyleFormat, StyleScope};
use crate::context::Context;

/// Create a style filter closure for Tera
///
/// # Arguments
/// * `context` - The context containing color configurations
///
/// # Returns
/// A closure that can be used with Tera's register_filter
pub fn create_style_filter(
    context: Arc<Context>,
) -> impl Fn(&Value, &HashMap<String, Value>) -> Result<Value, TeraError> {
    move |value: &Value, args: &HashMap<String, Value>| apply_style(&context, value, args)
}

/// Apply style formatting to the given text using the provided color values
///
/// # Arguments
/// * `context` - The context containing color configurations
/// * `value` - The text value to style
/// * `args` - The filter arguments containing optional fg_color and bg_color
///
/// # Returns
/// A Result containing either the styled text or an error
fn apply_style(
    context: &Context,
    value: &Value,
    args: &HashMap<String, Value>,
) -> Result<Value, TeraError> {
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

    Ok(Value::String(ColorManager::format(context, &text, style)))
}
