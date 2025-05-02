use std::{
    io::{stdin, stdout, Write},
    path::PathBuf,
};

use crate::{
    config::{MainConfig, DEFAULT_TEMPLATE_EXT, DEFAULT_TEMPLATE_NAME},
    context::Context,
    debug, display,
    error::*,
    formatter::TemplateFormatter,
    reader::TemplateReader,
    writer::TemplateWriter,
};

use crate::utils;

#[cfg(feature = "fetcher")]
use crate::fetcher::TemplateFetcher;

#[cfg(feature = "fetcher")]
use crate::config::DEFAULT_REMOTE_REPO;

#[cfg(feature = "display")]
use crate::theme::ThemeManager;

use glob::glob;
use nu_ansi_term::Color::{Green, Red, Yellow};

pub struct TemplatesController<'a> {
    pub input_dir: PathBuf,
    pub config: &'a MainConfig,
}

/// Provides all the operations involving templates management (list, open, create, edit, add (when "fetcher" feature enabled))
/// and formatting/rendering.
impl<'a> TemplatesController<'a> {
    pub fn new(input_dir: PathBuf, config: &'a MainConfig) -> Self {
        Self { input_dir, config }
    }

    /// Runs any of the templates subcommands. Currently supported :
    /// - list : shows the files stored in the templates repository
    /// - edit : opens or creates if not existing the given template in the default system editor (see "edit" crate for more information)
    /// - create : creates a new template from sratch with a default template pattern
    /// - remove : deletes the given template from the templates repository
    /// - add (only when feature "fetcher" is enabled) : downloads and installs a template from the default templates remote repository
    ///   or a custom URL
    ///
    /// # Arguments
    /// * `context` - The context containing the subcommand and template name.
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    pub fn run_template_subcommand(&self, context: &Context) -> Result<bool> {
        match context.get("subcommand") {
            Some(cmd) => match cmd {
                "list" => {
                    #[cfg(feature = "display")]
                    {
                        self.list(context)
                    }
                    #[cfg(not(feature = "display"))]
                    {
                        self.list()
                    }
                }
                "create" | "edit" | "remove" | "show" => {
                    let template_name = context
                        .get("template")
                        .ok_or_else(|| Error::CommandError("Missing template name".to_string()))?;
                    if cmd == "create" {
                        let created = self.create(template_name)?;
                        if created && !context.contains("no-edit") {
                            self.open(template_name)
                        } else {
                            Ok(true)
                        }
                    } else if cmd == "edit" {
                        self.open(template_name)
                    } else if cmd == "remove" {
                        self.remove(template_name)
                    } else if cmd == "show" {
                        self.display(template_name, context)
                    } else {
                        Err(Error::ArgsProcessingError(
                            "Invalid subcommand provided".to_string(),
                        ))
                    }
                }
                #[cfg(feature = "fetcher")]
                "add" => context
                    .get("url")
                    .ok_or_else(|| Error::CommandError("Missing URL".to_string()))
                    .and_then(|url| {
                        TemplateFetcher::fetch(url, &self.input_dir, context.is_active("force"))
                    }),
                _ => Err(Error::ArgsProcessingError(
                    "Invalid subcommand provided".to_string(),
                )),
            },
            _ => Err(Error::ArgsProcessingError(
                "Command not found in context".to_string(),
            )),
        }
    }

    /// Lists the templates or themes currently available in the binary.
    ///
    /// This function retrieves the list of templates or themes from the binary and prints them to the console.
    ///
    /// # Arguments
    /// * `context` - The context containing the subcommand and template name. Only used when "display" feature is enabled.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use titular::{controller::TemplatesController, config::MainConfig, context::Context};
    ///
    /// let config = MainConfig::default();
    /// let input_dir = PathBuf::from("templates");
    /// let controller = TemplatesController::new(input_dir, &config);
    /// let context = Context::new();
    ///
    /// #[cfg(feature = "display")]
    /// let result = controller.list(&context);
    /// #[cfg(not(feature = "display"))]
    /// let result = controller.list();
    ///
    /// assert!(result.is_ok());
    #[cfg(feature = "display")]
    pub fn list(&self, context: &Context) -> Result<bool> {
        if context.is_active("themes") {
            return self.list_themes();
        }
        self.list_templates()
    }

    #[cfg(not(feature = "display"))]
    pub fn list(&self) -> Result<bool> {
        self.list_templates()
    }

    /// Lists the themes currently available in the binary.
    ///
    /// This function retrieves the list of themes from the themes binary and prints them to the console.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    #[cfg(feature = "display")]
    pub fn list_themes(&self) -> Result<bool> {
        ThemeManager::init()?.list_themes()?;
        Ok(true)
    }

    /// Lists the templates currently available in the templates repository.
    ///
    /// This function retrieves the list of templates from the templates repository and prints them to the console.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///

    /// ```
    pub fn list_templates(&self) -> Result<bool> {
        if self.input_dir.exists() {
            let templates = glob(&format!(
                "{}{}{}",
                self.input_dir.to_string_lossy(),
                "/**/*",
                DEFAULT_TEMPLATE_EXT
            ))
            .expect("Failed to read glob pattern");

            let files: Vec<String> = templates
                .map(|t| {
                    t.unwrap()
                        .file_name()
                        .unwrap()
                        .to_owned()
                        .into_string()
                        .unwrap()
                })
                .collect();

            let root = self.input_dir.to_string_lossy().to_string();
            utils::print_tree(&files, "template", &root);

            Ok(true)
        } else {
            println!(
                "{}",
                Red.paint(format!(
                    "Templates directory \"{}\" not found",
                    self.input_dir.to_string_lossy()
                ))
            );
            Ok(false)
        }
    }

