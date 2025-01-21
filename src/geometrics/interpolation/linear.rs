use crate::error::InterpolationError;

/// A trait that provides functionality for performing linear interpolation.
///
/// # Overview
/// The `LinearInterpolation` trait defines the behavior for linearly interpolating
/// values along a curve or dataset. It allows implementors to compute the interpolated
/// point for a given input using linear methods.
///
/// Linear interpolation is a simple and widely-used technique in which new data points
/// can be estimated within the range of a discrete set of known data points by connecting
/// adjacent points with straight lines.
///
/// # Associated Types
///
/// - **`Point`**: Represents the type of the interpolated data point (e.g., coordinates,
///   numerical values, or other types).
/// - **`Input`**: Represents the type of the input value used to calculate the interpolation (e.g., a single
///   number such as an `f64` or a high-precision number like `Decimal`).
/// - **`Error`**: Defines the error type returned if interpolation cannot be performed due to invalid input
///   or other constraints (e.g., insufficient points for interpolation).
///
/// # Required Method
///
/// ## `linear_interpolate`
///
/// This method calculates the interpolated value for a given input.
///
/// - **Input:**  
///   Requires a value of type `Input` (e.g., a numerical value or coordinate) for which
///   the corresponding interpolated `Point` should be calculated.
///
/// - **Returns:**  
///   - On success, returns a `Result` containing the interpolated value of type `Point`.  
///   - On failure, returns a `Result` containing an error of type `Self::Error` that describes
///     the reason for failure (e.g., invalid input or insufficient data points).
///
/// # Typical Usage
/// This trait is abstract and should be implemented for data structures representing curves
/// or data sets where linear interpolation is required. Examples include interpolating
/// between points on a 2D curve, estimating values in a time series, or refining graphical
/// data.
///
/// # Example Use Case (General)
/// This trait can be used to:
/// - Estimate missing values in a dataset.
/// - Provide smooth transitions between adjacent points on a graph.
/// - Implement real-time interpolations for dynamic systems, such as animations or simulations.
///
/// # Notes on Implementation
/// - Implementors of this trait must ensure that the interpolation logic respects the
///   shape and constraints of the data being used. This involves:
///   - Validating input values.
///   - Handling boundary conditions (e.g., extrapolation or edge points).
///
/// - High precision or custom input/output types (e.g., `Decimal`) can be used when necessary
///   to avoid issues related to floating-point errors in sensitive calculations.
///
/// # Example Implementation Outline
/// 1. Access two known points adjacent to the input value.
/// 2. Compute the slope and intercept of the line between the two points.
/// 3. Evaluate the equation of the line at the given input to determine the interpolated point.
///
/// # Errors
/// When implementing this trait, common errors that may be returned include:
/// - Insufficient data points for interpolation.
/// - Out-of-bounds input values or indices.
/// - Invalid data point structures or internal state errors.
///
/// # See Also
/// - [`crate::curves::interpolation::CubicInterpolation`]: An alternative interpolation method using cubic polynomials.
/// - [`crate::types::InterpolationType`]: Enum representing supported interpolation methods in the library.
///
/// The `LinearInterpolation` trait is part of a modular design and is often re-exported
/// in higher-level library components for ease of use.
pub trait LinearInterpolation<Point, Input> {

    /// Performs linear interpolation for a given input value.
    ///
    /// # Parameters
    /// - `x`: The input value of type `Input` for which the interpolated point should be calculated.
    ///
    /// # Returns
    /// - `Ok(Point)`: The calculated interpolated value of type `Point`.
    /// - `Err(Self::Error)`: An error indicating the reason why interpolation failed.
    fn linear_interpolate(&self, x: Input) -> Result<Point, InterpolationError>;
}