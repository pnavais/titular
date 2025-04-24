use chrono::prelude::*;
use std::io::prelude::*;
use std::path::PathBuf;
use std::{env, fs::File};
use titular::config::DEFAULT_TEMPLATE_NAME;

pub use titular::{
    config::{parse as config_parse, MainConfig},
    error::*,
};

#[cfg(feature = "fetcher")]
use titular::config::DEFAULT_REMOTE_REPO;

use crate::directories::PROJECT_DIRS;

static DEFAULT_CONF: &str = "# File automatically generated on ${date}\n\
                            [defaults]\n\
                            fill_char      = \"*\"\n\
                            width          = \"full\"\n\
                            surround_start = \"[\"\n\
                            surround_end   = \"]\"\n\
                            time_pattern   = \"${space}%{time:fg[tc]}\"\n\
                            time_format    = \"%H:%M:%S\"\n\
                            display        = \"${default_pager}\"\n\n\
                            [vars]\n\
                            steel_blue   = \"RGB(70, 130, 180)\"\n\
                            light_purple = \"FIXED(134)\"\n\
                            red          = \"NAME(Red)\"\n\
                            green        = \"NAME(Green)\"\n\
                            yellow       = \"NAME(Yellow)\"\n\
                            blue         = \"NAME(Blue)\"\n\
                            orange       = \"RGB(255,165,0)\"\n\
                            space        = \" \"\n\n\
                            [templates]\n\
                            directory    = \"${templates_dir}\"\n\
                            default      = \"${default_template_name}\"\n";

const DEFAULT_CONF_FILE: &str = "titular.toml";

#[derive(Debug)]
pub struct BootStrap {
    config: MainConfig,
}

impl BootStrap {
    pub fn new() -> Result<Self> {
        Ok(BootStrap {
            config: BootStrap::init()?,
        })
    }

    /// Initializes the application by setting up necessary handlers and configurations.
    /// Currently sets up the Ctrl+C handler to restore cursor visibility when the program is interrupted.
    ///
    /// # Returns
    /// A `Result` containing the main configuration.
    pub fn init() -> Result<MainConfig> {
        #[cfg(feature = "fetcher")]
        {
            if let Err(e) = ctrlc::set_handler(titular::utils::cleanup) {
                return Err(Error::CommandError(format!(
                    "Failed to set Ctrl+C handler: {}",
                    e
                )));
            }
        }
        parse_main_config()
    }

    /// Retrieves the templates directory using the following order :
    ///
    /// - The directory path specified by the environment variable TITULAR_TEMPLATES_DIR
    /// - The directory path specified in the main configuration file
    /// - The default directory from PROJECT_DIRS
    ///
    /// # Returns
    /// The path to the templates directory
    ///
    /// # Errors
    /// Returns an error if the directory path cannot be interpolated
    pub fn template_dir(&self) -> Result<PathBuf> {
        let templates_dir_path = env::var_os("TITULAR_TEMPLATES_DIR")
            .map_or(self.config.templates.directory.clone(), |dir| {
                Some(dir.to_string_lossy().to_string())
            });

        let templates_dir = match templates_dir_path {
            Some(dir) => dir,
            None => PROJECT_DIRS.templates_dir().to_string_lossy().to_string(),
        };

        let template_dir = match shellexpand::env(&templates_dir) {
            Ok(dir) => dir.to_string(),
            Err(e) => {
                return Err(Error::InterpolationError {
                    location: ConfigType::MAIN,
                    cause: e.to_string(),
                });
            }
        };

        Ok(PathBuf::from(template_dir))
    }

    pub fn get_config(&self) -> &MainConfig {
        &self.config
    }
}

/// Creates the default main configuration file in the config directory
/// # Arguments
/// * `config_file` - The path to the configuration file
///
/// # Returns
/// A `Result` containing the configuration data as a `String`
///
/// # Errors
/// * `ConfigError` - If the configuration file cannot be created
/// * `IoError` - If an I/O error occurs while creating the configuration file
///
/// # Examples
/// ```
/// use titular::bootstrap::create_default_config;
/// let config_file = PathBuf::from("/path/to/config.toml");
/// let config_data = create_default_config(&config_file);
/// assert!(config_data.is_ok());
/// ```
fn create_default_config(config_file: &PathBuf) -> Result<String> {
    let parent_dir = config_file
        .parent()
        .ok_or_else(|| Error::ConfigError(config_file.to_string_lossy().into_owned()))?;
    std::fs::create_dir_all(parent_dir)?;
    let templates_dir = parent_dir.join("templates").to_string_lossy().into_owned();
    let current_date: DateTime<Local> = Local::now();
    let config_data = DEFAULT_CONF
        .replacen("${templates_dir}", &templates_dir, 1)
        .replacen("${date}", &current_date.to_string(), 1)
        .replacen("${default_template_name}", DEFAULT_TEMPLATE_NAME, 1)
        .replacen(
            "${default_pager}",
            #[cfg(feature = "display")]
            "fancy",
            #[cfg(not(feature = "display"))]
            "bat_or_pager",
            1,
        );

    let config_data = {
        #[cfg(feature = "fetcher")]
        {
            let mut data = config_data;
            data.push_str(&format!("remote_repo   = \"{}\"", DEFAULT_REMOTE_REPO));
            data
        }
        #[cfg(not(feature = "fetcher"))]
        config_data
    };

    File::create(&config_file)?.write_all(config_data.as_bytes())?;
    Ok(config_data)
}

/// Processes the main configuration file retrieving the associated `MainConfig` structure
///
/// This function reads the configuration file and returns a `MainConfig` structure.
/// If the file does not exist, it creates a default configuration file.
///
/// # Returns
///
/// This function returns a `MainConfig` structure.
///
/// # Errors
///
/// This function returns an error if the configuration file cannot be read or parsed.
pub fn parse_main_config() -> Result<MainConfig> {
    let conf_file = &PROJECT_DIRS.config_dir().clone().join(DEFAULT_CONF_FILE);
    let toml_data = match config_parse(conf_file) {
        Ok(data) => data,
        Err(Error::Io(e)) if e.kind() == ::std::io::ErrorKind::NotFound => {
            create_default_config(conf_file)?
        }
        Err(Error::Io(e)) => {
            return Err(Error::ConfigReadError {
                file: String::from(DEFAULT_CONF_FILE),
                cause: e.to_string(),
            });
        }
        Err(e) => return Err(e),
    };

    let res: std::result::Result<MainConfig, ::toml::de::Error> = toml::from_str(&toml_data);
    let main_config = match res {
        Ok(mut config) => {
            config.init();
            config
        }
        Err(e) => {
            return Err(Error::SerdeTomlError {
                location: ConfigType::MAIN,
                file: String::from(DEFAULT_CONF_FILE),
                cause: e.to_string(),
            });
        }
    };

    Ok(main_config)
}
