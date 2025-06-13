use crate::constants::template::DEFAULT_TIME_FORMAT;
use crate::error::*;
use chrono::{DateTime, Local};
use nu_ansi_term::Color::{Blue, Yellow};
use num;
use std::path::PathBuf;
use std::process::Command;

pub const ROOT_PREFIX: &str = "\u{f115}";
pub const ELEMENT_PREFIX: &str = "\u{ea7b}";

/// Safely formats a DateTime using the provided format string.
/// If the format string is invalid, returns a default format (%H:%M:%S).
///
/// # Arguments
///
/// * `dt` - The DateTime to format
/// * `format` - The format string to use for time formatting
///
/// # Returns
///
/// A string containing the formatted time
pub fn safe_time_format(dt: &DateTime<Local>, format: &str) -> String {
    // Parse the format string first to validate it
    let items: Vec<_> = chrono::format::strftime::StrftimeItems::new(format).collect();

    // If any item is an error, use default format
    if items
        .iter()
        .any(|item| matches!(item, chrono::format::Item::Error))
    {
        eprintln!(
            "{}",
            Yellow.paint(format!(
                "WARNING: Invalid time format specified \"{}\"",
                format
            ))
        );
        return dt.format(DEFAULT_TIME_FORMAT).to_string();
    }

    // Use the validated format items
    dt.format_with_items(items.into_iter()).to_string()
}

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
/// - Terminates the program with a proper exit code
#[cfg(feature = "fetcher")]
pub fn cleanup() {
    let _ = crossterm::execute!(std::io::stdout(), crossterm::cursor::Show);
    std::process::exit(1);
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

/// Creates a backup of an existing file before downloading a new version.
/// The backup will have the same name as the original file but with a .bak extension.
///
/// # Arguments
/// * `path` - The path of the file to backup.
///
/// # Returns
/// Returns a Result indicating success or failure.
pub fn create_backup(path: &PathBuf) -> Result<()> {
    let backup_path = if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        path.with_extension(format!("{}.bak", ext))
    } else {
        path.with_extension("bak")
    };
    std::fs::rename(path, &backup_path)?;
    Ok(())
}

/// Restores a backup file by renaming it back to its original name.
///
/// # Arguments
/// * `path` - The path of the file to restore from backup.
///
/// # Returns
/// Returns a Result indicating success or failure.
pub fn restore_backup(path: &PathBuf) -> Result<()> {
    let backup_path = if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        path.with_extension(format!("{}.bak", ext))
    } else {
        path.with_extension("bak")
    };
    if backup_path.exists() {
        std::fs::rename(&backup_path, path)?;
    }
    Ok(())
}

/// Removes a backup file if present.
///
/// # Arguments
/// * `path` - The path of the backup file to remove.
///
/// # Returns
/// Returns a Result indicating success or failure.
pub fn remove_backup(path: &PathBuf) -> Result<()> {
    let backup_path = if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
        path.with_extension(format!("{}.bak", ext))
    } else {
        path.with_extension("bak")
    };
    if backup_path.exists() {
        std::fs::remove_file(&backup_path)?;
    }
    Ok(())
}

