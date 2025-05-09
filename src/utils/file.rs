use std::io::{Error as IoError};
use std::path::Path;
use tracing::{debug, trace};

/// Prepares a path for writing by removing any existing file and creating necessary directories.
///
/// This function:
/// 1. Removes the file if it already exists
/// 2. Creates all parent directories if they don't exist
///
/// # Arguments
///
/// * `path` - The path where the file will be written
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Ok if successful, or an IoError if it failed
pub fn prepare_file_path(path: &Path) -> Result<(), IoError> {
    // Remove file if it already exists
    if path.exists() {
        std::fs::remove_file(path)?;
        trace!("Removed existing file: {}", path.display());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
            debug!("Created directory: {}", path.display());
        }
    }

    Ok(())
}