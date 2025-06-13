use chrono::Local;
use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error as StdError;
use std::sync::Mutex;
use tera::Tera;

use crate::config::TemplateConfig;
use crate::constants::template::DEFAULT_TIME_FORMAT;
use crate::error::*;
use crate::filters::{append, color, hide, pad, style, surround};
use crate::prelude::*;
use crate::utils::safe_time_format;

static TERA_VAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\{\{([^}]+)\}\}").unwrap());

static TERA: Lazy<Mutex<Tera>> = Lazy::new(|| {
    let mut tera = Tera::default();
    tera.register_filter("color", color::create_color_filter());
    tera.register_filter("style", style::create_style_filter());
    tera.register_filter("surround", surround::create_surround_filter());
    tera.register_filter("append", append::create_append_filter());
    tera.register_filter("pad", pad::create_pad_filter());
    tera.register_filter("hide", hide::create_hide_filter());
    Mutex::new(tera)
});

static FILTER_ARGS_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Captures: 1=filter_name, 2=arguments
    Regex::new(r"(\w+)\(([^)]+)\)").unwrap()
});

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
        Self::add_time_marker(Self::add_default_markers(pattern))
    }

    /// Extracts filter arguments and adds default values for unquoted values to the context
    /// if not already present.
    ///
    /// # Arguments
    /// * `filter_chain` - The filter chain to process
    ///
    fn process_filter_args(filter_chain: &str) {
        // Skip the variable part and process only the filters
        filter_chain
            .split('|')
            .skip(1) // Skip the variable part (before first |)
            .flat_map(|args_part| FILTER_ARGS_REGEX.captures_iter(args_part))
            .filter_map(|caps| caps.get(2).map(|m| m.as_str()))
            .flat_map(|args| args.split(','))
            .filter_map(|arg| arg.trim().split_once('=').map(|(_, v)| v.trim()))
            .filter(|value| !matches!(value.chars().next(), Some('"') | Some('\'')))
            .for_each(|value| {
                ContextManager::get()
                    .update(|ctx| {
                        if ctx.get(value).is_none() {
                            ctx.insert(value, "");
                        }
                    })
                    .unwrap_or_default();
            });
    }

    /// Adds default markers to all variables in the pattern
    ///
    /// # Arguments
    /// * `pattern` - The pattern to add default markers to
    ///
    /// # Returns
    /// The processed pattern with default markers
    fn add_default_markers(pattern: &str) -> String {
        TERA_VAR_REGEX
            .replace_all(pattern, |caps: &regex::Captures| {
                let content = caps.get(1).unwrap().as_str().trim();
                if content.contains('|') {
                    // Process filter arguments to add missing vars to context
                    Self::process_filter_args(content);
                    // Has filters, insert default as first filter
                    format!(
                        "{{{{ {} | default(value='') | {} }}}}",
                        content.split('|').next().unwrap().trim(),
                        content
                            .split('|')
                            .skip(1)
                            .collect::<Vec<_>>()
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

    /// Adds a time marker to the processed string if the with-time flag is active
    ///
    /// # Arguments
    /// * `processed` - The processed string to add the time marker to
    ///
    /// # Returns
    /// Result containing the processed string
    fn add_time_marker(mut processed: String) -> Result<String> {
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

        let mut tera = TERA
            .lock()
            .map_err(|e| Error::TemplateRenderError(e.to_string()))?;
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
