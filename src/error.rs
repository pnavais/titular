use std::io::Write;
use strum_macros::Display;
use thiserror::Error;

#[derive(Debug, Display)]
pub enum ConfigType {
    #[strum(serialize = "Main config")]
    MAIN,
    #[strum(serialize = "Template")]
    TEMPLATE,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("error processing args. Cause : {0}")]
    ArgsProcessingError(String),
    #[cfg(feature = "fetcher")]
    #[error("HTTP client error: {0}")]
    ClientError(::isahc::Error),
    #[cfg(feature = "fetcher")]
    #[error("HTTP client error: {0}")]
    ClientHttpError(::isahc::http::Error),
    #[error("unable to create default configuration file in {0}")]
    ConfigError(String),
    #[error("unable to read configuration file {file:?}. Cause : {cause}")]
    ConfigReadError { file: String, cause: String },
    #[error("Error executing command. Cause : {0}")]
    CommandError(String),
    #[error(transparent)]
    Fmt(#[from] ::std::fmt::Error),
    #[error("unable to parse {location} file {file:?}. Cause : {cause}")]
    SerdeTomlError {
        location: ConfigType,
        file: String,
        cause: String,
    },
    #[error("JSON parsing error: {0}")]
    JsonError(String),
    #[cfg(feature = "display")]
    #[error(transparent)]
    SyntectError(#[from] ::syntect::Error),
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[cfg(feature = "fetcher")]
    #[error("template with the same name already exists : \"{0}\"")]
    TemplateAlreadyExists(String),
    #[cfg(feature = "fetcher")]
    #[error("error downloading template \"{0}\". Cause : {1}")]
    TemplateDownloadError(String, String),
    #[error("unable to open template file {file:?}. Cause : {cause}")]
    TemplateNotFound { file: String, cause: String },
    #[error("unable to read template file {file:?}. Cause : {cause}")]
    TemplateReadError { file: String, cause: String },
    #[error("unable to interpolate variable. Cause : {cause}")]
    InterpolationError { location: ConfigType, cause: String },
    #[error("error writing to template. Cause : {0}")]
    TemplateWriteError(String),
    #[error("{0}")]
    Msg(String),
}

#[cfg(feature = "fetcher")]
impl From<isahc::Error> for Error {
    fn from(error: isahc::Error) -> Self {
        // For any other client errors
        Error::ClientError(error)
    }
}

#[cfg(feature = "fetcher")]
impl From<isahc::http::Error> for Error {
    fn from(error: isahc::http::Error) -> Self {
        // For connection errors and other client errors
        Error::ClientHttpError(error)
    }
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

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn default_error_handler(error: &Error, output: &mut dyn Write) {
    use nu_ansi_term::Color::Red;

    match error {
        Error::Io(io_error) if io_error.kind() == ::std::io::ErrorKind::BrokenPipe => {
            ::std::process::exit(0);
        }
        Error::ConfigReadError { file: _, cause: _ } => {
            writeln!(output, "{}: {}", Red.paint("[config error : {}]"), error).ok();
        }
        Error::SerdeTomlError {
            location: _,
            file: _,
            cause: _,
        }
        | Error::InterpolationError {
            location: _,
            cause: _,
        } => {
            writeln!(output, "{}: {}", Red.paint("[config error]"), error).ok();
        }
        Error::TemplateNotFound { file: _, cause: _ }
        | Error::TemplateReadError { file: _, cause: _ } => {
            writeln!(output, "{}: {}", Red.paint("[template error]"), error).ok();
        }
        _ => {
            writeln!(output, "{}: {}", Red.paint("[titular error]"), error).ok();
        }
    };
}
