#[cfg(feature = "display")]
mod build_display;

/// Main function for the build script building the following assets:
/// - Syntaxes
/// - Themes
///
/// # Returns
/// A `Result` indicating success or failure.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "display")]
    build_display::build()?;
    Ok(())
}
