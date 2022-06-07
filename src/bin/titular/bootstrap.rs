use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

pub use titular:: {
    error::*,
    config::{ MainConfig, parse as config_parse }
};

use crate::{
    directories::PROJECT_DIRS,
};

static DEFAULT_CONF: &str = "[defaults]\n\
                            fill_char      = \"*\"\n\
                            width          = \"full\"\n\
                            surround_start = \"[\"\n\
                            surround_end   = \"]\"\n\
                            time_pattern   = \"${space}%{time:fg[tc]}\"\n\
                            time_format    =  \"%H:%M:%S\"\n\n\
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
                            directory    = '${templates_dir}'\n\
                            default      = \"basic\"\n";

static DEFAULT_CONF_FILE: &str = "titular.conf";

#[derive(Debug)]
pub struct BootStrap {
    config: MainConfig,
}

impl BootStrap {
    pub fn new() -> Result<Self> {
        Ok(BootStrap { config: parse_main_config()? })
    }

    /// Retrieves the templates directory from 
    /// the main configuration interpolating environment variables if needed
    pub fn template_dir(&self) -> Result<PathBuf> {        
        let template_dir = match shellexpand::env(&self.config.templates.directory) {
            Ok(dir) => dir.as_ref().to_owned(),
            Err(e) => return Err(Error::InterpolationError{ location: ConfigType::MAIN, cause: e.to_string() }),
        };
        Ok(PathBuf::from(template_dir))
    }

    pub fn get_config(&self)  -> &MainConfig {
        &self.config
    }
}

/// Creates the default main configuration file in the config directory
fn create_default_config(config_file: &PathBuf) -> Result<String> {   
    let parent_dir = config_file.parent().ok_or_else(|| Error::ConfigError(config_file.to_string_lossy().into_owned()))?;
    std::fs::create_dir_all(parent_dir)?;
    let templates_dir = parent_dir.join("templates").to_string_lossy().into_owned();
    let config_data = DEFAULT_CONF.replacen("${templates_dir}", &templates_dir, 1);
    File::create(&config_file)?.write_all(config_data.as_bytes())?;
    Ok(config_data)
}

/// Process the main configuration file retrieving the associated `MainConfig` structure
pub fn parse_main_config() -> Result<MainConfig> {
    let conf_file = &PROJECT_DIRS.config_dir().clone().join(DEFAULT_CONF_FILE);
    let toml_data = match config_parse(conf_file) {
        Ok(data) => data,
        Err(Error::Io(e)) if e.kind() == ::std::io::ErrorKind::NotFound => create_default_config(conf_file)?,
        Err(Error::Io(e)) => return Err(Error::ConfigReadError{file: String::from(DEFAULT_CONF_FILE), cause: e.to_string() }),
        Err(e) => return Err(e),
    };

    let res : std::result::Result<MainConfig, ::toml::de::Error> = toml::from_str(&toml_data);
    let main_config = match res {
        Ok(mut config) => { config.init(); config },
        Err(e) => return Err(Error::SerdeTomlError{location: ConfigType::MAIN, file: String::from(DEFAULT_CONF_FILE), cause: e.to_string()}),
    };

    Ok(main_config)
}
