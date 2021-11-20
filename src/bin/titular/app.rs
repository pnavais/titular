use crate::{
    clap_app,
    bootstrap::BootStrap,
};

use titular::{    
    error::*,
    templates::TemplatesController,
    context::Context,
};

use atty::{self, Stream};

use clap::ArgMatches;

pub struct App {
    pub matches: ArgMatches<'static>,
    pub interactive_output: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        #[cfg(windows)]
        let _ = ansi_term::enable_ansi_support();

        let interactive_output = atty::is(Stream::Stdout);

        Ok(App {
            matches: Self::matches(interactive_output)?,
            interactive_output: interactive_output,
        })
    }

    fn matches(interactive_output: bool) -> Result<ArgMatches<'static>> {
        Ok(clap_app::build_app(interactive_output).get_matches())
    }
    

    /// Runs any of the templates subcommands. Currently supported :
    /// - list : shows the files stored in the templates repository
    /// - edit : opens or creates if not existing the given template in the default system editor (see "edit" crate for more information)
    /// - create : creates a new template from sratch with a default template pattern
    /// - remove : deletes the given template from the templates repository
    /// - add (only when feature "fetcher" is enabled) : downloads and installs a template from the default templates remote repository
    ///   or a custom URL
    fn run_template_subcommand(&self, controller: TemplatesController, matches: &clap::ArgMatches) -> Result<()> {
        if matches.is_present("list") {
            controller.list();
        } else if matches.is_present("edit") {
            controller.open(matches.subcommand_matches("edit").unwrap().value_of("template").unwrap())?;
        } else if matches.is_present("create") {
            controller.create(matches.subcommand_matches("create").unwrap().value_of("template").unwrap())?;
        } else if matches.is_present("remove") {
            controller.remove(matches.subcommand_matches("remove").unwrap().value_of("template").unwrap())?;
        } 
        else if matches.is_present("add") {
            #[cfg(feature = "fetcher")]
            controller.add(&matches.subcommand_matches("add").unwrap().values_of("url").unwrap().map(|v| v.to_string()).collect())?;
        }

        Ok(())
    }

    /// Creates the context with the matched information supplied in the command 
    /// line arguments. (e.g. template name, messages, fillers, vars, etc...)
    fn build_context(&self) -> Result<Context> {
        let mut context = Context::new();

        context.insert("template", self.matches.value_of("template").or(Some("")).unwrap());
        if self.matches.is_present("message") {                        
            context.insert_multi("m", self.matches.values_of("message").unwrap().map(|v| v.to_string()).collect());
        }
        if self.matches.is_present("filler") {                        
            context.insert_multi("f", self.matches.values_of("filler").unwrap().map(|v| v.to_string()).collect());
        }
        if self.matches.is_present("key=value") {
            for v in self.matches.values_of("key=value").unwrap() {
                let k: Vec<&str> = v.split("=").collect();
                if k.len()>1 {
                    context.insert(k[0], k[1]);
                } else {
                    return Err(Error::ArgsProcessingError(format!("Invalid set parameter supplied \"{}\" (Must be in key=value format)", v)))
                }
            }            
        }
        if self.matches.is_present("width") {
            context.insert("width", self.matches.value_of("width").unwrap());
        }
        if self.matches.is_present("no-newline") {            
            context.insert("skip-newline", "true");
        }
        if self.matches.is_present("with-time") {
            context.insert("with-time", "true");
        }
        if self.matches.is_present("hide") {
            context.insert("hide", "true");
        }
        
        Ok(context)
    }

    /// Main entry point of the application, bootstraps the main configuration and creates the controller
    /// to process the templates rendering.
    pub fn start(&self) -> Result<bool> {        
        // Parse the default config
        let bootstrap = BootStrap::new()?;
        let controller = TemplatesController { input_dir:  bootstrap.template_dir()?, config: &bootstrap.get_config() };
        
        match self.matches.subcommand() {
            ("templates", Some(tpl_params)) => {
                self.run_template_subcommand(controller, tpl_params)?;
                Ok(true)
            }
            _ => { 
                let context = self.build_context()?;
                let template_name = self.matches.value_of("template").or(Some(&bootstrap.get_config().templates.default)).unwrap();                                                
                controller.format(&context, template_name)?;
                Ok(true)                
            }
        }
    }
}

