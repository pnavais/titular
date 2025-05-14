use crate::error::*;
use nu_ansi_term::Color::{Blue, Yellow};
use std::path::PathBuf;
use std::process::Command;
use unicode_segmentation::UnicodeSegmentation;

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

/// Expands a string to the target width by repeating it until it reaches the target number of characters
pub fn expand_to_width(content: &str, target_width: usize) -> String {
    if content.is_empty() || target_width == 0 {
        return String::new();
    }

    let mut result = String::new();
    let mut graphemes = content.graphemes(true).cycle();

    for _ in 0..target_width {
        if let Some(g) = graphemes.next() {
            result.push_str(g);
        }
    }

    result
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
    fn test_expand_to_width() {
        // Test empty cases
        assert_eq!(expand_to_width("", 10), "");
        assert_eq!(expand_to_width("test", 0), "");
        assert_eq!(expand_to_width("", 0), "");

        // Test single character expansion
        assert_eq!(expand_to_width("X", 3), "XXX");
        assert_eq!(expand_to_width("X", 5), "XXXXX");

        // Test multi-character expansion
        assert_eq!(expand_to_width("XY", 3), "XYX");
        assert_eq!(expand_to_width("XY", 5), "XYXYX");

        // Test Unicode characters
        assert_eq!(expand_to_width("★", 3), "★★★");
        assert_eq!(expand_to_width("→", 4), "→→→→");
        assert_eq!(expand_to_width("ñ", 3), "ñññ");
        assert_eq!(expand_to_width("漢", 3), "漢漢漢");
        assert_eq!(expand_to_width("漢漢", 3), "漢漢漢");

        // Test emojis
        assert_eq!(expand_to_width("😊", 3), "😊😊😊");
        assert_eq!(expand_to_width("🌟", 4), "🌟🌟🌟🌟");
        assert_eq!(expand_to_width("👨‍👩‍👧‍👦", 2), "👨‍👩‍👧‍👦👨‍👩‍👧‍👦");
        assert_eq!(expand_to_width("👨‍👩‍👧‍👦", 3), "👨‍👩‍👧‍👦👨‍👩‍👧‍👦👨‍👩‍👧‍👦");
    }
}
