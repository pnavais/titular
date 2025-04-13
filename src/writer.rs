use std::{
    fs::create_dir_all,
    io::{Write, stdin, stdout},
    path::{Path, PathBuf},
};

use nu_ansi_term::Color::Yellow;

use crate::{
    config::{DEFAULT_TEMPLATE_EXT, MainConfig},
    error::*,
};

pub const DEFAULT_TEMPLATE: &str = "[details]\n\
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

pub struct TemplateWriter {}

impl TemplateWriter {
    /// Retrieves the template file name (with extension)
    pub fn get_template_file(name: &str) -> String {
        let file_name = String::from(name).to_lowercase();
        if name.ends_with(DEFAULT_TEMPLATE_EXT) {
            file_name
        } else {
            file_name + DEFAULT_TEMPLATE_EXT
        }
    }

    /// Writes a new template file using default and automatically computed contents (i.e. user name)
    pub fn write_new(file_path: &PathBuf, config: &MainConfig) -> Result<()> {
        let file_name = TemplateWriter::get_template_name(file_path);
        let mut template = DEFAULT_TEMPLATE.replacen("@name", &file_name, 1);

        let author = match config.vars.get(&"username".to_owned()) {
            Some(u) => u,
            None => &config.defaults.username,
        };

        let url = match config.vars.get(&"template_url".to_owned()) {
            Some(u) => u,
            None => &config.defaults.templates_url,
        };

        template = template.replacen("@author", author, 1);
        template = template.replacen("@url", url, 1);
        match file_path.parent() {
            Some(parent) => {
                create_dir_all(parent)?;
                match std::fs::write(file_path, template) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Error::TemplateWriteError(format!(
                        "Cannot write file {} -> {}",
                        file_path.to_string_lossy(),
                        e
                    ))),
                }
            }
            None => Err(Error::TemplateWriteError(format!(
                "Cannot create directory for file {}",
                file_path.to_string_lossy()
            ))),
        }
    }

    /// Retrieves the template name (without extension)
    pub fn get_template_name(file_path: &Path) -> String {
        let file_name = file_path.file_name().map_or("@file_name".to_string(), |m| {
            m.to_string_lossy()
                .as_ref()
                .replacen(DEFAULT_TEMPLATE_EXT, "", 1)
        });

        let mut c = file_name.chars();
        match c.next() {
            None => file_name,
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    /// Creates a new template in the repository if not existing asking optionally
    /// the user using a confirmation prompt.
    pub fn create_new_template(
        name: &str,
        prompt_user: bool,
        input_dir: &PathBuf,
        config: &MainConfig,
    ) -> Result<(String, PathBuf, bool)> {
        let path = TemplateWriter::get_template_file(name);
        let template = input_dir.clone().join(&path);

        let mut template_created = false;

        if !template.exists() {
            if prompt_user {
                loop {
                    let mut input = String::new();
                    print!(
                        "Template \"{}\" not found. Do you want to create it ? [Y/n] : ",
                        Yellow.paint(name)
                    );
                    let _ = stdout().flush();
                    stdin()
                        .read_line(&mut input)
                        .expect("error: unable to read user input");
                    input = input.trim().to_lowercase();
                    if input == "y" || input == "yes" || input.is_empty() {
                        break;
                    } else if input == "n" || input == "no" {
                        return Ok(("".to_owned(), PathBuf::new(), false));
                    }
                }
            }
            TemplateWriter::write_new(&template, config)?;
            template_created = true;
        }
        Ok((path, template, template_created))
    }
}
