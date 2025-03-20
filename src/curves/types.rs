/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::error::curves::CurveError;
use crate::geometrics::HasX;
use crate::model::positive::is_positive;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

/// Represents a point in two-dimensional space with `x` and `y` coordinates.
///
/// # Overview
/// The `Point2D` struct is used to define a point in a 2D Cartesian coordinate system.
/// Both coordinates are stored as `Decimal` values to provide high precision,
/// making it suitable for applications requiring accurate numerical calculations.
///
/// # Usage
/// This structure serves as a fundamental data type in various geometric operations:
/// - Defining positions in curve plotting and interpolation
/// - Representing intersections between curves
/// - Serving as input/output for mathematical transformations
/// - Supporting coordinate-based algorithms in the library
///
/// # Examples
///
/// ```rust
/// use rust_decimal_macros::dec;
/// use optionstratlib::curves::Point2D;
///
/// // Create a point at coordinates (3.5, -2.25)
/// let point = Point2D {
///     x: dec!(3.5),
///     y: dec!(-2.25)
/// };
/// ```
///
/// # Derivable Traits
/// - `Debug`: Enables formatted debugging output
/// - `Clone` and `Copy`: Allow efficient duplication of point values
/// - `Serialize` and `Deserialize`: Support for serialization frameworks
///
/// This struct is primarily used in conjunction with the `Curve` and `Curvable` types
/// to represent mathematical curves and perform geometric operations.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point2D {
    /// The x-coordinate in the Cartesian plane, represented as a high-precision `Decimal`
    /// value to ensure accuracy in mathematical operations.
    pub x: Decimal,

    /// The y-coordinate in the Cartesian plane, represented as a high-precision `Decimal`
    /// value to ensure accuracy in mathematical operations.
    pub y: Decimal,
}

impl Display for Point2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

impl PartialEq for Point2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
    }
}

impl Eq for Point2D {}

impl Hash for Point2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.mantissa().hash(state);
        self.x.scale().hash(state);
    }
}

impl PartialOrd for Point2D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Point2D {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.x.cmp(&other.x) {
            Ordering::Equal => self.y.cmp(&other.y),
            x_ordering => x_ordering,
        }
    }
}

