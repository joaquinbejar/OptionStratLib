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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
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
    /// use optionstratlib::model::BasicAxisTypes;
    /// for axis_type in BasicAxisTypes::iter() {
    ///     println!("Processing axis type: {:?}", axis_type);
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
