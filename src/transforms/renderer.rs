use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error as StdError;
use std::sync::Arc;
use tera::Tera;

use crate::config::TemplateConfig;
use crate::context::Context;
use crate::error::*;
use crate::filters::color;
use crate::filters::style;
use crate::transforms::Transform;

static TERA_VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{([^}]+)\}\}").unwrap());

pub struct TemplateRenderer {}

/// TemplateRenderer is a transform that renders a template string using the provided context.
/// It uses the Tera template engine to render the template under the hood.
/// It also registers the custom filters (color and style filters) to be used by the Tera engine.
impl TemplateRenderer {
    pub fn new() -> Self {
        Self {}
    }

    /// Pre-processes the template pattern to add default filter to all variables
    ///
    /// # Arguments
    /// * `pattern` - The template pattern to pre-process
    ///
    /// # Returns
    /// A pre-processed template pattern
    fn pre_process_pattern(pattern: &str) -> String {
        TERA_VAR_REGEX
            .replace_all(pattern, |caps: &regex::Captures| {
                let content = caps.get(1).unwrap().as_str().trim();
                if content.contains('|') {
                    // Has filters, insert default as first filter
                    format!(
                        "{{{{ {} | default(value='') | {} }}}}",
                        content.split('|').next().unwrap().trim(),
                        content
                            .split('|')
                            .skip(1)
                            .collect::<Vec<&str>>()
                            .join("|")
                            .trim()
                    )
                } else {
                    // No filters, just add default
                    format!("{{{{ {} | default(value='') }}}}", content)
                }
            })
            .to_string()
    }

    /// Renders a template string using the provided context and applies the transform chain
    ///
    /// # Arguments
    /// * `context` - The context containing the template configuration and variables
    /// * `pattern_data` - The pattern data to be rendered
    ///
    /// # Returns
    /// A rendered string
    pub fn render(&self, context: Arc<Context>, pattern_data: &str) -> Result<String> {
        // Get the template configuration from the context's registry
        let template_content = context
            .get_object::<TemplateConfig>("template_config")
            .ok_or_else(|| {
                Error::TemplateRenderError("Template configuration not found".to_string())
            })?;

        let template_name = template_content
            .details
            .name
            .to_lowercase()
            .replace(" ", "_");

        let pattern = Self::pre_process_pattern(pattern_data);
        let mut tera = Tera::default();

        // Register filters
        tera.register_filter("color", color::create_color_filter(Arc::clone(&context)));
        tera.register_filter("style", style::create_style_filter(Arc::clone(&context)));

        tera.add_raw_template(&template_name, &pattern)?;
        let template =
            tera.render(&template_name, context.get_data())
                .map_err(|e: tera::Error| {
                    println!("Error: {:?}", e);
                    let mut error_msg = e.to_string();
                    if let Some(source) = e.source() {
                        error_msg.push_str("\nCaused by: ");
                        error_msg.push_str(&source.to_string());
                    }
                    Error::TemplateRenderError(error_msg)
                })?;

        Ok(template)
    }
}

impl Transform for TemplateRenderer {
    fn transform(&self, context: Arc<Context>, text: &str) -> Result<String> {
        self.render(context, text)
    }
}
