#[cfg(feature = "bundler")]
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "bundler")]
use crate::template_bundle;
use crate::{
    config::MainConfig,
    constants::template::DEFAULT_TEMPLATE_EXT,
    context::Context,
    display,
    error::{Error, Result},
    formatter::TemplateFormatter,
    writer::TemplateWriter,
};

use crate::utils;
use serde_json::json;

#[cfg(feature = "fetcher")]
use crate::fetcher::TemplateFetcher;

#[cfg(feature = "display")]
use crate::theme::ThemeManager;

use glob::glob;
use nu_ansi_term::Color::{Green, Red, Yellow};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ListOutputFormat {
    Tree,
    Txt,
    Json,
}

impl ListOutputFormat {
    fn from_context(ctx: &Context) -> Self {
        match ctx.get("output") {
            Some("txt") => ListOutputFormat::Txt,
            Some("json") => ListOutputFormat::Json,
            _ => ListOutputFormat::Tree,
        }
    }
}

fn template_stem(file_name: &str) -> String {
    file_name
        .strip_suffix(DEFAULT_TEMPLATE_EXT)
        .unwrap_or(file_name)
        .to_string()
}

pub struct TemplatesController<'a> {
    pub input_dir: PathBuf,
    pub config: &'a MainConfig,
}

/// Provides all the operations involving templates management (list, open, create, edit, add (when "fetcher" feature enabled))
/// and formatting/rendering.
impl<'a> TemplatesController<'a> {
    #[must_use]
    pub fn new(input_dir: PathBuf, config: &'a MainConfig) -> Self {
        Self { input_dir, config }
    }

