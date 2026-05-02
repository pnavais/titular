use std::io::IsTerminal;

use crate::{bootstrap::BootStrap, clap_app, env_cli};
use clap::{parser::ValueSource, ArgMatches};
use titular::{
    context::Context,
    controller::TemplatesController,
    error::{Error, Result},
    string_utils::unescape_cli_escapes,
};

pub struct App {
    pub matches: ArgMatches,
}

impl App {
    pub fn new() -> Self {
        #[cfg(windows)]
        let _ = nu_ansi_term::enable_ansi_support();

        let interactive_output = std::io::stdout().is_terminal();

        App {
            matches: Self::matches(interactive_output),
        }
    }

    pub fn matches(interactive_output: bool) -> ArgMatches {
        let mut iter = wild::args_os();
        let exe = iter.next().unwrap_or_default();
        let user_args: Vec<_> = iter.collect();
        let mut args = vec![exe];
        args.extend(env_cli::get_args_from_env_vars_filtered(&user_args));
        args.extend(user_args);
        clap_app::build_app(interactive_output)
            .get_matches_from(args)
    }

    /// Creates the context with the matched information supplied in the command
    /// line arguments. (e.g. template name, messages, fillers, vars, etc...)
    ///
    /// # Returns
    /// A `Result` containing the context.
    fn build_context(&self) -> Result<Context> {
        let mut context = Context::new();
        let interpret_escapes = self.matches.get_flag("interpret_escapes");

        context.insert(
            "template",
            self.matches
                .get_one::<String>("template")
                .map_or("", String::as_str),
        );
        #[cfg(feature = "display")]
        if let Some(theme) = self.matches.get_one::<String>("theme") {
            context.insert("theme", theme.as_str());
        }
        if self.matches.contains_id("message") {
            let messages: Vec<String> = self
                .matches
                .get_many::<String>("message")
                .unwrap()
                .map(|s| {
                    if interpret_escapes {
                        unescape_cli_escapes(s)
                    } else {
                        s.clone()
                    }
                })
                .collect();
            context.insert_multi("m", messages.iter().map(String::as_str).collect());
        }
        if self.matches.contains_id("filler") {
            let fillers: Vec<String> = self
                .matches
                .get_many::<String>("filler")
                .unwrap()
                .map(|s| {
                    if interpret_escapes {
                        unescape_cli_escapes(s)
                    } else {
                        s.clone()
                    }
                })
                .collect();
            context.insert_multi("f", fillers.iter().map(String::as_str).collect());
        }
        if self.matches.contains_id("color") {
            context.insert_multi(
                "c",
                self.matches
                    .get_many::<String>("color")
                    .unwrap()
                    .map(String::as_str)
                    .collect(),
            );
        }
        if self.matches.contains_id("set") {
            for v in self
                .matches
                .get_many::<String>("set")
                .unwrap()
                .map(String::as_str)
            {
                if let Some((key, value)) = v.split_once('=') {
                    context.insert(key, value);
                } else {
                    return Err(Error::ArgsProcessingError(format!(
                        "Invalid set parameter supplied \"{v}\" (Must be in key=value format)"
                    )));
                }
            }
        }
        if let Some(ValueSource::CommandLine) = self.matches.value_source("width") {
            context.insert(
                "width",
                self.matches
                    .get_one::<u8>("width")
                    .map(ToString::to_string)
                    .unwrap_or_default()
                    .as_str(),
            );
        }
        if self.matches.get_flag("no-newline") {
            context.insert("skip-newline", "true");
        }
        if self.matches.get_flag("with-time") {
            context.insert("with-time", "true");
        }
        if self.matches.get_flag("hide") {
            context.insert("hide", "true");
        }
        if self.matches.get_flag("clear") {
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
    fn add_params_to_context(context: &mut Context, tpl_params: &ArgMatches) {
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
                    let values_vec: Vec<&str> = values.map(String::as_str).collect();

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

        if let Some(("templates", tpl_params)) = self.matches.subcommand() {
            Self::add_params_to_context(&mut context, tpl_params);
            controller.run_template_subcommand(&context)?;
            return Ok(true);
        }

        let default_name = bootstrap.get_config().templates.default.as_str();
        let template_name = self
            .matches
            .get_one::<String>("template")
            .map_or(default_name, String::as_str);
        controller.format(&context, template_name)?;
        Ok(true)
    }
}
