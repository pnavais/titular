use crate::context_manager::ContextManager;
use crate::error::Result;
use crate::term::TERM_SIZE;
use crate::transforms::Transform;
use console::Term;

/// Handles line endings based on context flags
pub struct LineHandler;

impl LineHandler {
    pub fn new() -> Self {
        Self
    }
}

/// LineHandler is a transform that handles line manipulations based on context flags.
/// For example, it can be used to skip the newline character at the end of the text.
/// When clear is active, it will move to the beginning of the line and clear it.
impl Transform for LineHandler {
    fn transform(&self, text: &str) -> Result<String> {
        let ctx = ContextManager::get().read()?;

        if ctx.is_active("clear") {
            let term = Term::stdout();
            term.clear_line()?;
            term.move_cursor_left(TERM_SIZE.get_term_width())?;
        }

        Ok(format!(
            "{}{}",
            text,
            ctx.is_active("skip-newline").then_some("").unwrap_or("\n")
        ))
    }
}
