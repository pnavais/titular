use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::Local;
use serde::Deserialize;
use serde::Serialize;
use serde_json;

use crate::constants::template::{DEFAULT_TEMPLATE_NAME, DEFAULT_TIME_FORMAT};

#[cfg(feature = "fetcher")]
use crate::constants::template::DEFAULT_REMOTE_REPO;
#[cfg(feature = "display")]
use crate::constants::template::DEFAULT_THEME;
use crate::error::*;
use crate::utils::safe_time_format;

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Display {
    Pager,
    Bat,
    BatOrPager,
    Raw,
    #[cfg(feature = "display")]
    Fancy,
}

#[derive(Deserialize, Debug, Default)]
pub struct MainConfig {
    pub defaults: Defaults,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
    pub templates: Templates,
}

#[derive(Deserialize, Debug, Serialize)]
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
    pub display: Option<Display>,
    #[cfg(feature = "display")]
    pub display_theme: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Templates {
    pub directory: Option<String>,
    pub default: String,
    #[cfg(feature = "fetcher")]
    pub remote_repo: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TemplateConfig {
    pub details: Details,
    #[serde(default)]
    pub vars: BTreeMap<String, String>,
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

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            username: whoami::username(),
            templates_url: format!("{}/{}", "https://github.com", whoami::username()),
            templates_repo: "https://raw.githubusercontent.com/pnavais/titular/master/templates"
                .to_string(),
            fill_char: "*".to_string(),
            width: "full".to_string(),
            surround_start: "[".to_string(),
            surround_end: "]".to_string(),
            time_format: DEFAULT_TIME_FORMAT.to_string(),
            time_pattern: "${space}%{time:fg[tc]}".to_string(),
            display: Some(Display::Raw),
            #[cfg(feature = "display")]
            display_theme: Some(DEFAULT_THEME.to_string()),
        }
    }
}

impl Default for Templates {
    fn default() -> Templates {
        Templates {
            directory: None,
            default: DEFAULT_TEMPLATE_NAME.to_string(),
            #[cfg(feature = "fetcher")]
            remote_repo: Some(DEFAULT_REMOTE_REPO.to_string()),
        }
    }
}

impl MainConfig {
    pub fn new() -> Self {
        let mut main_config = MainConfig {
            ..Default::default()
        };
        main_config.init();
        main_config
    }

    /// Perfoms custom initialization using the main configuration values
    pub fn init(&mut self) {
        // Keep defaults as vars
        self.defaults.to_map().iter().for_each(|(k, v)| {
            self.vars
                .insert(format!("defaults.{}", k.to_string()), v.to_string());
        });
        // Add misc vars
        self.vars.insert(
            "time".to_owned(),
            safe_time_format(&Local::now(), &self.defaults.time_format),
        );
    }
}

impl FromStr for Display {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "pager" => Ok(Display::Pager),
            "bat" => Ok(Display::Bat),
            "bat_or_pager" | "batorpager" => Ok(Display::BatOrPager),
            "raw" => Ok(Display::Raw),
            #[cfg(feature = "display")]
            "fancy" => Ok(Display::Fancy),
            _ => Err(Error::ConfigError(format!("Invalid display: {}", s))),
        }
    }
}

pub fn parse(file_path: &PathBuf) -> Result<String> {
    let mut config_content = String::new();
    File::open(&file_path)?.read_to_string(&mut config_content)?;
    Ok(config_content)
}

impl Defaults {
    pub fn to_map(&self) -> BTreeMap<String, String> {
        // Convert the struct to a JSON value
        let json_value = serde_json::to_value(self).unwrap();

        // Convert JSON object to HashMap
        let mut map = BTreeMap::new();
        if let serde_json::Value::Object(obj) = json_value {
            for (key, value) in obj {
                if let serde_json::Value::String(s) = value {
                    map.insert(key, s);
                } else if let serde_json::Value::Null = value {
                    map.insert(key, "null".to_string());
                } else {
                    map.insert(key, value.to_string());
                }
            }
        }
        map
    }
}
