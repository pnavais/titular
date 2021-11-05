use serde::Deserialize;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::collections::HashMap;

use crate:: {
    error::*,
    fallback_map::MapProvider,
};

#[derive(Deserialize, Debug)]
pub struct MainConfig {    
    pub defaults: Defaults,
    #[serde(default)]
    pub colours: HashMap<String, String>,
    pub templates: Templates,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Defaults {    
    pub fill_char: String,    
    pub width: String,
    pub surround_start: String,
    pub surround_end: String,
}

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            fill_char: "*".to_string(),
            width: "full".to_string(),
            surround_start: "[".to_string(),
            surround_end: "]".to_string(),
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
    fn resolve(&self, key: &String) -> Option<&String> {
        self.colours.get(key)
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

impl MapProvider<String, String> for TemplateConfig {    
    fn resolve(&self, key: &String) -> Option<&String> {
        self.vars.get(key)
    }
}

pub fn parse(file_path: &PathBuf) -> Result<String> {
    let mut config_content = String::new();
    File::open(&file_path)?.read_to_string(&mut config_content)?;
    Ok(config_content)
}
