/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/2/25
******************************************************************************/
use serde::{Deserialize, Serialize};

/// Represents the basic axis types used in financial option analysis and visualization.
///
/// This enum defines the fundamental dimensions along which option characteristics can be analyzed,
/// plotted, or calculated. Each variant represents a specific financial metric or parameter that
/// is commonly used in options trading and risk management.
///
/// These axis types can be used to:
/// - Generate sensitivity analysis charts
/// - Create parameter surfaces (e.g., volatility surface)
/// - Define dimensions for multi-dimensional analysis
/// - Specify which metrics to calculate or display
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy, Hash, Eq)]
pub enum BasicAxisTypes {
    /// Sensitivity of option price to changes in underlying price (first derivative)
    Delta,

    /// Rate of change of Delta with respect to underlying price (second derivative)
    Gamma,

    /// Sensitivity of option price to time decay
    Theta,

    /// Sensitivity of option price to changes in volatility
    Vega,

    /// Implied or historical volatility of the underlying asset
    Volatility,

    /// Price of the option contract
    Price,

    /// Price of the underlying asset
    UnderlyingPrice,

    /// Strike price of the option
    Strike,

    /// Time until expiration of the option
    Expiration,
}

/// Iterator for traversing the basic axis types.
///
/// This iterator provides sequential access to each basic axis type defined in the system.
/// It maintains an internal index to track the current position in the iteration sequence.
///
/// The iterator is particularly useful when you need to enumerate or process all available
/// axis types in a systematic way, such as when generating charts, configuring visualization
/// components, or performing operations that need to be applied to all axis types.
///
pub struct BasicAxisTypesIter {
    /// Current position in the iteration sequence.
    /// Starts at 0 and increments with each call to `next()`.
    index: usize,
}

/// Provides implementation for the BasicAxisTypes enumeration.
impl BasicAxisTypes {
    /// A constant array containing all variants of the BasicAxisTypes enum.
    ///
    /// This array allows efficient access to all available axis types without
    /// needing to manually enumerate them in multiple places in the codebase.
    /// The order of types is maintained consistently for iteration purposes.
    const VALUES: [BasicAxisTypes; 9] = [
        BasicAxisTypes::Delta,
        BasicAxisTypes::Gamma,
        BasicAxisTypes::Theta,
        BasicAxisTypes::Vega,
        BasicAxisTypes::Volatility,
        BasicAxisTypes::Price,
        BasicAxisTypes::UnderlyingPrice,
        BasicAxisTypes::Strike,
        BasicAxisTypes::Expiration,
    ];

    /// Creates an iterator over all variants of BasicAxisTypes.
    ///
    /// This method provides a convenient way to iterate through all available
    /// axis types sequentially, which is useful for operations that need to
    /// process each type, such as generating comprehensive reports or
    /// applying calculations across all dimensions of options analysis.
    ///
    /// # Returns
    ///
    /// A `BasicAxisTypesIter` that yields each variant of `BasicAxisTypes` in order.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tracing::info;
    /// use optionstratlib::model::BasicAxisTypes;
    /// for axis_type in BasicAxisTypes::iter() {
    ///     info!("Processing axis type: {:?}", axis_type);
    ///     // Perform operations with each axis type
    /// }
    /// ```
    pub fn iter() -> BasicAxisTypesIter {
        BasicAxisTypesIter { index: 0 }
    }
}

