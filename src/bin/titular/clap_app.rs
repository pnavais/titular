use clap::{
    arg,
    builder::{styling::AnsiColor, Styles},
    crate_description, crate_name, crate_version, value_parser, Arg, ArgAction, ColorChoice,
    Command,
};

fn env_no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|x| !x.is_empty())
}

// Builds the application command line interface defining the commands, subcommands
// and arguments
pub fn build_app(interactive_output: bool) -> Command {
    let color_when = if interactive_output && !env_no_color() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    };

    let styles = Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Green.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default());

    let mut app = Command::new(crate_name!())
    .styles(styles)
    .version(crate_version!())
    .about(crate_description!())
    .color(color_when)
    .allow_hyphen_values(true)
    .arg(
        arg!(-t --template <VALUE> "Template to use for the title")
        .long_help(
            "Template to be rendered with the custom message. Must match a name \
                    inside the templates directory ($TITULAR_TEMPLATE_DIR).",
        ),
    )
    .arg(
        arg!(-m --message <VALUE> ... "Sets the message in the title used.")
        .long_help(
            "Explicitly sets the text messages to use in the pattern. \
                    When specifying multiple text options, \
                    the texts will be replaced following the same occurrence order (m2, m3, ...).",
        ),
    )
    .arg(
        arg!(-f --filler <VALUE> ... "Specifies the text used as filler.")
        .long_help(
            "Explicitly specify the filler characters to use in the pattern. \
                    If not specified, the default filler specified in the pattern will be used. \
                    When specifying multiple filler options, \
                    the latter will be assigned following the same occurrence order (f2, f3, ...).",
        ),
    )
    .arg(
        arg!(-c --color <VALUE> ... "Specifies the color to use in the pattern.")
        .long_help(
            "Explicitly specify the color to use in the pattern. \
                    If not specified, the default color specified in the pattern will be used. \
                    When specifying multiple color options, \
                    the latter will be assigned following the same occurrence order (c2, c3, ...).",
        ),
    )
    .arg(
        arg!(-s --set <VALUE> ... "Sets the value of a variable (key=value).")
        .long_help(
            "Specifies the value of a given variable used in the pattern by supplying \
                    the name of the variable and the associated value in (key=value) format.",
        ),
    )
    .arg(
        arg!(-w --width <VALUE> "Specifies the maximum % width of the terminal to use.")
        .long_help(
            "Explicitly specify the width percentage (%) of the maxium width \
                    of the terminal to use (defaults to 100%).",
        )
        .value_parser(value_parser!(u8).range(0..=100))
        .default_value("100"),
    )
    .arg(
        arg!(--"with-time" "Adds a trailing timestamp.")
        .long_help("Adds a timestamp to the end of the pattern using the time format
                    configured in the settings (defaults to : [%H:%M:%S].")
    ).arg(
        arg!(-n --"no-newline" "Supress new line after the generated title.")
        .long_help("Prevents writing a carriage return after generating the title.")
    ).arg(
        arg!(--hide "Hide all items flagged as invisible in the pattern.")
        .long_help("Prevents writing the items flagged as invisible but taking into account their width for padding purposes.")
    ).arg(
        arg!(--clear "Clears the current line and moves the cursor at the beginning.")
        .long_help("Erases the entire line the cursor is currently on then moves the cursor to the beginning of the line.")
    );

    // Add the templates subcommand
    app = app.subcommand(configure_subcommands());

    app
}

/// Configure the templates subcommands
fn configure_subcommands() -> Command {
    let templates_subcmd = Command::new("templates")
    .about("Modify the templates configuration")
    .arg_required_else_help(true)
    .subcommand(build_list_command())
    .subcommand(
        Command::new("create")
        .alias("new")
        .arg(arg!(<template> "The name of template to create"))
        .arg(arg!(-n --"no-edit" "Skip opening the editor after creation"))
        .about("Creates a new template with the given name.")
        .long_about(
            "Creates the given template in \
                    the templates directory (default: the templates folder inside configuration directory).\
                    The template will be created using the default \
                    template structure.",
        ),
    )
    .subcommand(
        Command::new("edit")
        .arg(arg!(<template> "The name of template to edit"))
        .about("Opens the selected installed template.")
        .long_about(
            "Opens the selected templates from \
                    the templates directory (default: the templates folder inside configuration directory) \
                    in the platform's default text editor. If the template does not exist the user is prompted \
                    to create a new one using a default template structure.",
        ),
    )
    .subcommand(build_show_command())
    .subcommand(
        Command::new("remove")
        .alias("rm")
        .arg(Arg::new("template")
        .required(true)
        .action(ArgAction::Set)
        .help("The name of template to remove")
        .index(1))
        .about("Removes the template with the given name.")
        .long_about(
            "Removes the given template in \
                    the templates directory (default: the templates folder inside configuration directory).",
        ),
    );

    #[cfg(feature = "fetcher")]
    let templates_subcmd = templates_subcmd.subcommand(
        Command::new("add")
        .arg(arg!(<url> "The URL of thetemplate to add"))
        .arg(arg!(-f --force "Overrides existing template"))
        .about("Downloads & install the template from the given URL.")
        .long_about(
            "Downloads the template in the specified URL and installs it in \
                    the templates directory (default: the templates folder inside configuration directory).\
                    In case of a github URL, the shortcut <repo_user>/<repo_name>:<path/to/template> can be specified.",
        ),
    );

    templates_subcmd
}

/// Builds the list command with optional themes argument when display feature is enabled
///
/// # Returns
/// A `Command` object representing the list command.
fn build_list_command() -> Command {
    let list_cmd = Command::new("list")
    .alias("ls")
    .about("List the current installed templates.")
    .long_about(
        "Displays the currently installed templates from \
            the templates directory (default: the templates folder inside configuration directory).",
    );

    #[cfg(feature = "display")]
    {
        let mut cmd = list_cmd;
        cmd = cmd.arg(
            arg!(-t --themes "List available syntax highlighting themes")
            .long_help("Displays all available syntax highlighting themes that can be used with the display feature."),
        );
        cmd
    }
    #[cfg(not(feature = "display"))]
    {
        list_cmd
    }
}

/// Builds the show command with optional themes argument when display feature is enabled
///
/// # Returns
/// A `Command` object representing the show command.
fn build_show_command() -> Command {
    let show_cmd = Command::new("show")
    .arg(arg!(<template> "The name of template to display"))
    .about("Displays the contents of the selected installed template.")
    .long_about(
        "Displays the selected templates from \
                    the templates directory (default: the templates folder inside configuration directory) \
                    using the platform's default pager.",
    );

    let mut cmd = show_cmd;
    cmd = cmd.arg(
        arg!(-m --mode <VALUE> "Sets the display mode")
            .long_help("Explicitly specify the display mode to use."),
    );

    #[cfg(feature = "display")]
    {
        cmd = cmd.arg(
            arg!(-t --theme <VALUE> "Sets the syntax highlighting theme")
                .long_help("Explicitly specify the syntax highlighting theme to use."),
        );
    }
    cmd
}
