use crate::{
    config::{parse as config_parse, TemplateConfig, DEFAULT_TEMPLATE_EXT},
    error::*,
    log,
};

use nu_ansi_term::Color::Yellow;

use std::path::PathBuf;

pub struct TemplateReader {}

impl TemplateReader {
    /// Read the template configuration from a file.
    ///
    /// # Arguments
    /// * `input_dir` - The input directory.
    /// * `template_name` - The template name.
    ///
    /// # Returns
    /// The template configuration.
    ///
    /// # Errors
    /// Returns an error if the template file cannot be read or parsed.
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use std::fs;
    /// use titular::reader::TemplateReader;
    ///
    /// // Create a temporary directory for testing
    /// let temp_dir = tempfile::tempdir().unwrap();
    /// let input_dir = temp_dir.path().to_path_buf();
    /// fs::create_dir_all(&input_dir).unwrap();
    ///
    /// // Create a test template file
    /// let template_content = r#"
    /// [details]
    /// name = "test"
    /// description = "Test template"
    ///
    /// [pattern]
    /// data = "test pattern"
    /// "#;
    /// fs::write(input_dir.join("test.tl"), template_content).unwrap();
    ///
    /// let template_config = TemplateReader::read(&input_dir, "test");
    /// assert!(template_config.is_ok());
    /// ```
    pub fn read(input_dir: &PathBuf, template_name: &str) -> Result<TemplateConfig> {
        let template_path = TemplateReader::get_template_path(input_dir, template_name)?;

        TemplateReader::parse_data(&template_path, template_name)
    }

    /// Read the template configuration from a file.
    ///
    /// # Arguments
    /// * `template_file` - The path to the template file.
    ///
    /// # Returns
    /// The template configuration.
    ///
    /// # Errors
    /// Returns an error if the template file cannot be read or parsed.
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use std::fs;
    /// use titular::reader::TemplateReader;
    ///
    /// // Create a temporary directory for testing
    /// let temp_dir = tempfile::tempdir().unwrap();
    /// let template_file = temp_dir.path().join("test.tl");
    ///
    /// // Create a test template file
    /// let template_content = r#"
    /// [details]
    /// name = "test"
    /// description = "Test template"
    ///
    /// [pattern]
    /// data = "test pattern"
    /// "#;
    /// fs::write(&template_file, template_content).unwrap();
    ///
    /// let template_config = TemplateReader::read_file(&template_file);
    /// assert!(template_config.is_ok());
    /// ```
    pub fn read_file(template_file: &PathBuf) -> Result<TemplateConfig> {
        TemplateReader::parse_data(template_file, "unknown")
    }

    /// Get the template name from the template file.
    ///
    /// This function takes a path to a template file and returns the template name.
    /// It uses the existing `parse_data` method to parse the template file and extract the template name.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the template file.
    ///
    /// # Returns
    ///
    /// The template name.
    ///
    /// # Errors
    ///
    /// Returns an error if the template file cannot be read or parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use std::fs;
    /// use titular::reader::TemplateReader;
    ///
    /// // Create a temporary directory for testing
    /// let temp_dir = tempfile::tempdir().unwrap();
    /// let template_file = temp_dir.path().join("test.tl");
    ///
    /// // Create a test template file
    /// let template_content = r#"
    /// [details]
    /// name = "test"
    /// description = "Test template"
    ///
    /// [pattern]
    /// data = "test pattern"
    /// "#;
    /// fs::write(&template_file, template_content).unwrap();
    ///
    /// let template_name = TemplateReader::get_template_name(&template_file);
    /// assert!(template_name.is_ok());
    /// assert_eq!(template_name.unwrap(), "test");
    /// ```
    pub fn get_template_name(path: &PathBuf) -> Result<String> {
        Self::parse_data(&path, "unknown").map(|config| config.details.name)
    }

    /// Get the path to the template file.
    ///
    /// This function takes an input directory and a template name, and returns the path to the template file.
    /// In case the template name points to an actual file, it returns the path to that file, otherwise it tries
    /// to look for the template in the templates directory by normalizing the template name.
    ///
    /// # Arguments
    ///
    /// * `input_dir` - The input directory.
    /// * `template_name` - The template name.
    ///
    /// # Returns
    ///
    /// The path to the template file.
    fn get_template_path(input_dir: &PathBuf, template_name: &str) -> Result<PathBuf> {
        // Normalize the template name by adding .tl extension if needed
        let normalized_name = if template_name.ends_with(DEFAULT_TEMPLATE_EXT) {
            template_name.to_string()
        } else {
            template_name.to_string() + DEFAULT_TEMPLATE_EXT
        };

        // Join the input directory with the normalized template name
        let template_path = input_dir.join(&normalized_name);
        Ok(template_path)
    }

    /// Parses the template data from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the template file.
    /// * `name` - The name of the template.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the parsed `TemplateConfig` or an error.
    ///
    /// # Errors
    ///
    /// Returns an error if the template file is not found or cannot be read.
    fn parse_data(template_path: &PathBuf, template_name: &str) -> Result<TemplateConfig> {
        // Read the template file
        let toml_data = match config_parse(template_path) {
            Ok(data) => data,
            Err(Error::Io(e)) if e.kind() == ::std::io::ErrorKind::NotFound => {
                return Err(Error::TemplateNotFound {
                    file: template_name.to_string(),
                    cause: log::debug_message(
                        e.to_string(),
                        Yellow
                            .paint(format!(
                                "\n[Template path] {}",
                                template_path.to_string_lossy().to_string()
                            ))
                            .to_string(),
                    ),
                });
            }
            Err(Error::Io(e)) => {
                return Err(Error::TemplateReadError {
                    file: template_name.to_string(),
                    cause: e.to_string(),
                });
            }
            Err(e) => return Err(e),
        };

        // Parse the TOML data into a TemplateConfig
        match toml::from_str::<TemplateConfig>(&toml_data) {
            Ok(config) => Ok(config),
            Err(e) => Err(Error::SerdeTomlError {
                location: ConfigType::TEMPLATE,
                file: template_path.to_string_lossy().to_string(),
                cause: log::debug_message(
                    e.to_string(),
                    Yellow
                        .paint(format!(
                            "\n[Template path]: {}",
                            template_path.to_string_lossy().to_string()
                        ))
                        .to_string(),
                ),
            }),
        }
    }
}
