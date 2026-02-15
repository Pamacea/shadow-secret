// Shadow Secret - Cleaner Module
//
// This module handles cleanup operations including:
// - Signal handling (SIGINT, SIGTERM)
// - Process termination (node, openclaw)
// - File restoration from backups
// - Panic handling

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::sync::{Mutex, OnceLock};
use sysinfo::System;

/// Global storage for file backups
static BACKUPS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

/// Initialize the global backups storage
fn init_backups() -> &'static Mutex<HashMap<String, String>> {
    BACKUPS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Register a backup for a file
fn register_backup_global(path: String, content: String) {
    if let Ok(mut backups) = init_backups().lock() {
        backups.insert(path, content);
    }
}

/// Get all backups and clear the storage
fn take_all_backups() -> HashMap<String, String> {
    if let Ok(mut backups) = init_backups().lock() {
        std::mem::take(&mut *backups)
    } else {
        HashMap::new()
    }
}

/// Check if there are any backups registered
fn backups_is_empty() -> bool {
    init_backups()
        .lock()
        .map(|b| b.is_empty())
        .unwrap_or(true)
}

/// Register a backup for a file to be restored on cleanup
///
/// # Arguments
/// * `path` - The file path to backup
/// * `content` - The original content of the file
///
/// # Example
/// ```no_run
/// use shadow_secret::cleaner::register_backup;
///
/// register_backup("/path/to/file.yaml", "original content");
/// ```
pub fn register_backup(path: &str, content: &str) {
    register_backup_global(path.to_string(), content.to_string());
}

/// Setup signal handlers for graceful shutdown
///
/// This registers handlers for:
/// - SIGINT (Ctrl+C)
/// - SIGTERM (termination signal)
/// - Panic handler
///
/// # Example
/// ```no_run
/// use shadow_secret::cleaner::setup_signal_handlers;
///
/// setup_signal_handlers();
/// ```
pub fn setup_signal_handlers() {
    // Setup Ctrl+C handler
    if let Err(e) = ctrlc::set_handler(|| {
        eprintln!("\n🛑 Received SIGINT (Ctrl+C)");
        cleanup_and_restore();
        std::process::exit(0);
    }) {
        eprintln!("⚠️  Failed to set SIGINT handler: {}", e);
    }

    // Setup panic handler
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("\n💥 PANIC: {}", panic_info);
        cleanup_and_restore();
    }));

    eprintln!("✓ Signal handlers registered");
}

/// Perform complete cleanup and restoration
///
/// This function is idempotent - safe to call multiple times.
/// It will:
/// 1. Kill blocking processes (node, openclaw)
/// 2. Restore all files from backups
/// 3. Clear the backups map
///
/// # Example
/// ```no_run
/// use shadow_secret::cleaner::cleanup_and_restore;
///
/// cleanup_and_restore();
/// ```
pub fn cleanup_and_restore() {
    if backups_is_empty() {
        eprintln!("📭 No backups to restore");
        return;
    }

    eprintln!("🧹 Starting cleanup...");

    // Step 1: Kill blocking processes
    if let Err(e) = kill_blocking_processes() {
        eprintln!("⚠️  Failed to kill processes: {}", e);
    }

    // Step 2: Restore all files
    let backups = take_all_backups();
    let total = backups.len();
    let mut restored = 0;

    for (path, content) in backups {
        match restore_file(&path, &content) {
            Ok(_) => {
                restored += 1;
                eprintln!("  ✓ Restored: {}", path);
            }
            Err(e) => {
                eprintln!("  ✗ Failed to restore {}: {}", path, e);
            }
        }
    }

    eprintln!("✅ Cleanup complete: {}/{} files restored", restored, total);
}

/// Kill blocking processes (node, openclaw)
///
/// Uses sysinfo to find and terminate processes that might be
/// blocking access to files or resources.
///
/// # Errors
/// Returns an error if process enumeration fails
///
/// # Example
/// ```no_run
/// use shadow_secret::cleaner::kill_blocking_processes;
///
/// if let Err(e) = kill_blocking_processes() {
///     eprintln!("Failed to kill processes: {}", e);
/// }
/// ```
pub fn kill_blocking_processes() -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let targets = ["node", "openclaw"];
    let mut killed = 0;

    for (pid, process) in sys.processes() {
        let name = process.name();
        let name_str = name.to_string_lossy();

        if targets.contains(&name_str.as_ref()) {
            eprintln!("  🔪 Killing process: {} (PID: {})", name_str, pid);
            if process.kill() {
                killed += 1;
            } else {
                eprintln!("  ⚠️  Failed to kill {} (PID: {})", name_str, pid);
            }
        }
    }

    if killed > 0 {
        eprintln!("✓ Killed {} blocking process(es)", killed);
    } else {
        eprintln!("✓ No blocking processes found");
    }

    Ok(())
}

/// Restore a file from its backup content
///
/// # Arguments
/// * `original_path` - The path to the file to restore
/// * `original_content` - The original content to write back
///
/// # Errors
/// Returns an error if the file cannot be written
///
/// # Example
/// ```no_run
/// use shadow_secret::cleaner::restore_file;
///
/// if let Err(e) = restore_file("/path/to/file.yaml", "original content") {
///     eprintln!("Failed to restore: {}", e);
/// }
/// ```
fn restore_file(original_path: &str, original_content: &str) -> Result<()> {
    fs::write(original_path, original_content)
        .with_context(|| format!("Failed to restore file: {}", original_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_register_and_restore_backup() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let original_content = "original content";
        let modified_content = "modified content";

        // Write modified content
        fs::write(path, modified_content).unwrap();

        // Register backup
        register_backup(path, original_content);

        // Perform cleanup
        cleanup_and_restore();

        // Verify restoration
        let restored = fs::read_to_string(path).unwrap();
        assert_eq!(restored, original_content);
    }

    #[test]
    fn test_cleanup_idempotent() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let original_content = "original content";

        fs::write(path, "modified").unwrap();
        register_backup(path, original_content);

        // Call cleanup multiple times - should not panic
        cleanup_and_restore();
        cleanup_and_restore();
        cleanup_and_restore();

        // File should be restored after first cleanup
        // Subsequent cleanups should be no-ops (no backups to restore)
        let restored = fs::read_to_string(path).unwrap();
        assert_eq!(restored, original_content);
    }

    #[test]
    fn test_restore_file_with_invalid_path() {
        let result = restore_file("/nonexistent/path/to/file.txt", "content");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_backups_cleanup() {
        // Should not panic when no backups registered
        cleanup_and_restore();
    }

    #[test]
    fn test_multiple_backups() {
        let temp1 = NamedTempFile::new().unwrap();
        let temp2 = NamedTempFile::new().unwrap();
        let path1 = temp1.path().to_str().unwrap();
        let path2 = temp2.path().to_str().unwrap();

        fs::write(path1, "modified1").unwrap();
        fs::write(path2, "modified2").unwrap();

        register_backup(path1, "original1");
        register_backup(path2, "original2");

        cleanup_and_restore();

        assert_eq!(fs::read_to_string(path1).unwrap(), "original1");
        assert_eq!(fs::read_to_string(path2).unwrap(), "original2");
    }
}
