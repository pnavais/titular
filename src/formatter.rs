use crate::prelude::*;
use crate::{
    config::MainConfig, constants::template::DEFAULT_TEMPLATE_NAME, context::Context, debug,
    reader::TemplateReader, transforms::TransformManager, writer::TemplateWriter,
};
use std::io::{stdout, Write};

#[cfg(feature = "fetcher")]
use crate::{constants::template::DEFAULT_REMOTE_REPO, fetcher::TemplateFetcher};

pub struct TemplateFormatter<'a> {
    input_dir: &'a std::path::PathBuf,
    config: &'a MainConfig,
}

impl<'a> TemplateFormatter<'a> {
    pub fn new(input_dir: &'a std::path::PathBuf, config: &'a MainConfig) -> Self {
        Self { input_dir, config }
    }

    /// Performs the rendering of the template using the template formatter.
    /// In case it's not present (and is not the default template), it will be downloaded
    /// automatically from the remote repository (if the "fetcher" feature is enabled).
    ///
    /// # Arguments
    /// * `context` - The context to be used for rendering the template.
    /// * `template_name` - The name of the template to be rendered.
    ///
    /// # Returns
    /// Returns `Ok(true)` if the template was rendered successfully, `Err(Error)` if the template does not exist.
    pub fn format(&self, context: &Context, template_name: &str) -> Result<bool> {
        self.preprocess_template(template_name)?;

        let template_payload = TemplateReader::read(self.input_dir, template_name)?;
        let pattern_data = template_payload.pattern.data.to_string();

        // Update the context in a clean way
        crate::context_manager::ContextManager::get().update(|ctx| {
            ctx.append_from(context);
            ctx.append(&self.config.vars);
            ctx.append(&template_payload.vars);
            ctx.store_object("template_config", template_payload);
        })?;

        write!(
            stdout(),
            "{}",
            TransformManager::get().process(&pattern_data)?
        )?;
        Ok(true)
    }

    /// Performs the preprocessing of the template.
    /// In case we are pointing to a recoverable template, we try to recover it (i.e. basic).
    /// In case the "fetched" feature is enabled, the template is downloaded
    /// automatically in case it's not present (and is available in the remote repository).
    ///
    /// # Arguments
    /// * `template_name` - The name of the template to be preprocessed.
    ///
    /// # Returns
    /// Returns `Ok(())` if the template was preprocessed successfully, `Err(Error)` if the template does not exist.
    fn preprocess_template(&self, template_name: &str) -> Result<()> {
        let path = TemplateWriter::get_template_file(template_name);
        let template = self.input_dir.join(&path);

        if !template.exists() && template_name == DEFAULT_TEMPLATE_NAME {
            debug!("Recovering template");
            TemplateWriter::write_new(&template, self.config)?;
        }
        #[cfg(feature = "fetcher")]
        if !template.exists() {
            // Try to fetch the template from the remote repository
            TemplateFetcher::fetch_from_remote(
                self.config
                    .templates
                    .remote_repo
                    .as_deref()
                    .unwrap_or(DEFAULT_REMOTE_REPO),
                template_name,
                self.input_dir,
            )?;
        }
        Ok(())
    }
}
