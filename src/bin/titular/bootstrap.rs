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
                            fill_char = \"*\"\n\
                            width = \"full\"\n\
                            surround_start = \"[\"\n\
                            surround_end = \"]\"\n\n\
                            [vars]\n\
                            steel_blue   = \"RGB(70, 130, 180)\"\n\
                            light_purple = \"FIXED(134)\"\n\
                            red          = \"NAME(Red)\"\n\
                            green        = \"NAME(Green)\"\n\
                            yellow       = \"NAME(Yellow)\"\n
                            space        = \" \"\n\n\
                            [templates]\n\
                            directory = \"$HOME/.config/titular/templates\"\n\
                            default = \"Basic\"\n";

static DEFAULT_CONF_FILE: &str = "titular.conf";

#[derive(Debug)]
pub struct BootStrap {
    config: MainConfig,
}

impl BootStrap {
    pub fn new() -> Result<Self> {
        Ok(BootStrap { config: parse_main_config()? })
    }

    pub fn template_dir(&self) -> Result<PathBuf> {        
        let template_dir = match shellexpand::env(&self.config.templates.directory) {
            Ok(dir) => dir.as_ref().to_owned(),
            Err(e) => return Err(Error::InterpolationError{ location: ConfigType::MAIN, cause: e.to_string() }),
        };
        Ok(PathBuf::from(template_dir))
    }

    pub fn get_config(&self)  -> &MainConfig {
        return &self.config;
    }
}

fn create_default_config(input_dir: &PathBuf) -> Result<String> {   
    let parent_dir = input_dir.parent().ok_or(Error::ConfigError(input_dir.to_string_lossy().into_owned()))?;
    std::fs::create_dir_all(parent_dir)?;
    File::create(&input_dir)?.write_all(DEFAULT_CONF.as_bytes())?;
    Ok(String::from(DEFAULT_CONF))
}

pub fn parse_main_config() -> Result<MainConfig> {
    let conf_file = &PROJECT_DIRS.config_dir().clone().join(DEFAULT_CONF_FILE);
    let toml_data = match config_parse(conf_file) {
        Ok(data) => data,
        Err(Error::Io(e)) if e.kind() == ::std::io::ErrorKind::NotFound => create_default_config(conf_file)?,
        Err(Error::Io(e)) => return Err(Error::ConfigReadError{file: String::from(DEFAULT_CONF_FILE), cause: e.to_string() }),
        Err(e) => return Err(e),
    };

    let res : std::result::Result<MainConfig, ::toml::de::Error> = toml::from_str(&toml_data);
    let mut main_config = match res {
        Ok(config) => config,
        Err(e) => return Err(Error::SerdeTomlError{location: ConfigType::MAIN, file: String::from(DEFAULT_CONF_FILE), cause: e.to_string()}),
    };
    
    // Keep defaults as vars
    main_config.vars.insert("defaults.fill_char".to_owned(), main_config.defaults.fill_char.to_owned());
    main_config.vars.insert("defaults.width".to_owned(), main_config.defaults.width.to_owned());
    main_config.vars.insert("defaults.surround_start".to_owned(), main_config.defaults.surround_start.to_owned());
    main_config.vars.insert("defaults.surround_end".to_owned(), main_config.defaults.surround_end.to_owned());

    Ok(main_config)
}