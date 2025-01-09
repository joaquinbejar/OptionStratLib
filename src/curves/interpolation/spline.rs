
use rust_decimal::Decimal;
use crate::curves::Point2D;
use crate::error::CurvesError;

/// A trait that defines the behavior for performing spline interpolation on a given dataset.
///
/// # Overview
/// The `SplineInterpolation` trait provides a method for interpolating data points
/// using a spline, a piecewise polynomial function that is smooth and continuous 
/// across its entire range. This is commonly used in numerical analysis, computer graphics,
/// and scientific applications to generate smooth curves that pass through or near
/// a set of given data points.
///
/// ## When to Use
/// Use `SplineInterpolation` to approximate unknown values within the range of 
/// known data points. Spline interpolation is particularly suited for scenarios
/// requiring smooth transitions and accurate representation of data trends.
///
/// # Required Method
///
/// ### `spline_interpolate`
///
/// Performs a spline interpolation to compute the `y` value at a given `x` coordinate.
///
/// - **Parameters:**
///   - `x`: A `Decimal` value representing the `x` coordinate at which the interpolation
///     should be performed.
///
/// - **Returns:**
///   - A `Result` where:
///     - `Ok(Point2D)`: The interpolated point in 2D space, with the given `x` coordinate 
///       and its corresponding `y` coordinate calculated using the spline.
///     - `Err(CurvesError)`: An error indicating why the interpolation failed. This could
///       be due to insufficient data, boundary conditions, or other issues.
///
/// # Error Handling
/// The method may return various `CurvesError` variants, such as:
/// - `InterpolationError`: Issues encountered during the spline computation.
/// - `Point2DError`: Errors related to specific points in the dataset (e.g., invalid points).
///
/// Implementors of this trait are expected to properly handle edge cases, such as
/// extrapolation requests, improper input datasets, or singularities in the spline.
///
/// # Required Dependencies
/// This trait depends on the following code components:
/// - [`Point2D`](crate::curves::types::Point2D): Represents a 2D Cartesian point where the 
///   x-coordinate is interpolated and the y-coordinate is calculated.
/// - [`CurvesError`](crate::error::curves::CurvesError): Represents errors encountered during 
///   interpolation.
///
/// # Example Usage
/// Implement this trait for your custom spline-based interpolator to handle 
/// specific datasets and interpolation logic, ensuring proper error handling
/// and achieving smooth, accurate approximations.
///
/// # Complementary Modules
/// This trait is part of the broader interpolation framework, which includes:
/// - `LinearInterpolation`: For linear interpolators.
/// - `CubicInterpolation`: For cubic interpolators.
/// - `BilinearInterpolation`: For bilinear interpolators.
///
/// By working together, these interpolators support a variety of approximation
/// techniques for diverse datasets and application requirements.
pub trait SplineInterpolation {
    /// Interpolates a y-value for the provided x-coordinate using spline interpolation.
    ///
    /// - **Parameters:**
    ///   - `x`: A `Decimal` representing the x-coordinate for which the y-value
    ///     needs to be interpolated.
    ///
    /// - **Returns:**
    ///   - `Ok(Point2D)`: Represents the interpolated (x, y) point.
    ///   - `Err(CurvesError)`: Indicates failure during the interpolation process.
    fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError>;
}