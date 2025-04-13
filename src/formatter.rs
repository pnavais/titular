use crate::config::TemplateConfig;
use crate::context::Context;
use crate::error::*;

pub struct TemplateFormatter {}

impl TemplateFormatter {
    pub fn format(template_content: &TemplateConfig, context: &Context) -> Result<bool> {
        //for (key, value) in context.iter() {
        //    formatted = formatted.replace(&format!("{{{}}}", key), value);
        //}
        //Ok(formatted)
        Ok(true)
    }
}
