use super::*;
use serde_json::to_string_pretty;
use std::fmt;

/// Macro to implement Display for serializable types
macro_rules! impl_json_display {
    ($($t:ty),+) => {
        $(
            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    match to_string_pretty(self) {
                        Ok(pretty_json) => write!(f, "{}", pretty_json),
                        Err(e) => write!(f, "Error serializing to JSON: {}", e),
                    }
                }
            }
        )+
    }
}

// Use the macro to implement Display for multiple types at once
impl_json_display!(
    BullCallSpread,
    BearCallSpread,
    BullPutSpread,
    BearPutSpread,
    LongButterflySpread,
    ShortButterflySpread,
    IronCondor,
    IronButterfly,
    LongStraddle,
    ShortStraddle,
    LongStrangle,
    ShortStrangle,
    LongCall,
    LongPut,
    ShortCall,
    ShortPut,
    PoorMansCoveredCall,
    CallButterfly,
    CustomStrategy
);

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use crate::model::Position;
    use crate::{ExpirationDate, OptionStyle, OptionType, Options, Side, pos};
    use chrono::Utc;
    use rust_decimal_macros::dec;
    use serde_json;

    // Helper function to create a sample ShortStrangle for testing
    fn create_test_short_strangle() -> ShortStrangle {
        // Create sample option positions
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos!(110.0), // strike_price
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.25),  // implied_volatility
            pos!(1.0),   // quantity
            pos!(100.0), // underlying_price
            dec!(0.05),  // risk_free_rate
            OptionStyle::Call,
            pos!(0.02), // dividend_yield
            None,
        );

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            "TEST".to_string(),
            pos!(90.0), // strike_price
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.25),  // implied_volatility
            pos!(1.0),   // quantity
            pos!(100.0), // underlying_price
            dec!(0.05),  // risk_free_rate
            OptionStyle::Put,
            pos!(0.02), // dividend_yield
            None,
        );

        // Create positions for the options
        let short_call = Position::new(
            short_call_option,
            pos!(3.0),  // premium
            Utc::now(), // date
            pos!(0.5),  // open_fee
            pos!(0.5),  // close_fee
        );

        let short_put = Position::new(
            short_put_option,
            pos!(2.5),  // premium
            Utc::now(), // date
            pos!(0.5),  // open_fee
            pos!(0.5),  // close_fee
        );

        // Create a ShortStrangle with the positions
        ShortStrangle::get_strategy(&[short_put, short_call]).unwrap()
    }

    #[test]
    fn test_short_strangle_serialization() {
        let strangle = create_test_short_strangle();

        // Serialize to JSON string
        let serialized =
            serde_json::to_string(&strangle).expect("Failed to serialize ShortStrangle");

        // Basic verification that the serialized string contains expected values
        assert!(serialized.contains("Short Strangle"));
        assert!(serialized.contains("ShortStrangle"));
        assert!(serialized.contains("A short strangle involves selling an out-of-the-money"));
        assert!(serialized.contains("86.5"));
        assert!(serialized.contains("113.5"));
        assert!(serialized.contains("110")); // short_call strike
        assert!(serialized.contains("90")); // short_put strike
    }

    #[test]
    fn test_short_strangle_deserialization() {
        let original = create_test_short_strangle();

        // Serialize and then deserialize
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: ShortStrangle =
            serde_json::from_str(&serialized).expect("Failed to deserialize ShortStrangle");

        // Verify that the deserialized object matches the original
        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.kind, original.kind);
        assert_eq!(deserialized.description, original.description);
        assert_eq!(deserialized.break_even_points, original.break_even_points);
        assert_eq!(deserialized.to_string(), original.to_string());
    }

    #[test]
    fn test_short_strangle_json_structure() {
        let strangle = create_test_short_strangle();

        // Serialize to a pretty-printed JSON string for inspection
        let serialized = serde_json::to_string_pretty(&strangle).expect("Failed to serialize");

        // Parse back to a JSON Value to inspect structure
        let json_value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Verify top-level fields
        assert!(json_value.get("name").is_some());
        assert!(json_value.get("kind").is_some());
        assert!(json_value.get("description").is_some());
        assert!(json_value.get("break_even_points").is_some());
        assert!(json_value.get("short_call").is_some());
        assert!(json_value.get("short_put").is_some());

        // Verify short_call structure
        let short_call = json_value.get("short_call").unwrap();
        assert!(short_call.get("option").is_some());
        assert!(short_call.get("premium").is_some());

        // Verify short_put structure
        let short_put = json_value.get("short_put").unwrap();
        assert!(short_put.get("option").is_some());
        assert!(short_put.get("premium").is_some());
    }

    #[test]
    fn test_short_strangle_roundtrip_with_modified_values() {
        let mut strangle = create_test_short_strangle();

        // Modify some values
        strangle.name = "Modified Strangle".to_string();
        strangle.description = "A modified test short strangle".to_string();

        // Serialize and deserialize
        let serialized = serde_json::to_string(&strangle).expect("Failed to serialize");
        let deserialized: ShortStrangle =
            serde_json::from_str(&serialized).expect("Failed to deserialize ShortStrangle");

        // Verify the modified values are preserved
        assert_eq!(deserialized.name, "Modified Strangle");
        assert_eq!(deserialized.description, "A modified test short strangle");
    }

    #[test]
    fn test_short_strangle_deserialization_error_handling() {
        // Invalid JSON missing required fields
        let invalid_json = r#"{"name":"Invalid","kind":"ShortStrangle"}"#;

        // Should return an error because required fields are missing
        let result: Result<ShortStrangle, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());

        // Error should be about missing fields
        let error_message = format!("{}", result.unwrap_err());
        assert!(error_message.contains("missing field"));
    }

    #[test]
    fn test_short_strangle_with_custom_break_even_points() {
        // Create a short strangle with custom break-even points
        let mut strangle = create_test_short_strangle();
        strangle.break_even_points = vec![pos!(85.0), pos!(115.0)];

        // Serialize and deserialize
        let serialized = serde_json::to_string(&strangle).expect("Failed to serialize");
        let deserialized: ShortStrangle =
            serde_json::from_str(&serialized).expect("Failed to deserialize ShortStrangle");

        // Verify the custom break-even points were preserved
        assert_eq!(deserialized.break_even_points.len(), 2);
        assert_eq!(deserialized.break_even_points[0], pos!(85.0));
        assert_eq!(deserialized.break_even_points[1], pos!(115.0));
    }

    #[test]
    fn test_short_strangle_with_empty_break_even_points() {
        // Create a short strangle with empty break-even points
        let mut strangle = create_test_short_strangle();
        strangle.break_even_points = vec![];

        // Serialize and deserialize
        let serialized = serde_json::to_string(&strangle).expect("Failed to serialize");
        let deserialized: ShortStrangle =
            serde_json::from_str(&serialized).expect("Failed to deserialize ShortStrangle");

        // Verify the empty break-even points were preserved
        assert!(deserialized.break_even_points.is_empty());
    }
}