    /// Runs any of the templates subcommands. Currently supported :
    /// - list : shows the files stored in the templates repository
    /// - export / import : `.tpz` template bundles (`bundler` feature)
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
    ///
    /// # Errors
    /// Returns an error if the subcommand is missing, invalid, or an underlying operation fails.
    pub fn run_template_subcommand(&self, context: &Context) -> Result<bool> {
        match context.get("subcommand") {
            Some(cmd) => match cmd {
                "list" => self.list(context),
                #[cfg(feature = "bundler")]
                "export" => {
                    let out = match context.get("output") {
                        Some(p) => PathBuf::from(p),
                        None => template_bundle::default_export_path()?,
                    };
                    template_bundle::export_templates_dir(&self.input_dir, &out)?;
                    Ok(true)
                }
                #[cfg(feature = "bundler")]
                "import" => {
                    let bundle = context.get("bundle").ok_or_else(|| {
                        Error::CommandError("Missing .tpz bundle path".to_string())
                    })?;
                    template_bundle::import_bundle_to_templates_dir(
                        Path::new(bundle),
                        &self.input_dir,
                        context.is_active("force"),
                    )?;
                    Ok(true)
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

    /// Lists installed templates (default), or embedded themes when `--themes` is set (`display`).
    ///
    /// Honors `-o txt|json` for plain lines or JSON (`templates`, `themes` arrays respectively).
    ///
    /// # Arguments
    /// * `context` — Must include `subcommand=list`; optional `themes` flag and `output` format.
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use titular::{controller::TemplatesController, config::MainConfig, context::Context};
    ///
    /// let config = MainConfig::default();
    /// let input_dir = PathBuf::from("templates");
    /// let controller = TemplatesController::new(input_dir, &config);
    /// let mut context = Context::new();
    /// context.insert("subcommand", "list");
    ///
    /// let result = controller.list(&context);
    /// assert!(result.is_ok());
    /// ```
    pub fn list(&self, context: &Context) -> Result<bool> {
        let fmt = ListOutputFormat::from_context(context);
        #[cfg(feature = "display")]
        if context.is_active("themes") {
            return self.list_themes(fmt);
        }
        self.list_templates(fmt)
    }

    /// Lists the themes currently available in the binary.
    ///
    /// This function retrieves the list of themes from the themes binary and prints them to the console.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    /// Returns an error if themes cannot be loaded or listed.
    #[cfg(feature = "display")]
    fn list_themes(&self, fmt: ListOutputFormat) -> Result<bool> {
        let mgr = ThemeManager::init()?;
        match fmt {
            ListOutputFormat::Tree => {
                mgr.list_themes()?;
            }
            ListOutputFormat::Txt => {
                for name in mgr.theme_names_sorted() {
                    println!("{name}");
                }
            }
            ListOutputFormat::Json => {
                let names = mgr.theme_names_sorted();
                println!("{}", json!({ "themes": names }));
            }
        }
        Ok(true)
    }

    /// Lists the templates currently available in the templates repository.
    ///
    /// This function retrieves the list of templates from the templates repository and prints them to the console.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    /// Returns an error if the glob pattern is invalid or a matched path cannot be read.
    fn list_templates(&self, fmt: ListOutputFormat) -> Result<bool> {
        if self.input_dir.exists() {
            let pattern = format!(
                "{}{}{}",
                self.input_dir.to_string_lossy(),
                "/**/*",
                DEFAULT_TEMPLATE_EXT
            );
            let mut files = Vec::new();
            for entry in
                glob(&pattern).map_err(|e| Error::Msg(format!("Invalid glob pattern: {e}")))?
            {
                let path = entry.map_err(|e| Error::Msg(format!("Glob iteration error: {e}")))?;
                let name = path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                    Error::Msg("Non-UTF-8 or missing file name in template path".to_string())
                })?;
                files.push(name.to_string());
            }

            files.sort_by_key(|a| a.to_ascii_lowercase());

            let root = self.input_dir.to_string_lossy().to_string();
            match fmt {
                ListOutputFormat::Tree => {
                    utils::print_tree(&files, "template", &root);
                }
                ListOutputFormat::Txt => {
                    for f in &files {
                        println!("{}", template_stem(f));
                    }
                }
                ListOutputFormat::Json => {
                    let templates: Vec<String> = files.iter().map(|f| template_stem(f)).collect();
                    println!("{}", json!({ "templates": templates }));
                }
            }

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
    ///
    /// # Errors
    /// Returns an error if the template file cannot be written.
    pub fn create(&self, name: &str) -> Result<bool> {
        let (_, _, created) =
            TemplateWriter::create_new_template(name, false, &self.input_dir, self.config)?;
        if created {
            println!("New template \"{}\" created", Green.paint(name));
        } else {
            println!(
                "{}",
                Yellow.paint(format!("Template \"{name}\" already exists"))
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
    ///
    /// # Errors
    /// Returns an error if creating or opening the template fails.
    pub fn open(&self, name: &str) -> Result<bool> {
        let (path, template, _) =
            TemplateWriter::create_new_template(name, true, &self.input_dir, self.config)?;

        if path.is_empty() {
            Ok(true)
        } else {
            match edit::edit_file(&template) {
                Ok(()) => Ok(true),
                Err(e) => Err(Error::TemplateReadError {
                    file: path,
                    cause: e.to_string(),
                }),
            }
        }
    }

    /// Removes the template from the templates repository.
    ///
    /// # Arguments
    /// * `name` - The name of the template to remove.
    ///
    /// # Returns
    /// Returns `Ok(())` if the template was removed successfully, `Err(Error)` if the template does not exist.
    ///
    /// # Errors
    /// Returns an error if the template file cannot be removed.
    pub fn remove(&self, name: &str) -> Result<bool> {
        let path = TemplateWriter::get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        if template.exists() {
            match std::fs::remove_file(template) {
                Ok(()) => println!("Template \"{}\" removed", Green.paint(name)),
                Err(e) => {
                    return Err(Error::TemplateReadError {
                        file: path,
                        cause: e.to_string(),
                    });
                }
            }
        } else {
            println!("{}", Yellow.paint(format!("Template \"{name}\" not found")));
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
    ///
    /// # Errors
    /// Returns an error if the template cannot be read or rendered.
    pub fn display(&self, name: &str, context: &Context) -> Result<bool> {
        let path = TemplateWriter::get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        if template.exists() {
            // Create a fallback map with the config and the context
            let mut context_map = Context::from(&self.config.vars);
            context_map.append_from(context);
            return match display::display_template(&template, &context_map) {
                Ok(()) => Ok(true),
                Err(e) => Err(Error::TemplateReadError {
                    file: path,
                    cause: e.to_string(),
                }),
            };
        }
        println!("{}", Yellow.paint(format!("Template \"{name}\" not found")));
        Ok(true)
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
    ///
    /// # Errors
    /// Returns an error if preprocessing, reading, or rendering the template fails.
    pub fn format(&self, context: &Context, template_name: &str) -> Result<bool> {
        TemplateFormatter::new(&self.input_dir, self.config).format(context, template_name)
    }
}
