#[cfg(feature = "display")]
use std::env;
#[cfg(feature = "display")]
use std::fs;
#[cfg(feature = "display")]
use std::path::Path;
#[cfg(feature = "display")]
use std::str::FromStr;

#[cfg(feature = "display")]
use bincode;
#[cfg(feature = "display")]
use build_print::println as build_println;
#[cfg(feature = "display")]
use nu_ansi_term::Color::{Green, Red, Yellow};
#[cfg(feature = "display")]
use sublime_color_scheme::ColorScheme;
#[cfg(feature = "display")]
use syntect::highlighting::{Theme, ThemeSet};
#[cfg(feature = "display")]
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};
#[cfg(feature = "display")]
use syntect::LoadingError;

/// Extension trait for ThemeSet to add Sublime Text color scheme support
#[cfg(feature = "display")]
trait ThemeSetExt {
    /// Adds all Sublime Text color schemes from the given directory
    fn add_sublime_color_schemes(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Loads themes from the given directory with the specified extension
    fn load_themes(
        &mut self,
        dir: &Path,
        extension: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Loads a theme from the given directory
    fn load_theme(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>>;

    /// Loads a theme from the given directory
    fn load_color_scheme(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>>;
}

#[cfg(feature = "display")]
impl ThemeSetExt for ThemeSet {
    /// Adds all Sublime Text color schemes from the given directory
    ///
    /// # Arguments
    /// * `dir` - The directory to load themes from
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    fn add_sublime_color_schemes(&mut self, dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Handle .tmTheme files
        self.load_themes(dir, "tmTheme")?;

        // Then handle .sublime-color-scheme files
        self.load_themes(dir, "sublime-color-scheme")?;

        Ok(())
    }

    /// Loads themes from the given directory with the specified extension
    ///
    /// # Arguments
    /// * `dir` - The directory to load themes from
    /// * `extension` - The extension of the themes to load
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
    fn load_themes(
        &mut self,
        dir: &Path,
        extension: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively process subdirectories
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

    /// Loads a theme from the given directory
    ///
    /// # Arguments
    /// * `dir` - The directory to load the theme from
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
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

    /// Loads a color scheme from the given directory
    ///
    /// # Arguments
    /// * `dir` - The directory to load the color scheme from
    ///
    /// # Returns
    /// A `Result` indicating success or failure.
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
/// # Arguments
/// * `dir` - The directory to load syntaxes from
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(feature = "display")]
fn load_syntaxes(dir: &Path) -> Result<SyntaxSet, Box<dyn std::error::Error>> {
    let mut builder = SyntaxSetBuilder::new();

    // Walk through all subdirectories
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively load syntaxes from subdirectories
            builder.add_from_folder(&path, true)?;
        }
    }

    Ok(builder.build())
}

/// Recursively loads all themes from a directory
///
/// # Arguments
/// * `dir` - The directory to load themes from
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(feature = "display")]
fn load_themes(dir: &Path) -> Result<ThemeSet, Box<dyn std::error::Error>> {
    let mut theme_set = ThemeSet::load_defaults();

    // Always load base themes
    build_println!("{}", Yellow.paint("Loading base themes"));
    let base_dir = dir.join("base");
    if base_dir.exists() {
        theme_set.add_sublime_color_schemes(&base_dir)?;
    }

    // Load extended themes if the feature is enabled
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
/// # Arguments
/// * `data` - The data to serialize
/// * `out_dir` - The output directory
/// * `filename` - The filename to serialize to
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(feature = "display")]
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

/// Main function for the build script. Currently only serializes syntaxes and themes.
///
/// # Returns
/// A `Result` indicating success or failure.
#[cfg(feature = "display")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    // Create the output directory if it doesn't exist
    fs::create_dir_all(out_dir).unwrap();

    // Load and serialize syntaxes
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

    // Load and serialize themes
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

    // Print rerun-if-changed directives
    println!("cargo:rerun-if-changed=assets/syntaxes");
    println!("cargo:rerun-if-changed=assets/themes");

    Ok(())
}
