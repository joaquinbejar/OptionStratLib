use std::io::Error as IoError;
use std::path::Path;
use tracing::{debug, error, trace};

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
        match std::fs::remove_file(path) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to remove existing file: {}", path.display());
                return Err(IoError::new(
                    e.kind(),
                    format!("Failed to remove existing file: {}", path.display()),
                ));
            }
        };
        trace!("Removed existing file: {}", path.display());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        match std::fs::create_dir_all(parent) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to create parent directories: {}", path.display());
                return Err(IoError::new(
                    e.kind(),
                    format!("Failed to create parent directories: {}", path.display()),
                ));
            }
        };
        debug!("Created directory: {}", path.display());
    }

    Ok(())
}
