use crate::error::InterpolationError;

/// A trait for bilinear interpolation on 2D data grids.
///
/// # Purpose
/// The `BiLinearInterpolation` trait is designed to perform bilinear interpolation
/// to estimate intermediate values within a grid of 2D points. This method is often
/// used in numerical computation tasks, such as image processing, terrain modeling,
/// and scientific data visualization.
///
/// # Type Parameters
/// - `Point`: The output type, typically used to represent the interpolated 2D point.
/// - `Input`: The input type for the interpolation parameter, typically a scalar value.
///
/// # Associated Type
/// - `Error`: Defines the type returned in case of a failure during interpolation.
///
/// # Method
/// - [`bilinear_interpolate`](#method.bilinear_interpolate):
///   Computes the interpolated value at the given input, returning either the result
///   or an error if the operation cannot proceed.
///
/// # Errors
/// Any errors encountered during interpolation are encapsulated in the type `Error`.
/// This trait is expected to return meaningful errors in cases like:
/// - Insufficient or invalid data for computation.
/// - Inputs that are out of bounds for the given dataset.
/// - Issues specific to the interpolation logic.
///
/// # Example Usage
/// Below is an example demonstrating how an implementing struct might use this trait:
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib::curves::Point2D;
/// use optionstratlib::error::InterpolationError;
/// use optionstratlib::geometrics::BiLinearInterpolation;
/// struct GridInterpolator {
///     // Implementation-specific fields like the grid or data.
/// }
///
/// impl BiLinearInterpolation<Point2D, Decimal> for GridInterpolator {
///
///     fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
///         Ok(Point2D::new(x, x)) // Placeholder implementation
///     }
/// }
/// ```
///
/// In this example:
/// - `GridInterpolator` implements the trait for bilinear interpolation.
/// - The `bilinear_interpolate` method calculates the interpolated `Point2D` for a given `x` value.
///
/// # Related Types
/// - [`Point2D`](crate::curves::types::Point2D): A struct representing a 2D point with `x` and `y` coordinates.
/// - [`CurvesError`](crate::errors::CurvesError): A recommended error type for detailed error categorization.
///
/// # See Also
/// - [`crate::geometrics::interpolation::InterpolationType`](crate::curves::interpolation::traits::InterpolationType):
///   A module defining different types of interpolation methods.
/// - [`crate::geometrics::interpolation::LinearInterpolation`](crate::curves::interpolation::traits::LinearInterpolation):
///   A simpler interpolation method for one-dimensional data.
pub trait BiLinearInterpolation<Point, Input> {
    /// Performs bilinear interpolation to compute a value for the given input.
    ///
    /// # Parameters
    /// - `x`: The input value (e.g., an `x` coordinate in 2D space) for which the interpolation is performed.
    ///
    /// # Returns
    /// - `Ok(Point)`: The interpolated point (e.g., a `Point2D`) representing the computed values.
    /// - `Err(Self::Error)`: An error indicating why the interpolation could not be performed.
    ///
    /// # Example
    /// ```rust
    /// use std::collections::BTreeSet;
    /// use rust_decimal::Decimal;
    /// use tracing::info;
    /// use optionstratlib::curves::{Curve, Point2D};
    /// use optionstratlib::geometrics::BiLinearInterpolation;
    /// let curve = Curve::new(BTreeSet::from_iter(vec![
    ///            Point2D::new(Decimal::ZERO, Decimal::ZERO),
    ///            Point2D::new(Decimal::ONE, Decimal::TWO),
    ///        ]));
    /// let result = curve.bilinear_interpolate(Decimal::from(2));
    ///
    /// match result {
    ///     Ok(point) => info!("Interpolated point: {:?}", point),
    ///     Err(e) => info!("Interpolation failed: {:?}", e),
    /// }
    /// ```
    fn bilinear_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}