/// Safely parses a string into a numeric type, clamping the value to the type's bounds
/// if it exceeds them.
///
/// # Arguments
///
/// * `s` - The string to parse
///
/// # Returns
///
/// The parsed value, clamped to the type's bounds if necessary, or 0 if the input is not numeric
///
/// # Examples
///
/// ```
/// use titular::utils::safe_parse;
///
/// assert_eq!(safe_parse::<u8>("255"), 255);
/// assert_eq!(safe_parse::<u8>("256"), 255); // Clamped to u8::MAX
/// assert_eq!(safe_parse::<u8>("-1"), 0);    // Clamped to u8::MIN
/// assert_eq!(safe_parse::<u8>("abc"), 0);   // Non-numeric returns 0
/// assert_eq!(safe_parse::<i8>("-128"), -128);
/// assert_eq!(safe_parse::<i8>("-129"), -128); // Clamped to i8::MIN
/// ```
pub fn safe_parse<T>(s: &str) -> T
where
    T: std::str::FromStr
        + std::cmp::PartialOrd
        + Copy
        + num::Bounded
        + num::Zero
        + std::fmt::Display,
{
    // First try to parse as i64 to handle any numeric value
    match s.parse::<i64>() {
        Ok(val) => {
            if val < 0 {
                if T::min_value() < T::zero() {
                    // For signed types, clamp to min_value
                    T::min_value()
                } else {
                    // For unsigned types, clamp to zero
                    T::zero()
                }
            } else if val
                > T::max_value()
                    .to_string()
                    .parse::<i64>()
                    .unwrap_or(i64::MAX)
            {
                T::max_value()
            } else {
                // Now we know it's a valid number in range, parse as the target type
                s.parse::<T>().unwrap_or(T::zero())
            }
        }
        Err(_) => T::zero(),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

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

    #[test]
    fn test_backup_operations() -> Result<()> {
        // Create a temporary file with some content
        let mut temp_file = NamedTempFile::new()?;
        let original_path = temp_file.path().to_path_buf();
        writeln!(temp_file, "Original content")?;
        temp_file.flush()?;

        // Create a new path with a known extension
        let new_path = original_path.with_extension("txt");
        std::fs::rename(&original_path, &new_path)?;

        // Test create_backup
        create_backup(&new_path)?;
        let backup_path = new_path.with_extension("txt.bak");
        assert!(
            backup_path.exists(),
            "Backup file should exist after create_backup"
        );
        assert!(
            !new_path.exists(),
            "Original file should not exist after create_backup"
        );

        // Test restore_backup
        restore_backup(&new_path)?;
        assert!(
            new_path.exists(),
            "Original file should exist after restore_backup"
        );
        assert!(
            !backup_path.exists(),
            "Backup file should not exist after restore_backup"
        );

        // Test remove_backup
        create_backup(&new_path)?;
        assert!(
            backup_path.exists(),
            "Backup file should exist before remove_backup"
        );
        remove_backup(&new_path)?;
        assert!(
            !backup_path.exists(),
            "Backup file should not exist after remove_backup"
        );

        Ok(())
    }

    #[test]
    fn test_backup_operations_with_extension() -> Result<()> {
        // Create a temporary file with an extension
        let mut temp_file = NamedTempFile::new()?;
        let original_path = temp_file.path().to_path_buf();
        let new_path = original_path.with_extension("txt");
        std::fs::rename(&original_path, &new_path)?;
        writeln!(temp_file, "Original content")?;
        temp_file.flush()?;

        // Test create_backup with extension
        create_backup(&new_path)?;
        let backup_path = new_path.with_extension("txt.bak");
        assert!(
            backup_path.exists(),
            "Backup file should exist after create_backup"
        );
        assert!(
            !new_path.exists(),
            "Original file should not exist after create_backup"
        );

        // Test restore_backup with extension
        restore_backup(&new_path)?;
        assert!(
            new_path.exists(),
            "Original file should exist after restore_backup"
        );
        assert!(
            !backup_path.exists(),
            "Backup file should not exist after restore_backup"
        );

        Ok(())
    }

    #[test]
    fn test_backup_operations_without_extension() -> Result<()> {
        // Create a temporary file without an extension
        let mut temp_file = NamedTempFile::new()?;
        let original_path = temp_file.path().to_path_buf();
        writeln!(temp_file, "Original content")?;
        temp_file.flush()?;

        // Rename the file to remove the extension
        let new_path = original_path.with_extension("");
        std::fs::rename(&original_path, &new_path)?;

        // Test create_backup without extension
        create_backup(&new_path)?;
        let backup_path = new_path.with_extension("bak");
        assert!(
            backup_path.exists(),
            "Backup file should exist after create_backup"
        );
        assert!(
            !new_path.exists(),
            "Original file should not exist after create_backup"
        );

        // Test restore_backup without extension
        restore_backup(&new_path)?;
        assert!(
            new_path.exists(),
            "Original file should exist after restore_backup"
        );
        assert!(
            !backup_path.exists(),
            "Backup file should not exist after restore_backup"
        );

        Ok(())
    }

    #[test]
    fn test_safe_parse() {
        // Test u8 parsing
        assert_eq!(safe_parse::<u8>("0"), 0);
        assert_eq!(safe_parse::<u8>("255"), 255);
        assert_eq!(safe_parse::<u8>("256"), 255); // Clamped to u8::MAX
        assert_eq!(safe_parse::<u8>("-1"), 0); // Clamped to u8::MIN
        assert_eq!(safe_parse::<u8>("abc"), 0); // Non-numeric returns 0

        // Test i8 parsing
        assert_eq!(safe_parse::<i8>("-128"), -128);
        assert_eq!(safe_parse::<i8>("127"), 127);
        assert_eq!(safe_parse::<i8>("-129"), -128); // Clamped to i8::MIN
        assert_eq!(safe_parse::<i8>("128"), 127); // Clamped to i8::MAX
        assert_eq!(safe_parse::<i8>("abc"), 0); // Non-numeric returns 0

        // Test u16 parsing
        assert_eq!(safe_parse::<u16>("0"), 0);
        assert_eq!(safe_parse::<u16>("65535"), 65535);
        assert_eq!(safe_parse::<u16>("65536"), 65535); // Clamped to u16::MAX
        assert_eq!(safe_parse::<u16>("-1"), 0); // Clamped to u16::MIN
        assert_eq!(safe_parse::<u16>("abc"), 0); // Non-numeric returns 0

        // Test i16 parsing
        assert_eq!(safe_parse::<i16>("-32768"), -32768);
        assert_eq!(safe_parse::<i16>("32767"), 32767);
        assert_eq!(safe_parse::<i16>("-32769"), -32768); // Clamped to i16::MIN
        assert_eq!(safe_parse::<i16>("32768"), 32767); // Clamped to i16::MAX
        assert_eq!(safe_parse::<i16>("abc"), 0); // Non-numeric returns 0
    }

    #[test]
    fn test_safe_time_format() {
        let now = Local::now();
        let default_format = safe_time_format(&now, DEFAULT_TIME_FORMAT);
        assert!(!default_format.is_empty()); // Should not be empty
        assert!(
            default_format.contains(':'),
            "Default format should contain time separators"
        );

        let date_format = safe_time_format(&now, "%Y-%m-%d");
        assert_eq!(date_format.len(), 10); // e.g. "2024-03-20"

        let invalid_format = safe_time_format(&now, "%)H");
        assert!(!default_format.is_empty()); // Should not be empty
        assert!(
            invalid_format.contains(':'),
            "Invalid format should contain time separators"
        );

        assert!(!default_format.is_empty()); // Should not be empty

        let no_time_format = safe_time_format(&now, "no_time_format");
        assert!(!no_time_format.is_empty()); // Should fall back to default format
        assert_eq!(no_time_format.as_str(), "no_time_format"); // Should fall back to default format
    }
}
