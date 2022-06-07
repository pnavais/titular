use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::collections::HashMap;

use serde::Deserialize;
use chrono::Local;

use crate:: {
    error::*,
    fallback_map::MapProvider,
};

#[derive(Deserialize, Debug, Default)]
pub struct MainConfig {    
    pub defaults: Defaults,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    pub templates: Templates,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Defaults {    
    pub username: String,
    pub templates_url: String,
    pub templates_repo: String,
    pub fill_char: String,    
    pub width: String,
    pub surround_start: String,
    pub surround_end: String,
    pub time_format: String,
    pub time_pattern: String,
}

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            username: whoami::username(),
            templates_url: format!("{}/{}", "https://github.com", whoami::username()),
            templates_repo: "https://raw.githubusercontent.com/pnavais/titular/master/templates".to_string(),
            fill_char: "*".to_string(),
            width: "full".to_string(),
            surround_start: "[".to_string(),
            surround_end: "]".to_string(),
            time_format: "%H:%M:%S".to_string(),
            time_pattern: "${space}%{time:fg[tc]}".to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Templates {
    pub directory: String,
    pub default: String,
}

impl Default for Templates {
    fn default() -> Templates {
        Templates {
            directory: "$HOME/.config/titular/templates".to_string(),
            default: "basic".to_string(),
        }
    }
}

impl MapProvider<String, String> for MainConfig {    
    fn contains(&self, key: &String) -> bool {
        self.vars.contains_key(key)
    }

    fn resolve(&self, key: &String) -> Option<&String> {
        self.vars.get(key)
    }  
    
    fn is_active(&self, key: &String) -> bool {
        match self.resolve(key) {
            Some(v) => v == "true",
            None => false,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct TemplateConfig {
    pub details: Details,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    pub pattern: Pattern,
}

#[derive(Deserialize, Debug, Default)]
pub struct Details {
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct Pattern {
    pub data: String,
}

impl MainConfig {
    pub fn new() -> Self {
        let mut main_config = MainConfig { ..Default::default() };
        main_config.init();
        main_config
    }

    /// Perfoms custom initialization using the main configuration values
    pub fn init(&mut self) {
        // Keep defaults as vars
        self.vars.insert("defaults.fill_char".to_owned(), self.defaults.fill_char.to_owned());
        self.vars.insert("defaults.width".to_owned(), self.defaults.width.to_owned());
        self.vars.insert("defaults.surround_start".to_owned(), self.defaults.surround_start.to_owned());
        self.vars.insert("defaults.surround_end".to_owned(), self.defaults.surround_end.to_owned());
        self.vars.insert("time".to_owned(), Local::now().format(&self.defaults.time_format).to_string());
    }
}

impl MapProvider<String, String> for TemplateConfig {
    fn contains(&self, key: &String) -> bool {
        self.vars.contains_key(key)
    }

    fn resolve(&self, key: &String) -> Option<&String> {
        self.vars.get(key)
    }
    
    fn is_active(&self, key: &String) -> bool {
        match self.resolve(key) {
            Some(v) => v == "true",
            None => false,
        }
    }
}

pub fn parse(file_path: &PathBuf) -> Result<String> {
    let mut config_content = String::new();
    File::open(&file_path)?.read_to_string(&mut config_content)?;
    Ok(config_content)
}
