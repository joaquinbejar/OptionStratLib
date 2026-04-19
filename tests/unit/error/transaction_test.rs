use optionstratlib::OptionType;
use optionstratlib::error::TransactionError;
use std::error::Error;

#[test]
fn test_transaction_error_not_implemented_display() {
    let error = TransactionError::not_implemented("add_transaction", "Position");
    assert_eq!(
        format!("{error}"),
        "add_transaction not implemented for Position"
    );
}

#[test]
fn test_transaction_error_unsupported_option_type_display() {
    let error = TransactionError::unsupported_option_type(OptionType::American);
    assert!(
        format!("{error}")
            .to_lowercase()
            .contains("unsupported option type")
    );
    assert!(format!("{error}").contains("American"));
}

#[test]
fn test_transaction_error_other_display() {
    let error = TransactionError::other("boom");
    assert_eq!(format!("{error}"), "transaction error: boom");
}

#[test]
fn test_transaction_error_debug() {
    let error = TransactionError::not_implemented("get_transactions", "Position");
    assert!(format!("{error:?}").contains("get_transactions"));
    assert!(format!("{error:?}").contains("Position"));
}

#[test]
fn test_transaction_error_as_error_trait_object() {
    let error = TransactionError::other("boxed");
    let boxed_error: Box<dyn Error> = Box::new(error);
    assert_eq!(boxed_error.to_string(), "transaction error: boxed");
}