impl Iterator for BasicAxisTypesIter {
    type Item = BasicAxisTypes;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < BasicAxisTypes::VALUES.len() {
            let value = BasicAxisTypes::VALUES[self.index];
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests_basic_axis_types {
    use super::*;
    use serde_json;

    #[test]
    fn test_debug_implementation() {
        assert_eq!(format!("{:?}", BasicAxisTypes::Delta), "Delta");
        assert_eq!(format!("{:?}", BasicAxisTypes::Gamma), "Gamma");
        assert_eq!(format!("{:?}", BasicAxisTypes::Theta), "Theta");
        assert_eq!(format!("{:?}", BasicAxisTypes::Vega), "Vega");
        assert_eq!(format!("{:?}", BasicAxisTypes::Volatility), "Volatility");
        assert_eq!(format!("{:?}", BasicAxisTypes::Price), "Price");
        assert_eq!(format!("{:?}", BasicAxisTypes::Strike), "Strike");
        assert_eq!(format!("{:?}", BasicAxisTypes::Expiration), "Expiration");
    }

    #[test]
    fn test_clone_implementation() {
        let axis_type = BasicAxisTypes::Delta;
        let cloned = axis_type;
        assert_eq!(axis_type, cloned);
    }

    #[test]
    fn test_copy_implementation() {
        let axis_type = BasicAxisTypes::Gamma;
        let copied = axis_type;
        // Both should be valid due to Copy trait
        assert_eq!(axis_type, copied);
    }

    #[test]
    fn test_equality_comparison() {
        assert_eq!(BasicAxisTypes::Delta, BasicAxisTypes::Delta);
        assert_ne!(BasicAxisTypes::Delta, BasicAxisTypes::Gamma);
        assert_ne!(BasicAxisTypes::Theta, BasicAxisTypes::Vega);
    }

    #[test]
    fn test_serialization() {
        // Test serialization for each variant
        let variants = vec![
            BasicAxisTypes::Delta,
            BasicAxisTypes::Gamma,
            BasicAxisTypes::Theta,
            BasicAxisTypes::Vega,
            BasicAxisTypes::Volatility,
            BasicAxisTypes::Price,
            BasicAxisTypes::Strike,
        ];

        for variant in variants {
            let serialized = serde_json::to_string(&variant).expect("Failed to serialize");
            let deserialized: BasicAxisTypes =
                serde_json::from_str(&serialized).expect("Failed to deserialize");
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_specific_serialization_format() {
        let serialized = serde_json::to_string(&BasicAxisTypes::Delta).unwrap();
        assert_eq!(serialized, "\"Delta\"");

        let serialized = serde_json::to_string(&BasicAxisTypes::Volatility).unwrap();
        assert_eq!(serialized, "\"Volatility\"");
    }

    #[test]
    fn test_deserialization() {
        let json_delta = "\"Delta\"";
        let deserialized: BasicAxisTypes = serde_json::from_str(json_delta).unwrap();
        assert_eq!(deserialized, BasicAxisTypes::Delta);

        let json_volatility = "\"Volatility\"";
        let deserialized: BasicAxisTypes = serde_json::from_str(json_volatility).unwrap();
        assert_eq!(deserialized, BasicAxisTypes::Volatility);
    }

    #[test]
    fn test_invalid_deserialization() {
        let result = serde_json::from_str::<BasicAxisTypes>("\"InvalidAxis\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_exhaustive_variant_comparison() {
        assert!(matches!(BasicAxisTypes::Delta, BasicAxisTypes::Delta));
        assert!(matches!(BasicAxisTypes::Gamma, BasicAxisTypes::Gamma));
        assert!(matches!(BasicAxisTypes::Theta, BasicAxisTypes::Theta));
        assert!(matches!(BasicAxisTypes::Vega, BasicAxisTypes::Vega));
        assert!(matches!(
            BasicAxisTypes::Volatility,
            BasicAxisTypes::Volatility
        ));
        assert!(matches!(BasicAxisTypes::Price, BasicAxisTypes::Price));
        assert!(matches!(BasicAxisTypes::Strike, BasicAxisTypes::Strike));
    }

    #[test]
    fn test_memory_layout() {
        use std::mem::size_of;
        // Ensure the enum is efficiently sized (should be 1 byte as it's a simple enum)
        assert_eq!(size_of::<BasicAxisTypes>(), 1);
    }
}

#[cfg(test)]
mod tests_basic_axis_types_extended {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_iterator_creation() {
        // Test that the iterator is created correctly
        let iterator = BasicAxisTypes::iter();
        assert_eq!(iterator.index, 0);
    }

    #[test]
    fn test_iterator_next() {
        // Test that the iterator returns the correct first element
        let mut iterator = BasicAxisTypes::iter();
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Delta));
        assert_eq!(iterator.index, 1);
    }

    #[test]
    fn test_iterator_complete_traversal() {
        // Test that the iterator traverses all variants in the expected order
        let mut iterator = BasicAxisTypes::iter();

        assert_eq!(iterator.next(), Some(BasicAxisTypes::Delta));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Gamma));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Theta));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Vega));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Volatility));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Price));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::UnderlyingPrice));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Strike));
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Expiration));

        // After all elements are exhausted, should return None
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn test_iterator_exhaustion() {
        // Test behavior after iterator is exhausted
        let mut iterator = BasicAxisTypes::iter();

        // Consume all elements
        for _ in 0..BasicAxisTypes::VALUES.len() {
            iterator.next();
        }

        // Should return None on subsequent calls
        assert_eq!(iterator.next(), None);
        assert_eq!(iterator.next(), None);
        assert_eq!(iterator.index, BasicAxisTypes::VALUES.len());
    }

    #[test]
    fn test_values_array_completeness() {
        // Test that VALUES array contains all enum variants exactly once
        let values_set: HashSet<BasicAxisTypes> = BasicAxisTypes::VALUES.iter().cloned().collect();

        // Check all variants are present
        assert!(values_set.contains(&BasicAxisTypes::Delta));
        assert!(values_set.contains(&BasicAxisTypes::Gamma));
        assert!(values_set.contains(&BasicAxisTypes::Theta));
        assert!(values_set.contains(&BasicAxisTypes::Vega));
        assert!(values_set.contains(&BasicAxisTypes::Volatility));
        assert!(values_set.contains(&BasicAxisTypes::Price));
        assert!(values_set.contains(&BasicAxisTypes::UnderlyingPrice));
        assert!(values_set.contains(&BasicAxisTypes::Strike));
        assert!(values_set.contains(&BasicAxisTypes::Expiration));

        // Check for exact count (no duplicates)
        assert_eq!(values_set.len(), 9);
    }

    #[test]
    fn test_iterator_collect() {
        // Test collecting all values from the iterator
        let collected: Vec<BasicAxisTypes> = BasicAxisTypes::iter().collect();

        assert_eq!(collected.len(), 9);
        assert_eq!(collected, BasicAxisTypes::VALUES);
    }

    #[test]
    fn test_iterator_count() {
        // Test counting the elements in the iterator
        let count = BasicAxisTypes::iter().count();
        assert_eq!(count, 9);
    }

    #[test]
    fn test_iterator_for_loop() {
        // Test using the iterator in a for loop
        let mut encountered = vec![];

        for axis_type in BasicAxisTypes::iter() {
            encountered.push(axis_type);
        }

        assert_eq!(encountered.len(), 9);
        assert_eq!(encountered, BasicAxisTypes::VALUES);
    }

    #[test]
    fn test_iterator_partial_consumption() {
        // Test partially consuming the iterator
        let mut iterator = BasicAxisTypes::iter();

        // Consume first 3 elements
        for _ in 0..3 {
            iterator.next();
        }

        // Check next element
        assert_eq!(iterator.next(), Some(BasicAxisTypes::Vega));
        assert_eq!(iterator.index, 4);
    }

    #[test]
    fn test_iterator_multiple_use() {
        // Test creating and using multiple iterators independently
        let mut iterator1 = BasicAxisTypes::iter();
        let mut iterator2 = BasicAxisTypes::iter();

        assert_eq!(iterator1.next(), Some(BasicAxisTypes::Delta));
        assert_eq!(iterator1.next(), Some(BasicAxisTypes::Gamma));

        // Second iterator should start from beginning
        assert_eq!(iterator2.next(), Some(BasicAxisTypes::Delta));

        // Continue with first iterator
        assert_eq!(iterator1.next(), Some(BasicAxisTypes::Theta));

        // Iterators should maintain independent state
        assert_eq!(iterator1.index, 3);
        assert_eq!(iterator2.index, 1);
    }

    #[test]
    fn test_filter_map_operations() {
        // Test using filter and map on the iterator
        let greeks: Vec<BasicAxisTypes> = BasicAxisTypes::iter()
            .filter(|axis| {
                matches!(
                    axis,
                    BasicAxisTypes::Delta
                        | BasicAxisTypes::Gamma
                        | BasicAxisTypes::Theta
                        | BasicAxisTypes::Vega
                )
            })
            .collect();

        assert_eq!(greeks.len(), 4);
        assert_eq!(greeks[0], BasicAxisTypes::Delta);
        assert_eq!(greeks[3], BasicAxisTypes::Vega);

        // Test mapping operation
        let names: Vec<&str> = BasicAxisTypes::iter()
            .map(|axis| match axis {
                BasicAxisTypes::Delta => "delta",
                BasicAxisTypes::Gamma => "gamma",
                BasicAxisTypes::Theta => "theta",
                BasicAxisTypes::Vega => "vega",
                BasicAxisTypes::Volatility => "volatility",
                BasicAxisTypes::Price => "price",
                BasicAxisTypes::UnderlyingPrice => "underlying",
                BasicAxisTypes::Strike => "strike",
                BasicAxisTypes::Expiration => "expiration",
            })
            .collect();

        assert_eq!(names.len(), 9);
        assert_eq!(names[0], "delta");
        assert_eq!(names[4], "volatility");
    }

    #[test]
    fn test_reverse_traversal() {
        // Since BasicAxisTypesIter doesn't implement DoubleEndedIterator,
        // we can manually collect and reverse
        let all_axes: Vec<BasicAxisTypes> = BasicAxisTypes::iter().collect();
        let reverse_order: Vec<BasicAxisTypes> = all_axes.into_iter().rev().collect();

        assert_eq!(reverse_order.len(), 9);
        assert_eq!(reverse_order[0], BasicAxisTypes::Expiration);
        assert_eq!(reverse_order[8], BasicAxisTypes::Delta);
    }

    #[test]
    fn test_find_in_iterator() {
        // Test finding elements in the iterator
        let price = BasicAxisTypes::iter().find(|&axis| axis == BasicAxisTypes::Price);
        assert_eq!(price, Some(BasicAxisTypes::Price));

        let nonexistent = BasicAxisTypes::iter().find(|_| false);
        assert_eq!(nonexistent, None);
    }

    #[test]
    fn test_functional_operations() {
        // Test various functional operations on the iterator

        // Test any() - checks if any element satisfies a condition
        let has_price = BasicAxisTypes::iter().any(|axis| axis == BasicAxisTypes::Price);
        assert!(has_price);

        // Test all() - checks if all elements satisfy a condition
        let all_valid = BasicAxisTypes::iter().all(|_| true);
        assert!(all_valid);

        // Test position() - finds the index of the first matching element
        let vega_position = BasicAxisTypes::iter().position(|axis| axis == BasicAxisTypes::Vega);
        assert_eq!(vega_position, Some(3));
    }
}

