use crate::error::InterpolationError;

/// A trait for performing cubic interpolation on a set of data points.
///
/// # Overview
/// The `CubicInterpolation` trait provides a framework for implementing cubic
/// interpolation algorithms. This technique is used to calculate smooth curves
/// through a set of discrete data points, commonly via cubic polynomials. The primary
/// functionality is exposed via the `cubic_interpolate` method.
///
/// Cubic interpolation offers smooth transitions between points and is widely
/// applied in fields such as:
/// - **Graphical applications**: Creating smooth animations or generating curves.
/// - **Signal processing**: Smoothing data or reconstructing missing data points.
/// - **Physics and simulations**: Modeling smooth transformations, trajectories, or motions.
///
/// When implementing this trait, it is recommended to carefully manage edge
/// cases, such as insufficient data points, invalid input values, or floating-point
/// precision issues.
///
/// # Associated Types
/// - `Point`: Represents the type of output point (e.g., 2D or n-dimensional points).
/// - `Input`: The type of the input coordinate, typically a numeric type (e.g., `Decimal`).
/// - `Error`: The type used for error handling when interpolation fails.
///
/// # Required Method
///
/// ## `cubic_interpolate`
///
/// Performs cubic interpolation for a provided input value, calculating
/// the corresponding interpolated point.
///
/// ### Parameters
/// - `x`: The input value (of type `Input`) for which the interpolated
///   point on the curve should be calculated.
///
/// ### Returns
/// - `Ok(Point)`: The interpolated point successfully calculated for the given input.
/// - `Err(Error)`: An error that provides details about why the interpolation operation
///   failed.
///
/// # Notes on Precision
/// The precision of the interpolation is directly influenced by the type of the
/// input (`Input`) and the internal calculations used. If high numerical precision
/// is required, using a type like `Decimal` (from the `rust_decimal` crate) is
/// highly recommended to avoid floating-point inaccuracies.
///
/// # Error Handling
/// Implementations of this trait should return meaningful errors of the
/// associated `Error` type when interpolation cannot be completed. Examples include:
/// - **Insufficient data points**: If computation requires more data than is available.
/// - **Out-of-bounds input values**: When the input value `x` falls outside the
///   defined interpolation range.
/// - **Invalid data**: Issues encountered in the provided data or curve structure.
///
/// # Example Usage
/// Below is a general outline for implementing this trait:
/// ```text
/// 1. Validate the data points necessary for interpolation.
/// 2. Compute the required coefficients for a cubic polynomial.
/// 3. Use the coefficients to evaluate the polynomial at the provided input `x`
///    to obtain the interpolated point.
/// 4. Handle edge cases and return appropriate errors if the operation fails.
/// ```
///
/// # See Also
/// - `Point`: Generic representation of a point in 2D or multi-dimensional space.
/// - `CurvesError`: A possible error type for representing issues during interpolation.
/// - `rust_decimal`: A crate for performing high-precision numerical calculations.
///
/// # Usage Example
/// An implementation of cubic interpolation could apply this trait to a data set
/// as follows:
/// ```rust
/// use rust_decimal::Decimal;///
///
/// use optionstratlib::curves::Point2D;
/// use optionstratlib::error::InterpolationError;
///
///
/// use optionstratlib::geometrics::CubicInterpolation;
///
/// struct MyCurve {
///     data_points: Vec<Point2D>,
/// }
///
/// impl CubicInterpolation<Point2D, Decimal> for MyCurve {
///
///     fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
///         // Validate the input and calculate the interpolated point.
///         // Example logic here.
///         Ok(Point2D::new(x, x)) // Placeholder implementation.
///     }
/// }
/// ```
pub trait CubicInterpolation<Point, Input> {
    /// Interpolates a new point on the curve for a given `x` input value
    /// using cubic interpolation.
    ///
    /// # Parameters
    /// - `x`: The input value along the curve for which an interpolated
    ///   point is calculated.
    ///
    /// # Returns
    /// - `Ok(Point)`: Represents the interpolated point on the curve.
    /// - `Err(Self::Error)`: Describes why the interpolation failed (e.g., invalid input).
    fn cubic_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
