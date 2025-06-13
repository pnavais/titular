use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use std::io::IsTerminal;
use std::str::FromStr;

use crate::config::Display;
use crate::constants::template::DEFAULT_THEME;
use crate::context::Context;
use crate::error::*;

use pager::Pager;

#[cfg(feature = "display")]
use syntect::{
    easy::HighlightLines,
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

#[cfg(feature = "display")]
use crate::term::TERM_SIZE;
#[cfg(feature = "display")]
use crate::theme::ThemeManager;

use crate::utils::command_exists;

/// Setups the pager to display the content in a terminal.
///
/// Sets up a pager if the content exceeds terminal height and we're in a terminal.
/// Uses TITULAR_PAGER or BAT_PAGER environment variables if defined.
///
/// # Returns
/// Whenever the display feature is enabled, and the content is less than the terminal height,
/// the content is displayed in the terminal without a pager.
fn setup_pager() {
    if std::io::stdout().is_terminal() {
        // Set up pager with custom environment variable if defined
        let pager = env::var("TITULAR_PAGER")
            .or_else(|_| env::var("TITULAR_BAT"))
            .or_else(|_| env::var("BAT_PAGER"))
            .ok();

        if let Some(pager) = pager {
            Pager::with_pager(&pager).setup();
        } else {
            Pager::new().setup();
        }
    }
}

/// Checks if bat is available and sets the appropriate pager environment variable.
///
/// # Arguments
/// * `context` - The context to use for the template
/// * `path` - The path to the template file
///
/// # Returns
/// A `Result` indicating success or failure.
fn check_pager(context: &Context, path: &Path) -> Result<()> {
    let display = Display::from_str(
        context
            .get("mode")
            .or_else(|| context.get("defaults.display"))
            .unwrap_or(&"raw".to_string()),
    )?;

    match display {
        Display::Raw => {}
        Display::Pager => {
            setup_pager();
        }
        Display::Bat | Display::BatOrPager => {
            if command_exists("bat") {
                let env_var = match display {
                    Display::Bat => "TITULAR_PAGER",
                    Display::BatOrPager => "TITULAR_BAT",
                    _ => unreachable!(),
                };
                env::set_var(
                    env_var,
                    format!("bat -l toml --file-name {}", path.display()),
                );
            }
            setup_pager();
        }
        #[cfg(feature = "display")]
        Display::Fancy => {}
    }

    Ok(())
}

/// Displays the content with syntax highlighting using syntect.
///
/// # Arguments
/// * `content` - The content to display with syntax highlighting
/// * `context` - The context to use for the template
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(feature = "display")]
fn display_fancy(content: &str, context: &Context) -> Result<()> {
    // Load the serialized syntax set from the build script
    let syntax_set_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/syntax_set.bin"));
    let syntax_set: SyntaxSet =
        bincode::serde::decode_from_slice(syntax_set_bytes, bincode::config::standard())
            .unwrap()
            .0;

    // Load the serialized theme set from the build script
    let theme_manager = ThemeManager::init()?;

    // Theme selection chain:
    // 1. Try to get theme from context
    // 2. Fallback to defaults.display_theme
    // 3. Finally use DEFAULT_THEME
    let theme_name = context
        .get("theme")
        .or_else(|| context.get("defaults.display_theme"))
        .map(|s| s as &str)
        .unwrap_or(DEFAULT_THEME);

    let theme = theme_manager
        .theme_set
        .themes
        .get(theme_name)
        .unwrap_or(theme_manager.get_theme(DEFAULT_THEME));

    // Find the TOML syntax
    let syntax = syntax_set
        .find_syntax_by_extension("toml")
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());

    // Create a highlighter with the default theme
    let mut h = HighlightLines::new(syntax, theme);

    // Highlight and print each line
    for line in LinesWithEndings::from(content) {
        let regions = h.highlight_line(line, &syntax_set).unwrap();
        let escaped = as_24_bit_terminal_escaped(&regions[..], false);
        print!("{}", escaped);
    }

    Ok(())
}

/// Displays the contents of a template file with syntax highlighting and pager support.
///
/// # Arguments
/// * `path` - The path to the template file
/// * `context` - The context to use for the template
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(feature = "display")]
pub fn display_template(path: &Path, context: &Context) -> Result<()> {
    // Load the template content
    let content = fs::read_to_string(path)?;
    let display = Display::from_str(
        context
            .get("mode")
            .or_else(|| context.get("defaults.display"))
            .unwrap_or(&"raw".to_string()),
    )?;

    // Setup pager if needed
    if !matches!(display, Display::Fancy) || content.lines().count() > TERM_SIZE.get_term_height() {
        check_pager(context, path)?;
    }

    // Display content based on display type
    match display {
        Display::Fancy => display_fancy(&content, context)?,
        _ => writeln!(io::stdout().lock(), "{}", content)?,
    }

    Ok(())
}

/// Displays the contents of a template file reading
/// its contents from the file system.
///
/// # Arguments
/// * `path` - The path to the template file
/// * `context` - The context to use for the template
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(not(feature = "display"))]
pub fn display_template(path: &Path, context: &Context) -> Result<()> {
    let content = fs::read_to_string(path)?;

    check_pager(context, path)?;

    writeln!(io::stdout().lock(), "{}", content)?;

    Ok(())
}
