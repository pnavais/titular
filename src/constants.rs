//! Module for shared constants used across the codebase

/// Unicode Private Use Area markers for padding
/// These markers are used to identify padding groups in the text
/// and are replaced during processing.
pub mod padding {
    /// Start marker for a padding group
    pub const START: char = '\u{F0000}';
    /// End marker for a padding group
    pub const END: char = '\u{F0001}';
}
