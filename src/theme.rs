use syntect::highlighting::{Theme, ThemeSet};

use crate::{error::Result, utils};
use nu_ansi_term::Color::{Green, Yellow};
pub struct ThemeManager {
    pub theme_set: ThemeSet,
}

impl ThemeManager {
    /// Loads built-in themes from the binary assets produced by the build script.
    ///
    /// # Errors
    /// Currently always returns `Ok`; reserved for future fallible initialization.
    pub fn init() -> Result<Self> {
        Ok(Self {
            theme_set: Self::load_themes(),
        })
    }

    ///
    /// This function loads the themes from the build script and returns them as a `ThemeSet`.
    ///
    /// # Returns
    /// The loaded `ThemeSet`.
    ///
    /// # Panics
    /// Panics if the embedded theme blob is corrupt or incompatible with the current `bincode` schema.
    fn load_themes() -> ThemeSet {
        // Load the serialized theme set from the build script
        let theme_set_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/theme_set.bin"));
        bincode::serde::decode_from_slice(theme_set_bytes, bincode::config::standard())
            .expect("theme_set.bin from build script should decode")
            .0
    }

    /// Lists the themes currently available in the binary.
    ///
    /// This function lists the themes currently available in the binary.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    ///
    /// # Errors
    /// Currently always returns `Ok(())`; reserved for future fallible output paths.
    pub fn list_themes(&self) -> Result<()> {
        let themes: Vec<&str> = self.theme_set.themes.keys().map(std::string::String::as_str).collect();
        utils::print_tree_with_prefixes(
            &themes,
            "theme",
            "Available themes",
            "\u{e22b}",
            "\u{f08b5}",
            |s| Yellow.paint(s).to_string(),
            |s| Green.paint(s).to_string(),
        );
        Ok(())
    }

    /// Gets a theme from the theme set.
    ///
    /// # Arguments
    /// * `theme_name` - The name of the theme to get.
    ///
    /// # Returns
    /// A `Result` indicating success or failure of the operation.
    #[must_use] 
    pub fn get_theme(&self, theme_name: &str) -> &Theme {
        &self.theme_set.themes[theme_name]
    }
}
