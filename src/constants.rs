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

pub mod template {
    /// Default template content (basic template)
    pub const DEFAULT_TEMPLATE: &str = "[details]\n\
                                name    = \"@name\"\n\
                                version = \"1.0\"\n\
                                author  = \"@author\"\n\
                                url     = \"@url\"\n\n\
                                [vars]\n\
                                main_color = \"green\"\n\
                                message_color = \"$main_color\"\n\
                                white=\"RGB(255,255,255)\"\n\
                                f=\"*\"\n\
                                fb=\"$f\"\n\
                                fe=\"${f2:f}\"\n\
                                c=\"$main_color\"\n\
                                c2=\"$message_color\"\n\
                                c3=\"$main_color\"\n\n\
                                [pattern]\n\
                                data = \"{{ fb | color(name=c) | pad }}{{ m | color(name=c2) }}{{ fe | color(name=c3) | pad }}\"\n";

    /// Default template file extension
    pub const DEFAULT_TEMPLATE_EXT: &str = ".tl";

    /// Default template name
    pub const DEFAULT_TEMPLATE_NAME: &str = "basic";

    /// Default theme for display
    pub const DEFAULT_THEME: &str = "base16-ocean.dark";

    /// Default time format
    pub const DEFAULT_TIME_FORMAT: &str = "%H:%M:%S";

    #[cfg(feature = "fetcher")]
    /// Default remote repository for templates
    pub const DEFAULT_REMOTE_REPO: &str = "github:pnavais/titular/templates";
}