impl Point2D {
    /// Creates a new instance of `Point2D` using the specified `x` and `y` coordinates.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate of the point, which implements `Into<Decimal>`.
    /// - `y`: The y-coordinate of the point, which implements `Into<Decimal>`.
    ///
    /// # Returns
    /// A `Point2D` instance with the provided `x` and `y` coordinates, converted into `Decimal`.
    ///
    /// # Usage
    /// This function is used when creating a `Point2D` object from any type that can be converted
    /// into `Decimal`, allowing flexibility in input types (e.g., `f64`, `i32`, etc.).
    pub fn new<T: Into<Decimal>, U: Into<Decimal>>(x: T, y: U) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
        }
    }

    /// Converts the `Point2D` instance into a tuple `(T, U)`.
    ///
    /// # Parameters
    /// - `T`: The type for the x-coordinate, which must implement `From<Decimal>` and have a 'static lifetime.
    /// - `U`: The type for the y-coordinate, which must implement `From<Decimal>` and have a 'static lifetime.
    ///
    /// # Returns
    /// - `Ok`: A tuple `(T, U)` containing the converted `x` and `y` values.
    /// - `Err`: A `CurvesError` if conversion constraints are violated:
    ///   - `x` must be positive if `T` is the `Positive` type.
    ///   - `y` must be positive if `U` is the `Positive` type.
    ///
    /// # Errors
    /// This function returns an error if the positivity constraints are violated or if
    /// conversions fail due to invalid type requirements.
    pub fn to_tuple<T: From<Decimal> + 'static, U: From<Decimal> + 'static>(
        &self,
    ) -> Result<(T, U), CurveError> {
        if is_positive::<T>() && self.x <= Decimal::ZERO {
            return Err(CurveError::Point2DError {
                reason: "x must be positive for type T",
            });
        }

        if is_positive::<U>() && self.y <= Decimal::ZERO {
            return Err(CurveError::Point2DError {
                reason: "y must be positive for type U",
            });
        }

        Ok((T::from(self.x), U::from(self.y)))
    }

    /// Creates a new `Point2D` instance from a tuple containing `x` and `y` values.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate, which implements `Into<Decimal>`.
    /// - `y`: The y-coordinate, which implements `Into<Decimal>`.
    ///
    /// # Returns
    /// - `Ok`: A new `Point2D` instance with the given `x` and `y` coordinates.
    /// - `Err`: A `CurvesError` if coordinate creation fails.
    ///
    /// # Usage
    /// This function allows constructing a `Point2D` directly from a tuple representation.
    pub fn from_tuple<T: Into<Decimal>, U: Into<Decimal>>(x: T, y: U) -> Result<Self, CurveError> {
        Ok(Self::new(x, y))
    }

    /// Converts the `Point2D` instance into a tuple of `(f64, f64)`.
    ///
    /// # Returns
    /// - `Ok`: A tuple `(f64, f64)` containing the `x` and `y` values.
    /// - `Err`: A `CurvesError` if either `x` or `y` cannot be converted from
    ///   `Decimal` to `f64` (e.g., out-of-range value).
    ///
    /// # Errors
    /// Returns a `CurvesError::Point2DError` with a reason explaining the failure.
    pub fn to_f64_tuple(&self) -> Result<(f64, f64), CurveError> {
        let x = self.x.to_f64();
        let y = self.y.to_f64();

        match (x, y) {
            (Some(x), Some(y)) => Ok((x, y)),
            _ => Err(CurveError::Point2DError {
                reason: "Error converting Decimal to f64",
            }),
        }
    }

    /// Creates a new `Point2D` instance from a tuple of `(f64, f64)` values.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate of the point as a `f64`.
    /// - `y`: The y-coordinate of the point as a `f64`.
    ///
    /// # Returns
    /// - `Ok`: A new `Point2D` instance if both `x` and `y` values can be successfully
    ///   converted from `f64` to `Decimal`.
    /// - `Err`: A `CurvesError` if the conversion fails (e.g., invalid precision).
    ///
    /// # Errors
    /// Returns a `CurvesError::Point2DError` with a reason if either `x` or `y` could not be
    /// converted from `f64`.
    pub fn from_f64_tuple(x: f64, y: f64) -> Result<Self, CurveError> {
        let x = Decimal::from_f64(x);
        let y = Decimal::from_f64(y);
        match (x, y) {
            (Some(x), Some(y)) => Ok(Self::new(x, y)),
            _ => Err(CurveError::Point2DError {
                reason: "Error converting f64 to Decimal",
            }),
        }
    }
}

impl From<&Point2D> for Point2D {
    fn from(point: &Point2D) -> Self {
        *point
    }
}

