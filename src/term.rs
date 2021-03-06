extern crate term_size;

use lazy_static::lazy_static;

#[derive(Default)]
pub struct TermSize {
    width: usize,
    height: usize,
}

impl TermSize {
    pub fn new() -> TermSize {
        if let Some((w, h)) = term_size::dimensions() {
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

    /// Retrieves the stored terminal width
    pub fn get_term_width(&self) -> usize {
        self.width
    }

    /// Retrieves the stored terminal height
    pub fn get_term_height(&self) -> usize {
        self.height
    }

    /// Retrieves the current terminal width
    pub fn get_current_witdh() -> usize {
        if let Some((w, _h)) = term_size::dimensions() {
            w
        } else {
            0
        }
    }
}

lazy_static! {
    pub static ref TERM_SIZE: TermSize = TermSize::new();
}
