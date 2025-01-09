use rust_decimal::Decimal;
use crate::curves::Point2D;
use crate::error::CurvesError;

/// A trait that defines bilinear interpolation functionality.
///
/// # Overview
/// The `BiLinearInterpolation` trait provides an abstraction for performing bilinear
/// interpolation on a set of data. Bilinear interpolation is a mathematical
/// method used to estimate values within a grid of known points, commonly
/// used in 2D data, such as image processing, geographical mapping, or 
/// numerical simulations.
///
/// # Required Method
///
/// This trait requires the implementation of one method:
/// - `bilinear_interpolate`: Takes a single value `x` of type `Decimal` and calculates
///   the corresponding interpolated `Point2D`, or returns an error of type `CurvesError` 
///   if the interpolation cannot be performed.
///
/// # Usage
/// Implementing this trait on a struct makes that struct capable of performing
/// bilinear interpolation. This allows developers to use the underlying 
/// implementation to estimate intermediate values between known 2D points 
/// with high precision.
///
/// # Examples
/// The exact usage of this trait depends on its implementor. The `bilinear_interpolate`
/// method is intended to be invoked with relevant data, such as a `Decimal` value
/// representing the x-coordinate, to produce a `Point2D` result. Errors encountered
/// during the process are returned as `CurvesError` variants, such as:
/// - Missing or insufficient data for interpolation.
/// - Out-of-bound values for the given dataset.
/// - Other operation-specific issues.
///
/// # Errors
/// The method may return a `CurvesError` if:
/// - The input data is invalid for interpolation.
/// - Any computation failure occurs.
///
/// The specific type or reason for the error is encapsulated by the `CurvesError` enum.
///
/// # Related Types
/// - [`Point2D`](crate::curves::types::Point2D): The output type representing 
///   a point in 2D space.
/// - [`CurvesError`](crate::error::curves::CurvesError): The error type representing
///   different categories of errors that may occur during interpolation.
///
/// # See Also
/// - [`InterpolationType::Bilinear`](crate::types::InterpolationType): 
///   Represents bilinear interpolation as one of the general interpolation types in the module.
/// - [`LinearInterpolation`](crate::linear::LinearInterpolation): Similar interpolation
///   type for one-dimensional data.
/// - [`CubicInterpolation`](crate::cubic::CubicInterpolation): Higher-order interpolation
///   for smooth curve fitting in one dimension.
pub trait BiLinearInterpolation {
    /// Performs bilinear interpolation on the implementing data structure.
    ///
    /// # Parameters
    /// - `x`: A `Decimal` value representing the x-coordinate for the interpolation.
    ///
    /// # Returns
    /// - `Ok(Point2D)`: A `Point2D` instance with the interpolated values.
    /// - `Err(CurvesError)`: An error indicating the reason for the failure during the interpolation process.
    fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError>;
}