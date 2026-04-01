#[cfg(feature = "display")]
mod build_display;

/// Main function for the build script building the following assets:
/// - Syntaxes
/// - Themes
fn main() {
    #[cfg(feature = "display")]
    build_display::build();
}
