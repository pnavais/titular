use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error as StdError;
use std::sync::Arc;
use tera::Tera;

use crate::color_filter;
use crate::config::TemplateConfig;
use crate::context::Context;
use crate::error::*;
use crate::style_filter;

static TERA_VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{([^}]+)\}\}").unwrap());

pub struct TemplateFormatter {
    context: Arc<Context>,
}

impl TemplateFormatter {
    pub fn new(context: Context) -> Self {
        Self {
            context: Arc::new(context),
        }
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

    /// Formats a template string using the provided context.
    ///
    /// # Arguments
    /// * `template_content` - The template configuration containing the pattern to format
    ///
    /// # Returns
    /// A formatted string
    pub fn format(&self, template_content: &TemplateConfig) -> Result<String> {
        let template_name = template_content
            .details
            .name
            .to_lowercase()
            .replace(" ", "_");

        let pattern = Self::pre_process_pattern(&template_content.pattern.data);
        let mut tera = Tera::default();

        // Register filters
        tera.register_filter(
            "color",
            color_filter::create_color_filter(Arc::clone(&self.context)),
        );
        tera.register_filter(
            "style",
            style_filter::create_style_filter(Arc::clone(&self.context)),
        );

        tera.add_raw_template(&template_name, &pattern)?;
        let template = tera
            .render(&template_name, &self.context.get_data())
            .map_err(|e| {
                println!("Error: {:?}", e);
                let mut error_msg = e.to_string();
                let mut current = e.source();
                while let Some(source) = current {
                    error_msg.push_str("\nCaused by: ");
                    error_msg.push_str(&source.to_string());
                    current = source.source();
                }
                Error::TemplateRenderError(error_msg)
            })?;
        Ok(template)
    }
}
