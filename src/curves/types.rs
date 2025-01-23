/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::error::curves::CurvesError;
use crate::geometrics::HasX;
use crate::model::positive::is_positive;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::cmp::Ordering;

/// Represents a point in two-dimensional space with `x` and `y` coordinates.
///
/// # Overview
/// The `Point2D` struct is used to define a point in a 2D Cartesian coordinate system.
/// Both coordinates (`x` and `y`) are stored as `Decimal` values to provide high precision,
/// making it suitable for applications requiring accurate numerical calculations, such
/// as mathematical curve analysis, interpolation, and geometry.
///
/// # Usage
/// This structure is commonly used as a fundamental data type to represent points
/// in various operations, such as:
/// - Curve interpolation
/// - Defining specific positions or intersections in the Cartesian plane
/// - Transformations (translation, scaling, etc.)
///
/// # Derivable Traits
/// - `Debug`: Enables formatted output of the structure for debugging purposes.
/// - `Clone`: Allows the point to be cloned, producing a duplicate in memory.
/// - `Copy`: Simplifies handling by enabling value duplication without explicit cloning.
/// - `PartialEq`: Enables equality comparison between two `Point2D` instances.
///
/// # Examples of Use
/// The `Point2D` struct is generally used in combination with mathematical and
/// graphical operations within the library, as outlined in the relevant modules,
/// such as `curve_traits` or `operations`. Examples of such use cases include
/// finding intersections between curves and performing translations or scaling.
///
/// # Fields
/// - **x**: The x-coordinate of the point, represented as a `Decimal`.
/// - **y**: The y-coordinate of the point, represented as a `Decimal`.
///
/// This structure enables high precision for x and y values, making it particularly
/// well-suited for scientific applications and precise geometry.
#[derive(Debug, Clone, Copy, Hash)]
pub struct Point2D {
    pub x: Decimal,
    pub y: Decimal,
}

impl PartialEq for Point2D {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
    }
}

impl Eq for Point2D {}

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
    ) -> Result<(T, U), CurvesError> {
        if is_positive::<T>() && self.x <= Decimal::ZERO {
            return Err(CurvesError::Point2DError {
                reason: "x must be positive for type T",
            });
        }

        if is_positive::<U>() && self.y <= Decimal::ZERO {
            return Err(CurvesError::Point2DError {
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
    pub fn from_tuple<T: Into<Decimal>, U: Into<Decimal>>(x: T, y: U) -> Result<Self, CurvesError> {
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
    pub fn to_f64_tuple(&self) -> Result<(f64, f64), CurvesError> {
        let x = self.x.to_f64();
        let y = self.y.to_f64();

        match (x, y) {
            (Some(x), Some(y)) => Ok((x, y)),
            _ => Err(CurvesError::Point2DError {
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
    pub fn from_f64_tuple(x: f64, y: f64) -> Result<Self, CurvesError> {
        let x = Decimal::from_f64(x);
        let y = Decimal::from_f64(y);
        match (x, y) {
            (Some(x), Some(y)) => Ok(Self::new(x, y)),
            _ => Err(CurvesError::Point2DError {
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

/// Enumeration representing various types of curves that can be analyzed, constructed,
/// or manipulated in mathematical and financial applications.
///
/// # Overview
/// The `CurveType` enum defines a comprehensive set of curve categories, providing clear
/// differentiation between different types of curves used in computations. These curves
/// are often employed in mathematical modeling, financial analysis, data interpolation,
/// and visualization.
///
/// This enum supports extensibility and can be used in conjunction with traits like
/// `CurveOperations` for defining curve-specific methodologies, such as creating,
/// transforming, or analyzing individual curves.
///
/// # Variants
/// - **Volatility**: Represents a curve modeling volatility in financial or statistical contexts.
/// - **Delta**: Used to describe a curve of option sensitivity with respect to the underlying price.
/// - **Gamma**: Refers to a curve showing the rate of change in Delta with respect to the underlying price.
/// - **Theta**: Represents a curve of options time decay, defining how an option's price changes over time.
/// - **Rho**: Represents the sensitivity of an option's price to changes in interest rate.
/// - **RhoD**: A more refined variant of the Rho calculation.
/// - **Vega**: Defines the sensitivity of an option's price with respect to volatility.
/// - **Binomial**: Refers to curves derived from binomial option pricing models.
/// - **BlackScholes**: Curves based on the Black-Scholes model used in option pricing.
/// - **Telegraph**: Represents special-purpose curves, e.g., telegraph-like processes in modeling.
/// - **Payoff**: Defines a curve showing the payoff structure of an option or derivative.
/// - **IntrinsicValue**: Represents intrinsic value curves describing the actual value of an option.
/// - **TimeValue**: Refers to the curve denoting the time value of an option beyond its intrinsic value.
///
/// # Usage
/// This enumeration is typically employed in financial modeling or mathematical computations
/// requiring different categories of curves. It is used extensively within various `CurveOperations`
/// to categorize and generate specific types of curves and mathematical constructs.
///
/// # Examples
/// This enum can be passed as an argument to methods like:
/// - `generate_curve`
/// - `analyze_curve`
///
/// # Derivable Traits
/// - `Debug`: Enables formatted output of the enum variant for debugging purposes.
/// - `Clone`: Allows duplication of a `CurveType` instance.
/// - `Copy`: Simplifies the handling of enum values by allowing implicit copying.
///
/// # Integrations
/// The `CurveType` enum is used heavily across modules within the `curves` package such
/// as `analysis`, `construction`, and `visualization`. It provides type safety and ensures
/// domain-specific clarity for curves utilized in:
/// - Statistical analysis
/// - Model generation
/// - Graphical data rendering
#[derive(Debug, Clone, Copy)]
pub enum CurveType {
    Volatility,
    Delta,
    Gamma,
    Theta,
    Rho,
    RhoD,
    Vega,
    Binomial,
    BlackScholes,
    Telegraph,
    Payoff,
    IntrinsicValue,
    TimeValue,
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
            Err(CurvesError::Point2DError {
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
            Err(CurvesError::Point2DError {
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
            Err(CurvesError::Point2DError { reason }) => {
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
