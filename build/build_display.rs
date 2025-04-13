use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;

use bincode;
use build_print::println as build_println;
use nu_ansi_term::Color::{Green, Red, Yellow};
use sublime_color_scheme::ColorScheme;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};
use syntect::LoadingError;

/// Extension trait for ThemeSet to add Sublime Text color scheme support
trait ThemeSetExt {
    /// Adds all Sublime Text color schemes from the given directory
    ///
    /// # Arguments
    /// * `dir` - The directory containing the color scheme files
    ///
    /// # Returns
    /// A `Result` indicating success or failure
    fn add_sublime_color_schemes(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Loads themes from the given directory with the specified extension
    ///
    /// # Arguments
    /// * `dir` - The directory to search for theme files
    /// * `extension` - The file extension to look for (e.g., "tmTheme" or "sublime-color-scheme")
    ///
    /// # Returns
    /// A `Result` indicating success or failure
    fn load_themes(
        &mut self,
        dir: &Path,
        extension: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Loads a theme from a .tmTheme file
    ///
    /// # Arguments
    /// * `dir` - The path to the .tmTheme file
    ///
    /// # Returns
    /// A `Result` indicating success or failure
    fn load_theme(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Loads a color scheme from a .sublime-color-scheme file
    ///
    /// # Arguments
    /// * `dir` - The path to the .sublime-color-scheme file
    ///
    /// # Returns
    /// A `Result` indicating success or failure
    fn load_color_scheme(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>>;
}

impl ThemeSetExt for ThemeSet {
    fn add_sublime_color_schemes(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        self.load_themes(dir, "tmTheme")?;
        self.load_themes(dir, "sublime-color-scheme")?;
        Ok(())
    }

    fn load_themes(
        &mut self,
        dir: &Path,
        extension: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.load_themes(&path, extension)?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == extension) {
                match extension {
                    "tmTheme" => self.load_theme(&path)?,
                    "sublime-color-scheme" => self.load_color_scheme(&path)?,
                    _ => {}
                };
            }
        }
        Ok(())
    }

    fn load_theme(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        build_println!(
            "Loading Sublime theme from {}",
            Green.paint(dir.display().to_string())
        );
        let theme = Self::get_theme(dir)?;
        let basename = dir
            .file_stem()
            .and_then(|x| x.to_str())
            .ok_or(LoadingError::BadPath)?;

        self.themes.insert(basename.to_owned(), theme);
        Ok(())
    }

    fn load_color_scheme(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        build_println!(
            "Loading Sublime color scheme from {}",
            Green.paint(dir.display().to_string())
        );

        let content = fs::read_to_string(&dir)?;
        if let Ok(color_scheme) = ColorScheme::from_str(&content) {
            let theme = Theme::try_from(color_scheme)?;
            if let Some(theme_name) = dir.file_stem().and_then(|s| s.to_str()) {
                self.themes.insert(theme_name.to_string(), theme);
            }
        }
        Ok(())
    }
}

/// Recursively loads all syntax definitions from a directory
///
/// This function traverses the given directory and its subdirectories to find
/// and load all syntax definition files.
///
/// # Arguments
/// * `dir` - The root directory to start searching for syntax definitions
///
/// # Returns
/// A `Result` containing the loaded `SyntaxSet` or an error
fn load_syntaxes(dir: &Path) -> Result<SyntaxSet, Box<dyn std::error::Error>> {
    let mut builder = SyntaxSetBuilder::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            builder.add_from_folder(&path, true)?;
        }
    }

    Ok(builder.build())
}

/// Loads all themes from the specified directory
///
/// This function loads both base themes and, if the `display-themes` feature is enabled,
/// extended themes from their respective subdirectories.
///
/// # Arguments
/// * `dir` - The root directory containing theme files
///
/// # Returns
/// A `Result` containing the loaded `ThemeSet` or an error
fn load_themes(dir: &Path) -> Result<ThemeSet, Box<dyn std::error::Error>> {
    let mut theme_set = ThemeSet::load_defaults();

    build_println!("{}", Yellow.paint("Loading base themes"));
    let base_dir = dir.join("base");
    if base_dir.exists() {
        theme_set.add_sublime_color_schemes(&base_dir)?;
    }

    #[cfg(feature = "display-themes")]
    {
        let extended_dir = dir.join("extended");
        if extended_dir.exists() {
            build_println!("{}", Yellow.paint("Loading extended themes"));
            theme_set.add_sublime_color_schemes(&extended_dir)?;
        }
    }

    Ok(theme_set)
}

/// Serializes data to a binary file
///
/// This function takes any serializable data and writes it to a binary file
/// in the specified output directory.
///
/// # Arguments
/// * `data` - The data to serialize
/// * `out_dir` - The output directory where the file will be written
/// * `filename` - The name of the file to create
///
/// # Returns
/// A `Result` indicating success or failure
fn serialize_to_file<T: serde::Serialize>(
    data: &T,
    out_dir: &Path,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = out_dir.join(filename);
    let serialized = bincode::serde::encode_to_vec(data, bincode::config::standard())?;
    fs::write(path, serialized)?;
    Ok(())
}

/// Main build function for the display feature
///
/// This function is responsible for:
/// 1. Creating the output directory
/// 2. Loading and serializing syntax definitions
/// 3. Loading and serializing themes
/// 4. Setting up cargo rerun-if-changed directives
///
/// # Returns
/// A `Result` indicating success or failure
pub fn build() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    fs::create_dir_all(out_dir).unwrap();

    let syntaxes_dir = Path::new("assets/syntaxes");
    if syntaxes_dir.exists() {
        match load_syntaxes(syntaxes_dir) {
            Ok(syntax_set) => {
                serialize_to_file(&syntax_set, out_dir, "syntax_set.bin").unwrap();
            }
            Err(e) => {
                build_println!(
                    "{}",
                    Red.paint(format!("Error: Failed to load syntaxes: {}", e))
                );
            }
        }
    }

    let themes_dir = Path::new("assets/themes");
    if themes_dir.exists() {
        match load_themes(themes_dir) {
            Ok(theme_set) => {
                serialize_to_file(&theme_set, out_dir, "theme_set.bin").unwrap();
            }
            Err(e) => build_println!(
                "{}",
                Red.paint(format!("Error: Failed to load themes: {}", e))
            ),
        }
    }

    println!("cargo:rerun-if-changed=assets/syntaxes");
    println!("cargo:rerun-if-changed=assets/themes");

    Ok(())
}
