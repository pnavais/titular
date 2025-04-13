use std::env;

use once_cell::sync::Lazy;

pub static DEBUG_ENABLED: Lazy<bool> = Lazy::new(|| {
    env::var("TITULAR_DEBUG").map_or(false, |log_level| log_level.eq("true") || log_level.eq("1"))
});

pub fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED
}

/// Logs a debug message with details.
///
/// # Arguments
///
/// * `message` - The main message to log.
/// * `details` - Additional details to include in the log.
///
/// # Returns
///
/// A formatted string containing the message and details.
pub fn debug_message<M, D>(message: M, details: D) -> String
where
    M: AsRef<str>,
    D: AsRef<str>,
{
    if *DEBUG_ENABLED {
        format!("{}{}", message.as_ref(), details.as_ref())
    } else {
        format!("{}", message.as_ref())
    }
}

/// Logs a debug message with optional formatted arguments.
///
/// # Arguments
///
/// * `fmt` - The format string for the debug message.
/// * `args` - Optional arguments to be formatted into the message.
///
/// # Examples
///
/// ```
/// use std::env;
/// use titular::debug;
///
/// // Enable debug logging for this test
/// env::set_var("TITULAR_DEBUG", "true");
///
/// // These will print in yellow when debug is enabled
/// debug!("Operation completed");
/// debug!("User {} logged in from {}", "alice", "192.168.1.1");
///
/// // Clean up
/// env::remove_var("TITULAR_DEBUG");
/// ```
#[macro_export]
macro_rules! debug {
    ($fmt:expr) => {
        if *$crate::log::DEBUG_ENABLED {
            println!("{}", nu_ansi_term::Color::Yellow.paint(format!("{}", $fmt)));
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        if *$crate::log::DEBUG_ENABLED {
            println!("{}", nu_ansi_term::Color::Yellow.paint(format!($fmt, $($arg)*)));
        }
    };
}
