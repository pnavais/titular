#[cfg(feature = "display")]
include!("build/build_display.rs");

#[cfg(not(feature = "display"))]
fn main() {}