impl HasX for Point2D {
    fn get_x(&self) -> Decimal {
        self.x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_is_positive_x_must_be_positive() {
        let point = Point2D {
            x: Decimal::ZERO,
            y: dec!(1.0),
        };

        let result = if is_positive::<Decimal>() && point.x <= Decimal::ZERO {
            Err(CurveError::Point2DError {
                reason: "x must be positive for type T",
            })
        } else {
            Ok(())
        };

        assert!(result.is_ok());
    }

    #[test]
    fn test_is_positive_y_must_be_positive() {
        let point = Point2D {
            x: dec!(1.0),
            y: Decimal::ZERO,
        };

        let result = if is_positive::<Decimal>() && point.y <= Decimal::ZERO {
            Err(CurveError::Point2DError {
                reason: "y must be positive for type U",
            })
        } else {
            Ok(())
        };

        assert!(result.is_ok());
    }

    #[test]
    fn test_to_f64_tuple_success() {
        let point = Point2D {
            x: dec!(1.0),
            y: dec!(2.0),
        };
        let result = point.to_f64_tuple();
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_f64_tuple_success() {
        let result = Point2D::from_f64_tuple(1.0, 2.0);
        assert!(result.is_ok());
        let point = result.unwrap();
        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
    }

    #[test]
    fn test_from_f64_tuple_error() {
        let result = Point2D::from_f64_tuple(f64::INFINITY, 2.0);
        assert!(result.is_err());
        match result {
            Err(CurveError::Point2DError { reason }) => {
                assert_eq!(reason, "Error converting f64 to Decimal");
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_equal() {
        let p1 = Point2D::from_f64_tuple(1.0, 2.0).unwrap();
        let p2 = Point2D::from_f64_tuple(1.0, 2.0).unwrap();
        let p3 = Point2D::from_f64_tuple(1.0, 3.0).unwrap();
        let p4 = Point2D::from_f64_tuple(1.0, 4.0).unwrap();
        let p5 = Point2D::from_f64_tuple(2.0, 2.0).unwrap();
        assert_eq!(p1, p2);
        assert_eq!(p1, p3);
        assert_eq!(p1, p4);
        assert_ne!(p1, p5);
    }
}

#[cfg(test)]
mod tests_point2d_serde {
    use super::*;
    use rust_decimal_macros::dec;
    use serde_json::Value;

    #[test]
    fn test_basic_serialization() {
        let point = Point2D {
            x: dec!(1.5),
            y: dec!(2.5),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point.x, deserialized.x);
        assert_eq!(point.y, deserialized.y);
    }

    #[test]
    fn test_zero_values() {
        let point = Point2D {
            x: dec!(0.0),
            y: dec!(0.0),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_negative_values() {
        let point = Point2D {
            x: dec!(-1.5),
            y: dec!(-2.5),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_high_precision_values() {
        let point = Point2D {
            x: dec!(1.12345678901234567890),
            y: dec!(2.12345678901234567890),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point.x, deserialized.x);
        assert_eq!(point.y, deserialized.y);
    }

    #[test]
    fn test_json_structure() {
        let point = Point2D {
            x: dec!(1.5),
            y: dec!(2.5),
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let json_value: Value = serde_json::from_str(&serialized).unwrap();

        // Verify JSON structure
        assert!(json_value.is_object());
        assert_eq!(json_value.as_object().unwrap().len(), 2);
        assert!(json_value.get("x").is_some());
        assert!(json_value.get("y").is_some());
    }

    #[test]
    fn test_pretty_print() {
        let point = Point2D {
            x: dec!(1.5),
            y: dec!(2.5),
        };

        let serialized = serde_json::to_string_pretty(&point).unwrap();

        // Verify pretty print format
        assert!(serialized.contains('\n'));
        assert!(serialized.contains("  "));

        // Verify we can still deserialize pretty-printed JSON
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();
        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_deserialize_from_integers() {
        let json_str = r#"{"x": 1, "y": 2}"#;
        let point: Point2D = serde_json::from_str(json_str).unwrap();

        assert_eq!(point.x, dec!(1.0));
        assert_eq!(point.y, dec!(2.0));
    }

    #[test]
    fn test_deserialize_from_strings() {
        let json_str = r#"{"x": "1.5", "y": "2.5"}"#;
        let point: Point2D = serde_json::from_str(json_str).unwrap();

        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    fn test_invalid_json() {
        // Missing field
        let json_str = r#"{"x": 1.5}"#;
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_err());

        // Invalid number format
        let json_str = r#"{"x": "invalid", "y": 2.5}"#;
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_err());

        // Wrong data type
        let json_str = r#"{"x": true, "y": 2.5}"#;
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_values() {
        let point = Point2D {
            x: Decimal::MAX,
            y: Decimal::MAX,
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_min_values() {
        let point = Point2D {
            x: Decimal::MIN,
            y: Decimal::MIN,
        };

        let serialized = serde_json::to_string(&point).unwrap();
        let deserialized: Point2D = serde_json::from_str(&serialized).unwrap();

        assert_eq!(point, deserialized);
    }

    #[test]
    fn test_json_to_vec() {
        let points = vec![
            Point2D {
                x: dec!(1.0),
                y: dec!(2.0),
            },
            Point2D {
                x: dec!(3.0),
                y: dec!(4.0),
            },
        ];

        let serialized = serde_json::to_string(&points).unwrap();
        let deserialized: Vec<Point2D> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(points, deserialized);
    }

    #[test]
    fn test_duplicate_fields() {
        let json_str = r#"{"x": 1.5, "y": 2.5, "x": 3.5}"#;
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_extra_fields() {
        let json_str = r#"{"x": 1.5, "y": 2.5, "z": 3.5}"#;
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_fields() {
        let json_str = r#"{"x": 1.5, "r": 2.5, "z": 3.5}"#;
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_array() {
        let json_str = "[1.5, 2.5]";
        let result = serde_json::from_str::<Point2D>(json_str);
        assert!(result.is_ok());
    }
}
