
use clap::{
    crate_description, crate_name, crate_version, App as ClapApp, AppSettings, Arg, SubCommand, 
};
use std::env;

pub fn build_app(interactive_output: bool) -> ClapApp<'static, 'static> {

    let clap_color_setting = if interactive_output && env::var_os("NO_COLOR").is_none() {
        AppSettings::ColoredHelp
    } else {
        AppSettings::ColorNever
    };

    let mut app = ClapApp::new(crate_name!())
        .version(crate_version!())
        .global_setting(clap_color_setting)
        .global_setting(AppSettings::DeriveDisplayOrder)
        .global_setting(AppSettings::UnifiedHelpMessage)
        .global_setting(AppSettings::HidePossibleValuesInHelp)
        .setting(AppSettings::ArgsNegateSubcommands)
        .setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::DisableHelpSubcommand)
        .setting(AppSettings::VersionlessSubcommands)
        .max_term_width(100)
        .about(concat!(crate_description!(),"\n\nUse '--help' instead of '-h' to see a more detailed version of the help text."))
        .long_about(crate_description!())
        .arg(
            Arg::with_name("template")
            .short("t")
            .long("template")
            .takes_value(true)
            .help("Specifies the template to use.")
            .long_help("Explicitly specify the template to display. \
                        When specifying a non-existing template or \
                        containing errors an according error message will be displayed.")
        )
        .arg(
            Arg::with_name("message")
                .short("m")
                .long("message")
                .takes_value(true)
                .multiple(true)
                .help("Sets the message in the title used.")
                .long_help("Explicitly sets the text messages to use in the pattern. \
                    When specifying multiple text options, \
                    the texts will be replaced following the same occurrence order (m2, m3, ...).")
        )
        .arg(
            Arg::with_name("filler")
            .short("f")
            .long("filler")
            .takes_value(true)
            .multiple(true)
            .help("Specifies the text used as filler.")
            .long_help("Explicitly specify the filler characters to use in the pattern. \
                        If not specified, the default filler specified in the pattern will be used. \
                        When specifying multiple filler options, \
                        the latter will be assigned following the same occurrence order (f2, f3, ...).")
        )
        .arg(
            Arg::with_name("key=value")
            .short("s")
            .long("set")
            .takes_value(true)
            .multiple(true)            
            .help("Sets the value of a variable (key=value).")
            .long_help("Specifies the value of a given variable used in the pattern by supplying \
                        the name of the variable and the associated value in (key=value) format.")
        )
        .arg(
            Arg::with_name("width")
            .short("w")
            .long("width")
            .takes_value(true)
            .help("Specifies the maximum % width of the terminal to use.")
            .long_help("Explicitly specify the width percentage (%) of the maxium width \
                        of the terminal to use (defaults to 100%).")
        )
        .arg(
            Arg::with_name("with-time")
            .long("with-time")
            .help("Adds a trailing timestamp.")
            .long_help("Adds a timestamp to the end of the pattern using the time format
                        configured in the settings (defaults to : [%H:%M:%S].")
        ).arg(
            Arg::with_name("n")
            .short("n")
            .long("no-newline")            
            .help("Supress new line after the generated title.")
            .long_help("Prevents writing a carriage return after generating the title.")
        );

        
        let mut templates_subcmd =  SubCommand::with_name("templates")
                .about("Modify the templates configuration")         
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(              
                    SubCommand::with_name("list")
                        .alias("ls")
                        .help("List the current installed templates.")                        
                        .about(
                            "Displays the currently installed templates from \
                            the templates directory (default: the templates folder inside configuration directory).",
                        )
                )
                .subcommand(
                    SubCommand::with_name("create")
                        .alias("new")                      
                        .arg(Arg::with_name("template")
                            .required(true)
                            .takes_value(true)
                            .help("The name of template to create")
                            .index(1))                                            
                        .help("Creates a new template with the given name.")
                        .about(
                            "Creates the given template in \
                            the templates directory (default: the templates folder inside configuration directory).\
                            The template will be created using the default \
                            template structure.",
                        ),
                )
                .subcommand(
                    SubCommand::with_name("edit")
                        .arg(Arg::with_name("template")
                            .required(true)
                            .takes_value(true)
                            .help("The name of template to open")
                            .index(1))                                            
                        .help("Opens the selected installed template.")
                        .about(
                            "Opens the selected templates from \
                            the templates directory (default: the templates folder inside configuration directory) \
                            in the platform's default text editor. If the template does not exist the user is prompted \
                            to create a new one using a default template structure.",
                        ),
                );

        templates_subcmd = templates_subcmd.subcommand(
                    SubCommand::with_name("remove")
                        .alias("rm")
                        .arg(Arg::with_name("template")
                            .required(true)
                            .takes_value(true)
                            .help("The name of template to remove")
                            .index(1))                                            
                        .help("Removes the template with the given name.")
                        .about(
                            "Removes the given template in \
                            the templates directory (default: the templates folder inside configuration directory).",
                        ),
        );
        
        #[cfg(feature = "fetcher")]
        {
            templates_subcmd = templates_subcmd.subcommand(
                SubCommand::with_name("add")
                    .arg(Arg::with_name("url")
                        .required(true)
                        .takes_value(true)
                        .multiple(true)
                        .help("The URL of template to add")
                        .index(1))                                            
                    .help("Downloads & install the template from the given URL.")
                    .about(
                        "Downloads the template in the specified URL and installs it in \
                        the templates directory (default: the templates folder inside configuration directory).",
                    ),
            );
        }

        app = app.subcommand(templates_subcmd);

        app
}