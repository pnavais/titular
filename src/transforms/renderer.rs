use chrono::Local;
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error as StdError;
use tera::Tera;

use crate::config::TemplateConfig;
use crate::context_manager::ContextManager;
use crate::error::*;
use crate::filters::color;
use crate::filters::style;
use crate::transforms::Transform;
use crate::utils::safe_time_format;
use crate::DEFAULT_TIME_FORMAT;

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
    fn pre_process_pattern(pattern: &str) -> Result<String> {
        let mut processed = TERA_VAR_REGEX
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
            .to_string();

        // Add time if with-time flag is present
        let time_info = {
            let ctx = ContextManager::get().read()?;
            if ctx.is_active("with-time") {
                Some((
                    ctx.get("defaults.time_format")
                        .unwrap_or(DEFAULT_TIME_FORMAT)
                        .to_string(),
                    ctx.get("defaults.time_pattern")
                        .unwrap_or(" [{{ time }}]")
                        .to_string(),
                ))
            } else {
                None
            }
        };

        if let Some((time_format, time_pattern)) = time_info {
            let current_time = safe_time_format(&Local::now(), &time_format);

            // Insert the current time into the context
            ContextManager::get().update(|ctx| {
                ctx.insert("time", &current_time);
            })?;

            processed.push_str(&time_pattern);
        }

        Ok(processed)
    }

    /// Renders a template string using the global context
    ///
    /// # Arguments
    /// * `pattern_data` - The pattern data to be rendered
    ///
    /// # Returns
    /// A rendered string
    pub fn render(&self, pattern_data: &str) -> Result<String> {
        let pattern = Self::pre_process_pattern(pattern_data)?;

        let mut tera = Tera::default();
        tera.register_filter("color", color::create_color_filter());
        tera.register_filter("style", style::create_style_filter());

        // Get template name and pattern first
        let template_name = {
            let ctx = ContextManager::get().read()?;
            let template_content = ctx
                .get_object::<TemplateConfig>("template_config")
                .ok_or_else(|| {
                    Error::TemplateRenderError("Template configuration not found".to_string())
                })?;

            template_content
                .details
                .name
                .to_lowercase()
                .replace(" ", "_")
        };

        tera.add_raw_template(&template_name, &pattern)?;

        // Do the render with the context directly
        let template = {
            let ctx = ContextManager::get().read()?;
            tera.render(&template_name, ctx.get_data())
                .map_err(|e: tera::Error| {
                    let mut error_msg = e.to_string();
                    if let Some(source) = e.source() {
                        error_msg.push_str("\nCaused by: ");
                        error_msg.push_str(&source.to_string());
                    }
                    Error::TemplateRenderError(error_msg)
                })?
        };

        Ok(template)
    }
}

impl Transform for TemplateRenderer {
    fn transform(&self, text: &str) -> Result<String> {
        self.render(text)
    }
}
