use crate::context_manager::ContextManager;
use crate::error::Result;
use crate::transforms::Transform;

/// Handles line endings based on context flags
pub struct LineHandler;

impl LineHandler {
    pub fn new() -> Self {
        Self
    }
}

/// LineHandler is a transform that handles line manipulations based on context flags.
/// For example, it can be used to skip the newline character at the end of the text.
impl Transform for LineHandler {
    fn transform(&self, text: &str) -> Result<String> {
        let ctx = ContextManager::get().read()?;
        Ok(format!(
            "{}{}",
            text,
            ctx.is_active("skip-newline").then_some("").unwrap_or("\n")
        ))
    }
}
