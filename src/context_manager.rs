use crate::context::Context;
use crate::error::{Error, Result};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Global context manager that provides thread-safe access to the shared context
pub struct ContextManager {
    context: Arc<RwLock<Context>>,
}

impl ContextManager {
    /// Gets a reference to the global context manager
    pub fn get() -> &'static ContextManager {
        static INSTANCE: once_cell::sync::Lazy<ContextManager> =
            once_cell::sync::Lazy::new(|| ContextManager {
                context: Arc::new(RwLock::new(Context::new())),
            });
        &INSTANCE
    }

    /// Gets a clone of the shared context
    pub fn share(&self) -> Arc<RwLock<Context>> {
        Arc::clone(&self.context)
    }

    /// Gets a mutable reference to the context
    /// This will wait for any other locks to be released
    pub fn write(&self) -> Result<RwLockWriteGuard<Context>> {
        self.context
            .write()
            .map_err(|e| Error::Msg(format!("Failed to acquire write lock: {}", e)))
    }

    /// Gets a read-only reference to the context
    /// This will wait for any write locks to be released
    pub fn read(&self) -> Result<RwLockReadGuard<Context>> {
        self.context
            .read()
            .map_err(|e| Error::Msg(format!("Failed to acquire read lock: {}", e)))
    }

    /// Initializes the context manager with a new context
    pub fn init(context: Context) {
        let manager = Self::get();
        if let Ok(mut ctx) = manager.write() {
            *ctx = context;
        }
    }

    /// Updates the context with new data and returns immediately
    /// The lock is automatically released when the operation is complete
    pub fn update<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Context),
    {
        let mut ctx = self.write()?;
        f(&mut ctx);
        Ok(())
    }
}

impl From<Context> for ContextManager {
    fn from(context: Context) -> Self {
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}
