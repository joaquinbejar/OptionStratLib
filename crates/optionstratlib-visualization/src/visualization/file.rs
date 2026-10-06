use std::io::Error as IoError;
use std::io::ErrorKind;
use std::path::Path;
use tracing::{debug, error, trace};

/// Prepares a path for writing by removing any existing file and creating necessary directories.
///
/// This function:
/// 1. Removes the file if it already exists, and accepts a file that is
///    already gone: the guarantee is that nothing sits at `path` afterwards
/// 2. Creates all parent directories if they don't exist
///
/// # Arguments
///
/// * `path` - The path where the file will be written
///
/// # Returns
///
/// * `Result<(), std::io::Error>` - Ok if successful, or an IoError if it failed
///
/// # Errors
///
/// Returns [`std::io::Error`] when the target file exists but cannot
/// be removed, or when the parent directory cannot be created
/// (typically `PermissionDenied`, `NotFound` on a broken symlink
/// ancestor, or the platform-specific disk-full case).
pub fn prepare_file_path(path: &Path) -> Result<(), IoError> {
    // What this has to guarantee is that nothing sits at `path`, so a file
    // that is already gone satisfies it. Testing `exists()` first and removing
    // afterwards is a race — anything else deleting the file in between makes
    // the removal fail with `NotFound` for a postcondition that already holds
    // — so the removal is attempted unconditionally and that one kind is
    // accepted. Two tests in one working directory are enough to lose the
    // race: it aborted a coverage run on `main` with `Failed to remove
    // existing file: multiple_curves_test.html`, kind `NotFound`.
    match std::fs::remove_file(path) {
        Ok(()) => trace!("Removed existing file: {}", path.display()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            trace!("Nothing to remove at: {}", path.display());
        }
        Err(e) => {
            error!("Failed to remove existing file: {}", path.display());
            return Err(IoError::new(
                e.kind(),
                format!("Failed to remove existing file: {}", path.display()),
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// The postcondition is "nothing sits at `path`", so a path that is
    /// already clear is a success, not a failure. Calling twice in a row is
    /// the sequential form of the race that aborted a coverage run: two tests
    /// sharing a working directory, one removing the file between the other's
    /// existence check and its removal.
    #[test]
    fn test_prepare_file_path_accepts_a_path_with_no_file() {
        let dir = std::env::temp_dir().join("optionstratlib_prepare_file_path");
        fs::create_dir_all(&dir).expect("the temp directory is writable");
        let path = dir.join("absent.html");
        let _ = fs::remove_file(&path);

        prepare_file_path(&path).expect("an absent file is already prepared");

        fs::write(&path, b"contents").expect("the temp directory is writable");
        prepare_file_path(&path).expect("an existing file is removed");
        assert!(!path.exists());

        prepare_file_path(&path).expect("preparing twice is not an error");
        assert!(!path.exists());
    }

    /// The parent directories are still created for a path that does not have
    /// them yet, which is the other half of what this does.
    #[test]
    fn test_prepare_file_path_creates_missing_parents() {
        let dir = std::env::temp_dir().join("optionstratlib_prepare_file_path/nested/deeper");
        let _ = fs::remove_dir_all(
            std::env::temp_dir().join("optionstratlib_prepare_file_path/nested"),
        );
        let path = dir.join("target.html");

        prepare_file_path(&path).expect("the parents are created");

        assert!(dir.exists());
    }
}
