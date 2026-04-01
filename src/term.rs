
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
    #[must_use] 
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
        // Prefer `term_size` when `minimal` is enabled; otherwise use crossterm when `display` is on.
        #[cfg(feature = "minimal")]
        {
            term_size::dimensions()
        }

        #[cfg(all(feature = "display", not(feature = "minimal")))]
        {
            crossterm::terminal::size()
                .ok()
                .map(|(w, h)| (w as usize, h as usize))
        }

        #[cfg(not(any(feature = "minimal", feature = "display")))]
        {
            None
        }
    }

    /// Retrieves the stored terminal width
    ///
    /// # Returns
    /// The stored terminal width
    #[must_use] 
    pub fn get_term_width(&self) -> usize {
        self.width
    }

    /// Retrieves the stored terminal height
    ///
    /// # Returns
    /// The stored terminal height
    #[must_use] 
    pub fn get_term_height(&self) -> usize {
        self.height
    }

    /// Retrieves the current terminal width
    ///
    /// # Returns
    /// The current terminal width
    #[must_use] 
    pub fn get_current_width() -> usize {
        if let Some((w, _h)) = Self::get_dimensions() {
            w
        } else {
            0
        }
    }
}

pub static TERM_SIZE: std::sync::LazyLock<TermSize> = std::sync::LazyLock::new(TermSize::new);
