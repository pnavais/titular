//! Filters for the Tera template engine
//! Currently supports the following filters:
//! - `color` : Apply a color to the text
//! - `style` : Apply a style to the text

pub mod append;
pub mod color;
pub mod hide;
pub mod pad;
pub mod style;
pub mod surround;

pub use append::create_append_filter;
pub use color::create_color_filter;
pub use pad::create_pad_filter;
pub use style::create_style_filter;
pub use surround::create_surround_filter;
