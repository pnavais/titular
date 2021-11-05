use glob::glob;
use std::path::{Path, PathBuf};
use ansi_term::Colour::{ Red, Yellow };
use crate:: {
    config::{MainConfig, TemplateConfig, parse as config_parse},
    error::*,
    formatter::TemplateFormatter,
    context::Context,
};

static DEFAULT_EXT: &str = "tpl";

pub struct TemplatesController {
    pub input_dir: PathBuf,
}

impl TemplatesController {
    pub fn list(&self) {        
        if Path::new(&self.input_dir).exists() {
            let templates = glob(&*format!("{}{}", self.input_dir.to_string_lossy(), "/**/*.tpl")).expect("Failed to read glob pattern");

            let files : Vec<String> = templates.map(|t| t.unwrap().file_name().unwrap().to_owned().into_string().unwrap()).collect();
            let num_files = files.len();
            if num_files >= 1 {
                println!("Found {} template{} in \"{}\"\n", num_files, if num_files > 1 { "s" } else { "" }, self.input_dir.to_string_lossy());
                for f in files {
                    println!("- {}", f);
                }
            } else {
                println!("{}", Yellow.paint("No templates found"));
            }
        } else {
            println!("{}", Red.paint(format!("Templates directory \"{}\" not found", self.input_dir.to_string_lossy())));
        }
    }

    
    fn parse(&self, name: &str) -> Result<TemplateConfig> {
        let path = self.get_template_file(name);
        let toml_data = match config_parse(&self.input_dir.clone().join(&path)) {
            Ok(data) => data,
            Err(Error::Io(e)) if e.kind() == ::std::io::ErrorKind::NotFound => return Err(Error::TemplateNotFound{file: String::from(path), cause: e.to_string() }),
            Err(Error::Io(e)) => return Err(Error::TemplateReadError{file: String::from(path), cause: e.to_string() }),
            Err(e) => return Err(e),
        };

        let res : std::result::Result<TemplateConfig, ::toml::de::Error> = toml::from_str(&toml_data);
        let template_config = match res {
            Ok(config) => config,
            Err(e) => return Err(Error::SerdeTomlError{ location: ConfigType::TEMPLATE, file: String::from(path), cause: e.to_string()}),
        };        

        Ok(template_config)
    }

    pub fn get_template_file(&self, name: &str) -> String {
        String::from(name).to_lowercase() + "." + DEFAULT_EXT        
    }
    
    pub fn format<'a>(&self, context: &Context, main_config: &MainConfig, template_name: &str) -> Result<bool> {        
        let template_config = self.parse(template_name)?;
        TemplateFormatter::new(&main_config).format(&context, &template_config)
    }
}