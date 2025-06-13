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
pub mod constants;
pub mod context;
pub mod context_manager;
pub mod controller;
#[cfg(feature = "fetcher")]
pub mod dispatcher;
pub mod display;
pub mod error;
#[cfg(feature = "fetcher")]
pub mod fetcher;
pub mod filters;
pub mod formatter;
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

/// The titular prelude
///
/// This module re-exports the most commonly used items from titular.
/// You can use it with `use titular::prelude::*;` to bring all common items into scope.
pub mod prelude {
    // Re-export commonly used traits
    pub use crate::transforms::Transform;

    // Re-export commonly used types
    pub use crate::context_manager::ContextManager;
    pub use crate::error::Result;

    // Re-export commonly used constants
    pub use crate::constants::padding;

    // Re-export commonly used functions
    pub use crate::string_utils::{expand_to_visual_width, is_visually_empty};
}
