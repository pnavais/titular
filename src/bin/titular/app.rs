use std::io::IsTerminal;

use crate::{bootstrap::BootStrap, clap_app};
use clap::{parser::ValueSource, ArgMatches};
use titular::{context::Context, controller::TemplatesController, error::*};

pub struct App {
    pub matches: ArgMatches,
}

impl App {
    pub fn new() -> Result<Self> {
        #[cfg(windows)]
        let _ = nu_ansi_term::enable_ansi_support();

        let interactive_output = std::io::stdout().is_terminal();

        Ok(App {
            matches: Self::matches(interactive_output)?,
        })
    }

    pub fn matches(interactive_output: bool) -> Result<ArgMatches> {
        Ok(clap_app::build_app(interactive_output).get_matches())
    }

    /// Creates the context with the matched information supplied in the command
    /// line arguments. (e.g. template name, messages, fillers, vars, etc...)
    ///
    /// # Returns
    /// A `Result` containing the context.
    fn build_context(&self) -> Result<Context> {
        let mut context = Context::new();

        context.insert(
            "template",
            self.matches
                .get_one::<String>("template")
                .map(|s| s.as_str())
                .unwrap_or(""),
        );
        if self.matches.contains_id("message") {
            context.insert_multi(
                "m",
                self.matches
                    .get_many::<String>("message")
                    .unwrap()
                    .map(|s| s.as_str())
                    .collect(),
            );
        }
        if self.matches.contains_id("filler") {
            context.insert_multi(
                "f",
                self.matches
                    .get_many::<String>("filler")
                    .unwrap()
                    .map(|s| s.as_str())
                    .collect(),
            );
        }
        if self.matches.contains_id("color") {
            context.insert_multi(
                "c",
                self.matches
                    .get_many::<String>("color")
                    .unwrap()
                    .map(|s| s.as_str())
                    .collect(),
            );
        }
        if self.matches.contains_id("set") {
            for v in self
                .matches
                .get_many::<String>("set")
                .unwrap()
                .map(|s| s.as_str())
            {
                let k: Vec<&str> = v.split('=').collect();
                if k.len() > 1 {
                    context.insert(k[0], k[1]);
                } else {
                    return Err(Error::ArgsProcessingError(format!(
                        "Invalid set parameter supplied \"{}\" (Must be in key=value format)",
                        v
                    )));
                }
            }
        }
        if self.matches.contains_id("width") {
            context.insert(
                "width",
                self.matches
                    .get_one::<u8>("width")
                    .map(|w| w.to_string())
                    .unwrap_or_default()
                    .as_str(),
            );
        }
        if self.matches.get_flag("no-newline") {
            context.insert("skip-newline", "true");
        }
        if self.matches.contains_id("with-time") {
            context.insert("with-time", "true");
        }
        if self.matches.contains_id("hide") {
            context.insert("hide", "true");
        }
        if self.matches.contains_id("clear") {
            context.insert("clear", "true");
        }

        Ok(context)
    }

    /// Add parameters to the context based on the template parameters.
    ///
    /// This function takes a mutable reference to a `Context` and a reference to an `ArgMatches` object.
    /// It iterates over the subcommands and arguments provided by the `ArgMatches` object and inserts them into the `Context`.
    ///
    /// # Arguments
    ///
    /// * `context` - A mutable reference to a `Context` object.
    /// * `tpl_params` - A reference to an `ArgMatches` object.
    ///
    /// # Returns
    ///
    /// This function returns nothing.
    fn add_params_to_context(&self, context: &mut Context, tpl_params: &ArgMatches) {
        let (id, args) = tpl_params.subcommand().unwrap();
        context.insert("subcommand", id);

        for arg_id in args.ids() {
            let arg_name = arg_id.as_str();

            if let Some(ValueSource::CommandLine) = args.value_source(arg_name) {
                // Handle flags
                if let Ok(Some(flag)) = args.try_get_one::<bool>(arg_name) {
                    context.insert(arg_name, flag.to_string().as_str());
                    continue;
                }

                // Handle single values
                if let Ok(Some(value)) = args.try_get_one::<String>(arg_name) {
                    context.insert(arg_name, value.as_str());
                    continue;
                }

                // Handle multiple values
                if let Ok(Some(values)) = args.try_get_many::<String>(arg_name) {
                    let values_vec: Vec<&str> = values.map(|v| v.as_str()).collect();

                    if !values_vec.is_empty() {
                        // If there are multiple values, add them as a multi-value entry
                        context.insert_multi(arg_name, values_vec);
                    }
                }
            }
        }
    }

    /// Start the application, bootstraps the configuration and forwards the request to the controller.
    ///
    /// When formatting a template, the application will create a context and pass it automatically.
    /// It will also translate command line arguments into context variables.
    ///
    /// # Returns
    /// A `Result` indicating whether the application started successfully.
    pub fn start(&self) -> Result<bool> {
        // Parse the default config
        let bootstrap = BootStrap::new()?;
        let controller =
            TemplatesController::new(bootstrap.template_dir()?, bootstrap.get_config());

        let mut context = self.build_context()?;

        match self.matches.subcommand() {
            Some(("templates", tpl_params)) => {
                self.add_params_to_context(&mut context, tpl_params);
                controller.run_template_subcommand(&context)?;
                Ok(true)
            }
            _ => {
                let template_name = self
                    .matches
                    .get_one::<String>("template")
                    .map(|s| s.as_str())
                    .or_else(|| Some(&bootstrap.get_config().templates.default))
                    .unwrap();
                controller.format(&context, template_name)?;
                Ok(true)
            }
        }
    }
}
