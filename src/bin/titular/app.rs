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
    

    fn run_template_subcommand(&self, controller: TemplatesController, matches: &clap::ArgMatches) -> Result<()> {
        if matches.is_present("list") {
            controller.list();
        } else if matches.is_present("open") {
            controller.open(matches.subcommand_matches("open").unwrap().value_of("template").unwrap())?;
        } else if matches.is_present("create") {
            controller.create(matches.subcommand_matches("create").unwrap().value_of("template").unwrap())?;
        } else if matches.is_present("remove") {
            controller.remove(matches.subcommand_matches("remove").unwrap().value_of("template").unwrap())?;
        }

        Ok(())
    }

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
        if self.matches.is_present("n") {            
            context.insert("skip-newline", "true");
        }
        
        Ok(context)
    }

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
                controller.format(&context, &bootstrap.get_config(), template_name)?;
                Ok(true)                
            }
        }
    }
}

