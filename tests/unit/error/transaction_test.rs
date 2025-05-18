use optionstratlib::error::TransactionError;
use std::error::Error;

#[test]
fn test_transaction_error_display() {
    // Create a new TransactionError with a test message
    let error = TransactionError {
        message: "Test error message".to_string(),
    };

    // Verify that the Display implementation works correctly
    assert_eq!(format!("{}", error), "TransactionError: Test error message");
}

#[test]
fn test_transaction_error_debug() {
    // Create a new TransactionError with a test message
    let error = TransactionError {
        message: "Test error message".to_string(),
    };

    // Verify that the Debug implementation works correctly
    assert!(format!("{:?}", error).contains("Test error message"));
}

#[test]
fn test_transaction_error_as_error() {
    // Create a new TransactionError
    let error = TransactionError {
        message: "Test error message".to_string(),
    };

    // Verify that it can be used as a Box<dyn Error>
    let boxed_error: Box<dyn Error> = Box::new(error);
    assert_eq!(
        boxed_error.to_string(),
        "TransactionError: Test error message"
    );
}
