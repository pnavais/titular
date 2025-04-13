use once_cell::sync::Lazy;

#[derive(Default)]
pub struct TermSize {
    width: usize,
    height: usize,
}

/// Retrieves the stored terminal width
///
/// # Returns
/// A `TermSize` struct with the stored terminal width and height
impl TermSize {
    pub fn new() -> TermSize {
        if let Some((w, h)) = Self::get_dimensions() {
            TermSize {
                width: w,
                height: h,
            }
        } else {
            TermSize {
                width: 0,
                height: 0,
            }
        }
    }

    /// Retrieves the stored terminal width and height
    ///
    /// # Returns
    /// A tuple with the stored terminal width and height
    fn get_dimensions() -> Option<(usize, usize)> {
        if cfg!(feature = "minimal") {
            #[cfg(feature = "minimal")]
            term_size::dimensions()
        } else if cfg!(feature = "display") {
            #[cfg(feature = "display")]
            {
                crossterm::terminal::size()
                    .ok()
                    .map(|(w, h)| (w as usize, h as usize))
            }
            #[cfg(not(feature = "display"))]
            {
                None
            }
        } else {
            None
        }
    }

    /// Retrieves the stored terminal width
    ///
    /// # Returns
    /// The stored terminal width
    pub fn get_term_width(&self) -> usize {
        self.width
    }

    /// Retrieves the stored terminal height
    ///
    /// # Returns
    /// The stored terminal height
    pub fn get_term_height(&self) -> usize {
        self.height
    }

    /// Retrieves the current terminal width
    ///
    /// # Returns
    /// The current terminal width
    pub fn get_current_width() -> usize {
        if let Some((w, _h)) = Self::get_dimensions() {
            w
        } else {
            0
        }
    }
}

pub static TERM_SIZE: Lazy<TermSize> = Lazy::new(|| TermSize::new());
