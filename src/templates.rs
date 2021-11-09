use std::io::{stdin, stdout, Write};

use glob::glob;
use std::path::{Path, PathBuf};
use ansi_term::Colour::{ Red, Yellow, Green };
use async_std::task;
use url::Url;

use crate:: {
    config::{MainConfig, TemplateConfig, parse as config_parse},
    error::*,
    formatter::TemplateFormatter,
    context::Context,
    fetcher::*,
};

static DEFAULT_EXT: &str = ".tl";

pub struct TemplatesController<'a> {
    pub input_dir: PathBuf,
    pub config: &'a MainConfig,
}

impl <'a> TemplatesController<'a> {

    pub fn list(&self) {        
        if Path::new(&self.input_dir).exists() {
            let templates = glob(&*format!("{}{}{}", self.input_dir.to_string_lossy(), "/**/*", DEFAULT_EXT)).expect("Failed to read glob pattern");

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

    pub fn create(&self, name: &str) ->  Result<()> {
        let (_, _, created) = self.create_new_template(name, false)?;
        if created {
            println!("New template \"{}\" created", Green.paint(name));
        } else {
            println!("{}", Yellow.paint(format!("Template \"{}\" already exists", name)));
        }
        Ok(())
    }

    pub fn open(&self, name: &str) ->  Result<()> {
        let (path, template, _) = self.create_new_template(name, true)?;
        
        if !path.is_empty() {
            match edit::edit_file(&template) {
                Ok(_) => Ok(()),
                Err(e) => return Err(Error::TemplateReadError{ file: path, cause: e.to_string() }),
            }
        } else {
            Ok(())
        }
    }

    pub fn remove(&self, name: &str) ->  Result<()> {
        let path = self.get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        if template.exists() {
            match std::fs::remove_file(template) {
                Ok(_) => println!("Template \"{}\" removed", Green.paint(name)),
                Err(e) => return Err(Error::TemplateReadError{ file: path, cause: e.to_string() }),
            }
        } else {
            println!("{}", Yellow.paint(format!("Template \"{}\" not found", name)));
        }

        Ok(())
    }
    
    pub fn format(&self, context: &Context, main_config: &MainConfig, template_name: &str) -> Result<bool> {        
        let template_config = self.parse(template_name)?;
        TemplateFormatter::new(&main_config).format(&context, &template_config)
    }

    pub fn add(&self, urls: &Vec<String>) -> Result<()> {
        for url in urls {
            self.add_template(&url)?;
        }
        Ok(())
    }

    fn add_template(&self, url: &str) -> Result<()> {
        let mut template_name = url.to_owned();
        
        // Normalize extension
        if !template_name.ends_with(DEFAULT_EXT) {
            template_name.push_str(DEFAULT_EXT);
        }

        let template_url = self.parse_url(&mut template_name, url);                
        let template_target = match self.compute_target(&template_name) {
            Some(t) => t,
            None => return Ok(()),
        };
        
        let result = async {            
            download_file(&template_url, &template_target).await
        };
        
        let res = task::block_on(result);
        if res.is_ok() {
            println!("\nTemplate \"{}\" added succesfully", Green.paint(template_name));
        }

        return res;
    }

    fn parse_url(&self, template_name: &mut String, url: &str) -> String {
        match Url::parse(url) {
            Ok(u) => { 
                let last_slash_idx = u.path().rfind('/').unwrap_or(0);
                let (_, filename) = u.path().split_at(last_slash_idx);
                *template_name = filename.replacen("/", "", 1);

                if template_name.is_empty() { 
                    print!("Template name not detected. Please specify it : ");
                    let _ = stdout().flush();
                    let mut input = String::new();
                    stdin().read_line(&mut input).expect("error: unable to read user input");
                    input = input.trim().to_lowercase();
                    *template_name = if input.is_empty() { "unknown".to_owned() } else { input }
                }
                u.to_string() 
            },
            Err(_) => {                
                format!("{}/{}",&self.config.defaults.templates_repo, template_name)
            }
        }
    }

    fn compute_target(&self, template_name: &String) -> Option<PathBuf> {
        let template_target = self.input_dir.clone().join(&template_name);

        if template_target.exists() {
            loop {
                let mut input = String::new();
                print!("Template \"{}\" already exists. Overwrite ? [yN] : ", Yellow.paint(template_name));
                let _ = stdout().flush();
                stdin().read_line(&mut input).expect("error: unable to read user input");
                input = input.trim().to_lowercase();
                
                if input == "y" || input == "yes" {
                    break;
                } else if input == "n" || input == "no" || input.len() <= 0 {
                    return None;
                }
            }
        }
        
        Some(template_target)
    }

    fn get_template_file(&self, name: &str) -> String {
        let file_name = String::from(name).to_lowercase();
        if name.ends_with(DEFAULT_EXT) { file_name } else { file_name + DEFAULT_EXT }
    }
    
    fn parse(&self, name: &str) -> Result<TemplateConfig> {
        let path = self.get_template_file(name);
        let toml_data = match config_parse(&self.input_dir.clone().join(&path)) {
            Ok(data) => data,
            Err(Error::Io(e)) if e.kind() == ::std::io::ErrorKind::NotFound => return Err(Error::TemplateNotFound{file: String::from(path), cause: e.to_string() }),
            Err(Error::Io(e)) => return Err(Error::TemplateReadError{ file: String::from(path), cause: e.to_string() }),
            Err(e) => return Err(e),
        };

        let res : std::result::Result<TemplateConfig, ::toml::de::Error> = toml::from_str(&toml_data);
        let template_config = match res {
            Ok(config) => config,
            Err(e) => return Err(Error::SerdeTomlError{ location: ConfigType::TEMPLATE, file: String::from(path), cause: e.to_string()}),
        };        

        Ok(template_config)
    }

    fn create_new_template(&self, name: &str, prompt_user: bool) -> Result<(String, PathBuf, bool)> {
        let path = self.get_template_file(name);
        let template = self.input_dir.clone().join(&path);

        let mut template_created = false;

        if !template.exists() {
            if prompt_user {
                loop {         
                    let mut input = String::new();
                    print!("Template \"{}\" not found. Do you want to create it ? [Y/n] : ", Yellow.paint(name));
                    let _ = stdout().flush();
                    stdin().read_line(&mut input).expect("error: unable to read user input");
                    input = input.trim().to_lowercase();
                    if input == "y" || input == "yes" || input.len() <= 0 {
                        break;
                    } else if input == "n" || input == "no" {
                        return Ok(("".to_owned(), PathBuf::new(), false));
                    }
                }
            }
            TemplateWriter::write_new(&template, self.config)?;
            template_created = true;
        }
        Ok((path, template, template_created))
    }

}

static DEFAULT_TEMPLATE: &str  = "[details]\n\
                                name    = \"@name\"\n\
                                version = \"1.0\"\n\
                                author  = \"@author\"\n\
                                url     = \"@url\"\n\n\
                                [vars]\n\
                                f  = \"*\"\n\
                                my_var = \"Hello\"\n\
                                my_color = \"green\"\n\n\
                                [pattern]\n\
                                data = \"${f:fg[cl]:pad}${my_var:fg[my_color]+[ ]}${m:fg[my_color]}${f:fg[cr]:pad}\"\n";

struct TemplateWriter {}

impl TemplateWriter {
    
    fn write_new(file_path: &PathBuf, config: &MainConfig) -> Result<()> {        
        let file_name = TemplateWriter::get_template_name(&file_path);
        let mut template = DEFAULT_TEMPLATE.replacen("@name", &file_name, 1);
        
        let author = match config.vars.get(&"username".to_owned()) {
            Some(u) => u.to_owned(),
            None => config.defaults.username.to_owned(),
        };

        let url = match config.vars.get(&"template_url".to_owned()) {
            Some(u) => u.to_owned(),
            None => config.defaults.templates_url.to_owned(),
        };

        template = template.replacen("@author", &author, 1);
        template = template.replacen("@url", &url, 1);
        match std::fs::write(file_path, template) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::TemplateWriteError(format!("Cannot write file {} -> {}", file_path.to_string_lossy(), e))),
        }
    }

    fn get_template_name(file_path: &PathBuf) -> String {
        let file_name = file_path.file_name().map_or("@file_name".to_string(), |m| { 
            m.to_string_lossy().as_ref().replacen(&format!("{}{}", ".", DEFAULT_EXT),  "", 1).to_owned() 
        });        

        let mut c = file_name.chars();
        match c.next() {
            None => file_name,
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }
}