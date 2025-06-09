use optionstratlib::utils::file::prepare_file_path;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_prepare_file_path_new_file() {
    // Create a temporary directory that will be deleted when the test completes
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("test_file.txt");
    
    // The file doesn't exist yet, so prepare_file_path should just ensure the directory exists
    let result = prepare_file_path(&file_path);
    assert!(result.is_ok(), "prepare_file_path should succeed for a new file");
    
    // The parent directory should exist
    assert!(file_path.parent().unwrap().exists(), "Parent directory should exist");
    
    // The file itself should not exist yet
    assert!(!file_path.exists(), "File should not exist yet");
}

#[test]
fn test_prepare_file_path_existing_file() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let file_path = temp_dir.path().join("existing_file.txt");
    
    // Create the file first
    fs::write(&file_path, "test content").expect("Failed to write test file");
    assert!(file_path.exists(), "File should exist before test");
    
    // Now prepare the file path, which should remove the existing file
    let result = prepare_file_path(&file_path);
    assert!(result.is_ok(), "prepare_file_path should succeed for an existing file");
    
    // The file should no longer exist
    assert!(!file_path.exists(), "File should have been removed");
}

#[test]
fn test_prepare_file_path_nested_directories() {
    // Create a temporary directory
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let nested_path = temp_dir.path().join("nested/dirs/for/test_file.txt");
    
    // The nested directories don't exist yet
    assert!(!nested_path.parent().unwrap().exists(), "Nested directories should not exist yet");
    
    // Prepare the file path, which should create all parent directories
    let result = prepare_file_path(&nested_path);
    assert!(result.is_ok(), "prepare_file_path should succeed for nested directories");
    
    // All parent directories should now exist
    assert!(nested_path.parent().unwrap().exists(), "Nested directories should have been created");
}

#[test]
fn test_prepare_file_path_permission_error() {
    // This test is only meaningful on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temp directory");
        
        // Create a subdirectory with no write permissions
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).expect("Failed to create readonly directory");
        
        // Create a file first, then make the directory read-only
        let file_path = readonly_dir.join("test_file.txt");
        fs::write(&file_path, "test content").expect("Failed to write test file");
        
        // Remove write permissions on the directory
        let metadata = fs::metadata(&readonly_dir).expect("Failed to get metadata");
        let mut perms = metadata.permissions();
        perms.set_mode(0o555); // read and execute, but no write
        fs::set_permissions(&readonly_dir, perms).expect("Failed to set permissions");
        
        // Now try to prepare the file path (this should fail when trying to remove the file)
        let result = prepare_file_path(&file_path);
        assert!(result.is_err(), "prepare_file_path should fail when removing file in readonly directory");
    }
}