    /// Creates a new template from stratch using the default template contents.
    ///
    /// # Arguments
    /// * `name` - The name of the template to create.
    ///
    /// # Returns
    /// Returns `Ok(true)` if the template was created successfully, `Ok(false)` if the template already exists.
    pub fn create(&self, name: &str) -> Result<bool> {
        let (_, _, created) =
            TemplateWriter::create_new_template(name, false, &self.input_dir, self.config)?;
        if created {
            println!("New template \"{}\" created", Green.paint(name));
        } else {
            println!(
                "{}",
                Yellow.paint(format!("Template \"{}\" already exists", name))
            );
        }
        Ok(created)
    }

    /// Opens the given template in the default system editor (see "edit" crate for detailed information).
    ///
    /// # Arguments
    /// * `name` - The name of the template to open.
    ///
    /// # Returns
    /// Returns `Ok(())` if the template was opened successfully, `Err(Error)` if the template does not exist.
    pub fn open(&self, name: &str) -> Result<bool> {
        let (path, template, _) = self.create_new_template(name, true)?;

        if !path.is_empty() {
            match edit::edit_file(&template) {
                Ok(_) => Ok(true),
                Err(e) => Err(Error::TemplateReadError {
                    file: path,
                    cause: e.to_string(),
                }),
            }
        } else {
            Ok(true)
        }
    }

    /// Removes the template from the templates repository.
    ///
    /// # Arguments
    /// * `name` - The name of the template to remove.
    ///
    /// # Returns
    /// Returns `Ok(())` if the template was removed successfully, `Err(Error)` if the template does not exist.
    pub fn remove(&self, name: &str) -> Result<bool> {
        let path = TemplateWriter::get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        if template.exists() {
            match std::fs::remove_file(template) {
                Ok(_) => println!("Template \"{}\" removed", Green.paint(name)),
                Err(e) => {
                    return Err(Error::TemplateReadError {
                        file: path,
                        cause: e.to_string(),
                    });
                }
            }
        } else {
            println!(
                "{}",
                Yellow.paint(format!("Template \"{}\" not found", name))
            );
        }

        Ok(true)
    }

    /// Displays the contents of the given template.
    ///
    /// # Arguments
    /// * `name` - The name of the template to display.
    ///
    /// # Returns
    /// Returns `Ok(())` if the template was displayed successfully,
    /// `Err(Error)` if the template does not exist.
    pub fn display(&self, name: &str, context: &Context) -> Result<bool> {
        let path = TemplateWriter::get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        if template.exists() {
            // Create a fallback map with the config and the context
            let mut context_map = Context::from(&self.config.vars);
            context_map.extend_from(context);
            return match display::display_template(&template, &context_map) {
                Ok(_) => Ok(true),
                Err(e) => Err(Error::TemplateReadError {
                    file: path,
                    cause: e.to_string(),
                }),
            };
        } else {
            println!(
                "{}",
                Yellow.paint(format!("Template \"{}\" not found", name))
            );
        }

        Ok(true)
    }

    /// Creates a new template in the repository if not existing asking optionally
    /// the user using a confirmation prompt.
    ///
    /// # Arguments
    /// * `name` - The name of the template to create.
    /// * `prompt_user` - Whether to prompt the user for confirmation.
    ///
    /// # Returns
    /// Returns a tuple containing the template name, path, and a boolean indicating
    /// whether the template was created or not.
    fn create_new_template(
        &self,
        name: &str,
        prompt_user: bool,
    ) -> Result<(String, PathBuf, bool)> {
        let path = TemplateWriter::get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        let mut template_created = false;

        if !template.exists() {
            if prompt_user {
                loop {
                    let mut input = String::new();
                    print!(
                        "Template \"{}\" not found. Do you want to create it ? [Y/n] : ",
                        Yellow.paint(name)
                    );
                    let _ = stdout().flush();
                    stdin()
                        .read_line(&mut input)
                        .expect("error: unable to read user input");
                    input = input.trim().to_lowercase();
                    if input == "y" || input == "yes" || input.is_empty() {
                        break;
                    } else if input == "n" || input == "no" {
                        return Ok(("".to_owned(), PathBuf::new(), false));
                    }
                }
            }
            TemplateWriter::write_new(&template, self.config)?;
            template_created = true;
        }
        Ok((path, template, template_created))
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

        let template_payload = TemplateReader::read(&self.input_dir, template_name)?;
        // Create a fallback map with the template vars, config and context
        let mut full_context = Context::new();
        full_context.extend(&template_payload.vars);
        full_context.extend(&self.config.vars);
        full_context.extend_from(context);

        let formatted = TemplateFormatter::new(full_context).format(&template_payload)?;
        writeln!(std::io::stdout(), "{}", formatted)?;
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
        let template = self.input_dir.clone().join(&path);

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
                &self.input_dir,
            )?;
        }
        Ok(())
    }
}
