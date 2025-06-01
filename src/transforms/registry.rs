use super::Transform;
use crate::error::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;

pub struct TransformRegistry {
    transforms: HashMap<String, Arc<Box<dyn Transform>>>,
    order: Vec<Arc<Box<dyn Transform>>>,
}

impl TransformRegistry {
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
            order: Vec::new(),
        }
    }

    /// Initializes the transform registry with the default transforms
    /// - TemplateRenderer: Renders the template using the Tera engine
    /// - TextProcessor: Processes the text handling padding and line wrapping
    /// - LineHandler: Handles line endings based on context flags
    /// - AnsiFormatter: Handles ANSI escape sequences and nested colors
    pub fn init(&mut self) {
        self.register("template_renderer", super::TemplateRenderer::new());
        self.register("text_processor", super::TextProcessor::default());
        self.register("line_handler", super::LineHandler::new());
        self.register("ansi_formatter", super::AnsiFormatter::new());
    }

    pub fn register<T: Transform + 'static>(&mut self, name: &str, transform: T) {
        let boxed = Arc::new(Box::new(transform) as Box<dyn Transform>);
        self.transforms.insert(name.to_string(), Arc::clone(&boxed));
        self.order.push(boxed);
    }

    pub fn get(&self, name: &str) -> Option<&Arc<Box<dyn Transform>>> {
        self.transforms.get(name)
    }

    /// Process the text through all registered transforms in sequence
    ///
    /// # Arguments
    /// * `text` - The text to process
    ///
    /// # Returns
    /// The processed text after applying all transforms or an error if any transform fails
    pub fn process(&self, text: &str) -> Result<String> {
        self.order
            .iter()
            .try_fold(text.to_string(), |acc, transform| transform.transform(&acc))
    }
}

/// Global transform manager that provides thread-safe access to the shared transform registry
pub struct TransformManager {
    registry: Arc<TransformRegistry>,
}

impl TransformManager {
    /// Gets a reference to the global transform manager
    pub fn get() -> &'static TransformManager {
        static INSTANCE: Lazy<TransformManager> = Lazy::new(|| {
            let mut registry = TransformRegistry::new();
            registry.init();
            TransformManager {
                registry: Arc::new(registry),
            }
        });
        &INSTANCE
    }

    /// Gets a clone of the shared registry
    pub fn share(&self) -> Arc<TransformRegistry> {
        Arc::clone(&self.registry)
    }

    /// Process the text through all registered transforms in sequence
    ///
    /// # Arguments
    /// * `text` - The text to process
    ///
    /// # Returns
    /// The processed text after applying all transforms or an error if any transform fails
    pub fn process(&self, text: &str) -> Result<String> {
        self.registry.process(text)
    }

    /// Gets a transform by name
    pub fn get_transform(&self, name: &str) -> Option<&Arc<Box<dyn Transform>>> {
        self.registry.get(name)
    }
}
