use nu_ansi_term::Color::{Blue, Yellow};
use std::process::Command;

pub const ROOT_PREFIX: &str = "\u{f115}";
pub const ELEMENT_PREFIX: &str = "\u{ea7b}";

/// Formats bytes into a human-readable string (KB, MB, etc.)
///
/// # Arguments
///
/// * `bytes` - The number of bytes to format
///
/// # Returns
///
/// A string representing the number of bytes in a human-readable format
///
/// # Examples
///
/// ```
/// use titular::utils::format_bytes;
///
/// assert_eq!(format_bytes(1024), "1.0 KB");
/// assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
/// assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
/// ```
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Performs cleanup operations when the program is interrupted
///
/// Currently the following operations are performed:
/// - Restores the cursor visibility in case of interruption
#[cfg(feature = "fetcher")]
pub fn cleanup() {
    let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
}

/// Checks if a command exists and is executable.
///
/// # Arguments
/// * `cmd` - The command to check
///
/// # Returns
/// `true` if the command exists and is executable, `false` otherwise
pub fn command_exists(cmd: &str) -> bool {
    if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    } else {
        // On Unix-like systems, use 'which' or 'command -v'
        Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

/// Prints a list of items in a tree-like format
///
/// # Arguments
/// * `items` - The list of items to print
/// * `element_name` - The name of the element to display before the tree
/// * `root` - The root path to display
///
/// # Examples
/// ```
/// use titular::utils::print_tree;
///
/// let items = vec!["file1.txt", "file2.txt", "file3.txt"];
/// print_tree(&items, "Found 3 files", "/path/to/files");
/// ```
pub fn print_tree<T: AsRef<str>>(items: &[T], element_name: &str, root: &str) {
    print_tree_with_prefixes(
        items,
        element_name,
        root,
        "\u{f115}",
        "\u{ea7b}",
        |s| Yellow.paint(s).to_string(),
        |s| Blue.paint(s).to_string(),
    );
}

/// Prints a list of items in a tree-like format
///
/// # Arguments
/// * `items` - The list of items to print
/// * `element_name` - The name of the element to display before the tree
/// * `root` - The root path to display
/// * `root_prefix` - The prefix to display before the root
/// * `element_prefix` - The prefix to display before the element
/// * `header_formatter` - A closure that formats the header text
/// * `root_formatter` - A closure that formats the root text
///
/// # Examples
/// ```
/// use titular::utils::print_tree_with_prefixes;
/// use nu_ansi_term::Color::Yellow;
///
/// let items = vec!["file1.txt", "file2.txt", "file3.txt"];
/// print_tree_with_prefixes(
///     &items,
///     "Found 3 files",
///     "/path/to/files",
///     "\u{f115}",
///     "\u{ea7b}",
///     |s| Yellow.paint(s).to_string(),
///     |s| s.to_string(),
/// );
/// ```
pub fn print_tree_with_prefixes<T: AsRef<str>, F, G>(
    items: &[T],
    element_name: &str,
    root: &str,
    root_prefix: &str,
    element_prefix: &str,
    header_formatter: F,
    root_formatter: G,
) where
    F: Fn(&str) -> String,
    G: Fn(&str) -> String,
{
    let num_items = items.len();
    if num_items >= 1 {
        let header = format!(
            "Found {} {}{}\n",
            num_items,
            element_name,
            if num_items > 1 { "s" } else { "" }
        );

        println!("{}", header_formatter(&header));
        println!("{}", root_formatter(&format!("{} {}", root_prefix, root)));
        // Handle all but the last file
        let (last, rest) = items.split_last().unwrap();

        for item in rest {
            println!("├── {} {}", element_prefix, item.as_ref());
        }

        println!("└── {} {}", element_prefix, last.as_ref());
    } else {
        println!(
            "{}",
            header_formatter(&format!("No {}s found", element_name))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_exists() {
        // Test with commands that should exist on all platforms
        assert!(command_exists("echo")); // Exists on all platforms

        // Platform-specific tests
        if cfg!(windows) {
            // Windows-specific commands
            assert!(command_exists("cmd"));
            assert!(command_exists("where"));
            assert!(command_exists("dir"));
        } else {
            // Unix-like systems (macOS, Linux, etc.)
            assert!(command_exists("ls"));
            assert!(command_exists("which"));
            assert!(command_exists("cat"));
        }

        // Test with commands that should not exist
        assert!(!command_exists("this_command_should_not_exist_123456789"));

        // Test with empty command
        assert!(!command_exists(""));

        // Test with spaces in command name
        assert!(!command_exists("command with spaces"));
    }

    #[test]
    fn test_format_bytes() {
        // Test bytes
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1), "1 B");
        assert_eq!(format_bytes(999), "999 B");

        // Test kilobytes
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB"); // 1.5 KB
        assert_eq!(format_bytes(1024 * 1024 - 1), "1024.0 KB"); // Just under 1 MB

        // Test megabytes
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 2), "2.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 - 1), "1024.0 MB"); // Just under 1 GB

        // Test gigabytes
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 2), "2.0 GB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 10), "10.0 GB");

        // Test edge cases
        // Instead of testing u64::MAX directly, test a large but manageable number
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024 * 16), "16384.0 GB");
    }
}
