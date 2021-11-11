use std::io::Write;
use thiserror::Error;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum ConfigType {
    #[strum(serialize = "Main config")]
    MAIN,
    #[strum(serialize = "Template")]
    TEMPLATE,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    ClientError(#[from] ::reqwest::Error),     
    #[error("unable to parse {location} file {file:?}. Cause : {cause}")]
    SerdeTomlError {
        location: ConfigType,
        file: String,
        cause: String,
    },
    #[error("unable to create default configuration file in {0}")]
    ConfigError(String),
    #[error("unable to read configuration file {file:?}. Cause : {cause}")]
    ConfigReadError {
        file: String,
        cause: String,
    },
    #[error("unable to open template file {file:?}. Cause : {cause}")]
    TemplateNotFound {
        file: String,
        cause: String,
    },
    #[error("unable to read template file {file:?}. Cause : {cause}")]
    TemplateReadError {
        file: String,
        cause: String,
    },
    #[error("unable to interpolate variable. Cause : {cause}")]
    InterpolationError {
        location: ConfigType,
        cause: String,
    },
    #[error("error writing to template. Cause : {0}")]
    TemplateWriteError(String),
    #[error("error downloading template \"{0}\". Cause : {1}")]
    TemplateDownloadError(String, String),
    #[error("error processing args. Cause : {0}")]
    ArgsProcessingError(String),
    #[error("{0}")]
    Msg(String),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Msg(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Msg(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn default_error_handler(error: &Error, output: &mut dyn Write) {
    use ansi_term::Colour::Red;

    match error {
        Error::Io(ref io_error) if io_error.kind() == ::std::io::ErrorKind::BrokenPipe => {
            ::std::process::exit(0);
        }
        Error::ConfigReadError{ file: _ , cause: _ } => {
            writeln!(output, "{}: {}", Red.paint("[config error : {}]"), error).ok();
        }
        Error::SerdeTomlError{location: _, file: _ , cause: _ } | Error::InterpolationError { location: _, cause: _ } => {
            writeln!(output, "{}: {}", Red.paint("[config error]"), error).ok();
        }        
        Error::TemplateNotFound{ file: _ , cause: _ } | Error::TemplateReadError{ file: _ , cause: _ }   => {
            writeln!(output, "{}: {}", Red.paint("[template error]"), error).ok();
        }
        _ => {
            writeln!(output, "{}: {}", Red.paint("[titular error]"), error).ok();
        }
    };
}