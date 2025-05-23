use crate::context::Context;
use crate::error::Result;
use crate::transforms::Transform;
use std::sync::Arc;

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
    fn transform(&self, context: Arc<Context>, text: &str) -> Result<String> {
        Ok(format!(
            "{}{}",
            text,
            context
                .is_active("skip-newline")
                .then_some("")
                .unwrap_or("\n")
        ))
    }
}
