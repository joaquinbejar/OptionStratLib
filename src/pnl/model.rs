/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/2/25
******************************************************************************/
use serde::{Deserialize, Serialize, Serializer};

/// Represents a range of Profit and Loss (PnL) values for bucketing or categorizing financial results.
///
/// This structure defines a discrete interval of PnL values that can be used for:
/// - Creating histograms of trading results
/// - Defining profit/loss categories for analysis
/// - Setting up thresholds for performance metrics
/// - Grouping trading outcomes for statistical analysis
///
/// The range is defined as [lower, upper) where lower is inclusive and upper is exclusive.
/// PnL values are represented as integer values (i32) rather than decimals for efficient
/// bucketing and categorization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct PnLRange {
    /// Lower bound of this PnL bucket (inclusive)
    pub lower: i32,
    /// Upper bound of this PnL bucket (exclusive)
    pub upper: i32,
}

/// Implements serialization for `PnLRange` using a custom string format.
///
/// This implementation serializes a `PnLRange` instance as a string in the format
/// "[lower, upper)" where:
/// - `lower` is the inclusive lower bound of the range
/// - `upper` is the exclusive upper bound of the range
///
/// This format provides a clear, human-readable representation of the profit and loss
/// range that preserves the mathematical half-open interval notation, making it ideal
/// for display in reports, logs, or API responses.
///
/// # Example
///
/// A `PnLRange` with `lower = 100` and `upper = 200` will be serialized as "[100, 200)".
///
/// # Implementation Details
///
/// This custom serialization is needed because the default derived implementation would
/// serialize the struct as a JSON object with separate fields, while this implementation
/// provides a more compact string representation.
impl Serialize for PnLRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("[{}, {})", self.lower, self.upper))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::collections::HashMap;

    #[test]
    fn test_pnl_range_serialization() {
        // Create a PnLRange instance
        let range = PnLRange {
            lower: -10,
            upper: 20,
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&range).unwrap();

        // Check that it serializes as expected string format
        assert_eq!(serialized, "\"[-10, 20)\"");
    }

    #[test]
    fn test_pnl_range_as_hashmap_key() {
        // Create a HashMap with PnLRange as keys
        let mut map = HashMap::new();

        map.insert(
            PnLRange {
                lower: -10,
                upper: 0,
            },
            rust_decimal::Decimal::new(25, 2), // 0.25
        );

        map.insert(
            PnLRange {
                lower: 0,
                upper: 10,
            },
            rust_decimal::Decimal::new(50, 2), // 0.50
        );

        map.insert(
            PnLRange {
                lower: 10,
                upper: 20,
            },
            rust_decimal::Decimal::new(25, 2), // 0.25
        );

        // Serialize the map to JSON
        let serialized = serde_json::to_string(&map).unwrap();

        // The serialized string should contain each range as a string key
        assert!(serialized.contains("\"[-10, 0)\""));
        assert!(serialized.contains("\"[0, 10)\""));
        assert!(serialized.contains("\"[10, 20)\""));
    }

    #[test]
    fn test_pnl_range_in_complex_structure() {
        // Create a more complex structure containing PnLRange
        #[derive(Serialize)]
        struct SimulationResult {
            name: String,
            distribution: HashMap<PnLRange, rust_decimal::Decimal>,
        }

        let mut distribution = HashMap::new();
        distribution.insert(
            PnLRange {
                lower: -5,
                upper: 5,
            },
            rust_decimal::Decimal::new(100, 2), // 1.00
        );

        let result = SimulationResult {
            name: "Test Simulation".to_string(),
            distribution,
        };

        // Serialize to JSON
        let serialized = serde_json::to_string_pretty(&result).unwrap();

        // Verify the format of the serialized JSON
        assert!(serialized.contains("\"name\": \"Test Simulation\""));
        assert!(serialized.contains("\"distribution\": {"));
        assert!(serialized.contains("\"[-5, 5)\""));
    }

    #[test]
    fn test_pnl_range_with_large_values() {
        // Test with very large integer values
        let range = PnLRange {
            lower: i32::MIN,
            upper: i32::MAX,
        };

        // Serialize to JSON
        let serialized = serde_json::to_string(&range).unwrap();

        // Verify the format
        assert_eq!(serialized, format!("\"[{}, {})\"", i32::MIN, i32::MAX));
    }

    #[test]
    fn test_pnl_range_array_serialization() {
        // Test serializing an array of PnLRange
        let ranges = vec![
            PnLRange {
                lower: -10,
                upper: 0,
            },
            PnLRange {
                lower: 0,
                upper: 10,
            },
            PnLRange {
                lower: 10,
                upper: 20,
            },
        ];

        // Serialize to JSON
        let serialized = serde_json::to_string(&ranges).unwrap();

        // Verify the format
        assert_eq!(serialized, "[\"[-10, 0)\",\"[0, 10)\",\"[10, 20)\"]");
    }

    #[test]
    fn test_pnl_range_deserialization_error() {
        // This test verifies that we can't directly deserialize a string
        // back into PnLRange with the current implementation
        let json_str = "\"[-10, 20)\"";

        let result = serde_json::from_str::<PnLRange>(json_str);

        // It should fail since we haven't implemented a custom deserializer
        assert!(result.is_err());
    }

    #[test]
    fn test_pnl_range_equality() {
        // Test that PnLRange equality works correctly
        let range1 = PnLRange {
            lower: -10,
            upper: 20,
        };
        let range2 = PnLRange {
            lower: -10,
            upper: 20,
        };
        let range3 = PnLRange {
            lower: 0,
            upper: 10,
        };

        assert_eq!(range1, range2);
        assert_ne!(range1, range3);

        // Test HashMap key functionality
        let mut map = HashMap::new();
        map.insert(range1.clone(), true);

        assert!(map.contains_key(&range2));
        assert!(!map.contains_key(&range3));
    }
}
