//! `titular` is a library to print custom formatted messages in the form of titles.
//!
//! The main struct of this crate is `TitlePrinter` which can be used to
//! configure and run the title generation.
//!
//! If you need more control, you can also use the structs in the submodules
//! (start with `controller::Controller`), but note that the API of these
//! internal modules is much more likely to change. Some or all of these
//! modules might be removed in the future.
//!
//! "Hello world" example:
//! ```
//! //use std::path::PathBuf;
//! //use titular::{TitlePrinter, config::MainConfig};
//!
//! //let config = MainConfig::default();
//! //let input_dir = PathBuf::from("templates");
//!
//! //TitlePrinter::new()
//! //    .input_from_bytes(b"Hello world!\n")
//! //    .template("basic")
//! //    .config(&config)
//! //    .input_dir(&input_dir)
//! //    .print()
//! //    .unwrap();
//! ```

pub mod color_manager;
pub mod config;
pub mod context;
pub mod controller;
#[cfg(feature = "fetcher")]
pub mod dispatcher;
pub mod display;
pub mod error;
#[cfg(feature = "fetcher")]
pub mod fetcher;
pub mod filters;
#[cfg(feature = "fetcher")]
pub mod github;
pub mod log;
pub mod reader;
pub mod string_utils;
#[cfg(feature = "minimal")]
pub mod term;
#[cfg(feature = "display")]
pub mod theme;
pub mod transforms;
pub mod utils;
pub mod writer;

/// Default template file extension
pub const DEFAULT_TEMPLATE_EXT: &str = ".tl";

/// Default template name
pub const DEFAULT_TEMPLATE_NAME: &str = "basic";

/// Default theme for display
pub const DEFAULT_THEME: &str = "base16-ocean.dark";

/// Default time format
pub const DEFAULT_TIME_FORMAT: &str = "%H:%M:%S";

#[cfg(feature = "fetcher")]
/// Default remote repository for templates
pub const DEFAULT_REMOTE_REPO: &str = "github:pnavais/titular/templates";
