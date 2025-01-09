use crate::curves::Point2D;
use crate::error::CurvesError;
use rust_decimal::Decimal;

/// A trait that provides functionality for performing cubic interpolation.
///
/// # Overview
/// The `CubicInterpolation` trait is used to define the behavior for
/// interpolating values along a curve using cubic interpolation.
/// Implementors of this trait must define the `cubic_interpolate` method
/// to calculate a `Point2D` based on a given input `x` value.
///
/// Cubic interpolation is a method of constructing new points within the range of a
/// discrete set of known points by using cubic polynomials. This method is widely
/// used in mathematical curve fitting and graphical applications where smooth
/// transitions between points are needed, such as physics simulations, animations,
/// and signal processing.
///
/// # Required Methods
///
/// ## `cubic_interpolate`
///
/// This method performs the core cubic interpolation operation.
///
/// - **Input:**  
///   Requires an `x` coordinate represented as a `Decimal` value for which the
///   corresponding point on the curve is interpolated.
///
/// - **Returns:**  
///   - On success, returns a `Point2D` representing the interpolated point on the curve.
///   - On failure, returns a `CurvesError` describing the reason for the error (e.g.,
///     attempting to interpolate with missing data or invalid input).
///
/// # Use Case Examples (General)
///
/// This trait is commonly implemented for interpolation modules or types where cubic
/// interpolation of data is needed:
/// - Interpolating a smooth curve through a set of 2D points
/// - Performing animations with precise movement based on cubic functions
/// - Simulating natural physical phenomena like easing or oscillations
///
/// # Notes on Precision
/// The use of `Decimal` as the input type ensures high precision when dealing
/// with floating-point calculations. This makes it suitable for scientific or
/// financial applications where numerical accuracy is of utmost importance.
///
/// # Errors
/// Implementations of this trait should handle and return appropriate errors
/// using the `CurvesError` type when the operation cannot be completed.
///
/// ## Typical Errors:
/// - `InterpolationError`: Issues during the interpolation process, such as
///   insufficient data points or invalid input values.
/// - `Point2DError`: Issues related to incorrect point data.
///
/// # Example Implementation Outline
/// When implementing this trait, you may structure the cubic interpolation
/// algorithm with the following steps:
/// 1. Access and validate the data points necessary for interpolation.
/// 2. Calculate the coefficients of the cubic polynomial.
/// 3. Evaluate the polynomial at the given `x` value to produce the interpolated result.
///
/// It is critical for implementations to conform to the trait's contract, ensuring
/// reliable operation and proper error handling in all cases.
///
/// # See Also
/// - `Point2D`: Represents a 2D point in space, often used as the result of interpolation.
/// - `CurvesError`: Captures errors that might occur during interpolation processes.
pub trait CubicInterpolation {
    /// Interpolates a point on the curve using cubic interpolation for a given `x` value.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate (of type `Decimal`) for which the corresponding
    ///   interpolated point should be calculated.
    ///
    /// # Returns
    /// - `Ok(Point2D)`: The interpolated point on the curve.
    /// - `Err(CurvesError)`: An error representing why the interpolation failed.
    fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError>;
}
