use crate::context::Context;
use crate::error::{Error, Result};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Global context manager that provides thread-safe access to the shared context
pub struct ContextManager {
    context: Arc<RwLock<Context>>,
}

impl ContextManager {
    /// Gets a reference to the global context manager
    #[must_use] 
    pub fn get() -> &'static ContextManager {
        static INSTANCE: std::sync::LazyLock<ContextManager> =
            std::sync::LazyLock::new(|| ContextManager {
                context: Arc::new(RwLock::new(Context::new())),
            });
        &INSTANCE
    }

    /// Gets a clone of the shared context
    #[must_use] 
    pub fn share(&self) -> Arc<RwLock<Context>> {
        Arc::clone(&self.context)
    }

    /// Gets a mutable reference to the context
    /// This will wait for any other locks to be released
    ///
    /// # Errors
    /// Returns an error if the write lock is poisoned (another thread panicked while holding it).
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, Context>> {
        self.context
            .write()
            .map_err(|e| Error::Msg(format!("Failed to acquire write lock: {e}")))
    }

    /// Gets a read-only reference to the context
    /// This will wait for any write locks to be released
    ///
    /// # Errors
    /// Returns an error if the read lock is poisoned (another thread panicked while holding it).
    pub fn read(&self) -> Result<RwLockReadGuard<'_, Context>> {
        self.context
            .read()
            .map_err(|e| Error::Msg(format!("Failed to acquire read lock: {e}")))
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
    ///
    /// # Errors
    /// Returns an error if the write lock cannot be acquired (e.g. poisoned).
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
