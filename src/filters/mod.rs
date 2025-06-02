//! Filters for the Tera template engine
//! Currently supports the following filters:
//! - `color` : Apply a color to the text
//! - `style` : Apply a style to the text

pub mod color;
pub mod style;
pub mod surround;

pub use color::create_color_filter;
pub use style::create_style_filter;
pub use surround::create_surround_filter;
