//! `titular` is a library to print predefined titles.
//!
//! The main struct of this crate is `TemplateController` which can be used to
//! format a given template with a supplied configuration and a context.
//!
//! The controller further delegates to the `TemplatesFormatter` which is in charge
//! of processing the pattern supplied in the template file and applying the defined
//! styles and transforms.
//!
//! "Hello world" example:
//! ```
//! use titular::*;
//! 
//!  // Create the main config
//!  let mut main_config: MainConfig = MainConfig::new();
//! // Modify main config as needed
//! main_config.defaults.fill_char = "*";
//!   
//! // Create a context and feed it with some data
//! let mut context = Context::new();
//! context.insert_multi("m", vec!["Message1".to_string(), "Message2".to_string()]);
//!
//! // Create the controller and format the given template name
//! let template_name = "basic"
//! let controller = TemplatesController { input_dir: PathBuf::from("/path/to/templates"), config: &main_config };
//! controller.format(&context, template_name);
//! 
//! ```
pub mod term;
pub mod error;
pub mod templates;
pub mod config;
pub mod transform;
pub mod formatter;
pub mod context;
pub mod color_manager;
pub mod fallback_map;
pub mod styler;
#[cfg(feature = "fetcher")]
pub mod fetcher;