// Add tests specifically for the VALUES array constant
#[cfg(test)]
mod tests_values_array {
    use super::*;

    #[test]
    fn test_values_array_length() {
        assert_eq!(BasicAxisTypes::VALUES.len(), 9);
    }

    #[test]
    fn test_values_array_content() {
        assert_eq!(BasicAxisTypes::VALUES[0], BasicAxisTypes::Delta);
        assert_eq!(BasicAxisTypes::VALUES[1], BasicAxisTypes::Gamma);
        assert_eq!(BasicAxisTypes::VALUES[2], BasicAxisTypes::Theta);
        assert_eq!(BasicAxisTypes::VALUES[3], BasicAxisTypes::Vega);
        assert_eq!(BasicAxisTypes::VALUES[4], BasicAxisTypes::Volatility);
        assert_eq!(BasicAxisTypes::VALUES[5], BasicAxisTypes::Price);
        assert_eq!(BasicAxisTypes::VALUES[6], BasicAxisTypes::UnderlyingPrice);
        assert_eq!(BasicAxisTypes::VALUES[7], BasicAxisTypes::Strike);
        assert_eq!(BasicAxisTypes::VALUES[8], BasicAxisTypes::Expiration);
    }

    #[test]
    fn test_values_array_order_consistency() {
        // Ensure the VALUES array order is consistent with the enum definition order
        let iter_values: Vec<BasicAxisTypes> = BasicAxisTypes::iter().collect();

        for (i, &value) in BasicAxisTypes::VALUES.iter().enumerate() {
            assert_eq!(value, iter_values[i]);
        }
    }

    #[test]
    fn test_direct_array_access() {
        // Test directly accessing array elements
        for i in 0..BasicAxisTypes::VALUES.len() {
            // This should not panic
            let _axis_type = BasicAxisTypes::VALUES[i];
        }
    }
}
