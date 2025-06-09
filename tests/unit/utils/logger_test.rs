use optionstratlib::utils::logger::{setup_logger, setup_logger_with_level};
use std::env;
use std::sync::Mutex;

// We need a mutex to ensure tests don't interfere with each other when modifying env vars
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_setup_logger() {
    // Acquire the mutex to prevent other tests from interfering
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Test the default logger setup
    setup_logger();
    
    // Since the logger is initialized only once, we can't easily verify its state
    // But we can at least ensure the function doesn't panic
}

#[test]
fn test_setup_logger_with_level_debug() {
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Test with DEBUG level
    setup_logger_with_level("DEBUG");
    
    // We can't easily verify the logger's state, but we can ensure the function doesn't panic
}

#[test]
fn test_setup_logger_with_level_error() {
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Test with ERROR level
    setup_logger_with_level("ERROR");
}

#[test]
fn test_setup_logger_with_level_warn() {
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Test with WARN level
    setup_logger_with_level("WARN");
}

#[test]
fn test_setup_logger_with_level_trace() {
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Test with TRACE level
    setup_logger_with_level("TRACE");
}

#[test]
fn test_setup_logger_with_level_invalid() {
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Test with an invalid level (should default to INFO)
    setup_logger_with_level("INVALID");
}

#[test]
fn test_setup_logger_with_env_var() {
    let _lock = TEST_MUTEX.lock().unwrap();
    
    // Set the LOGLEVEL environment variable
    unsafe {
        env::set_var("LOGLEVEL", "DEBUG");
    }
    
    // Call setup_logger which should use the environment variable
    setup_logger();
    
    // Clean up
    unsafe {
        env::remove_var("LOGLEVEL");
    }
}
