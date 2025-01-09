/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use crate::curves::construction::CurveConstructionMethod;
use crate::curves::interpolation::{
    BiLinearInterpolation, CubicInterpolation, Interpolate, InterpolationType, LinearInterpolation,
    SplineInterpolation,
};
use crate::error::curves::CurvesError;
use crate::model::positive::is_positive;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2D {
    pub x: Decimal,
    pub y: Decimal,
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

/// Represents a mathematical curve as a collection of 2D points.
///
/// The `Curve` struct is a fundamental representation of a curve, defined as a series
/// of points in a two-dimensional Cartesian coordinate system (`Point2D` instances).
/// Each curve is also associated with an `x_range`, specifying the inclusive domain
/// of the curve in terms of its x-coordinates.
///
/// # Overview
/// The `Curve` structure supports precise mathematical and computational operations,
/// including interpolation, analysis, transformations, and intersections. The use of
/// `Decimal` for `x_range` ensures high-precision calculations, making the struct
/// particularly suitable for scientific, financial, or mathematical applications.
///
/// # Fields
/// - **points**: 
///   - A vector of `Point2D` values representing the points that define the curve.
///   - Points must ideally be ordered along the x-axis to ensure meaningful interpolation
///     and transformation operations.
/// - **x_range**:
///   - A tuple `(min_x, max_x)`, where both values are of type `Decimal`.
///   - Specifies the range of valid x-values for the curve.
///   - Helps in ensuring that any external operation on the curve (e.g., interpolation
///     or slicing) remains within the computed bounds.
///
/// # Derivable Traits
/// - `Debug`: Enables formatted output of the `Curve` for debugging purposes.
/// - `Clone`: Allows duplication of a `Curve`, including all associated points and range.
///
/// # Usage
/// The `Curve` struct acts as the basis for high-level operations provided within
/// the `crate::curves` module. These include (but are not limited to):
/// - Generating statistical analyses (`CurveAnalysisResult`)
/// - Performing curve interpolation
/// - Logical manipulations, such as merging curves (`MergeOperation`)
/// - Visualizing graphs or curve plots using libraries like `plotters`
///
/// # Example Applications
/// The `Curve` type fits into mathematical or graphical operations such as:
/// - Modeling data over a range of x-values
/// - Comparing curves through transformations or intersections
/// - Calculating derivatives, integrals, and extrema along the curve
///
/// Since curves rely on the `Point2D` structure for representation, this type adheres
/// to the same precision and consistency expected from high-quality numerical computations.
///
/// # Constraints
/// - All points in the `points` vector must lie within the specified `x_range`.
/// - Methods working with `Curve` data will assume that the `points` vector is ordered
///   by the `x`-coordinate. Non-ordered inputs may lead to undefined behavior in specific
///   operations.
///
/// # See Also
/// - [`Point2D`](crate::curves::types::Point2D): The fundamental data type for representing points in 2D space.
/// - [`CurveOperations`](crate::curves::curve_traits::CurveOperations): The trait providing operations on curves.
/// - [`MergeOperation`](crate::curves::operations::arithmetic::MergeOperation): Enum for combining multiple curves.
///
/// # Fields
/// - **points**:
///   - A vector of `Point2D` objects that defines the curve in terms of its x-y plane coordinates.
/// - **x_range**:
///   - A tuple `(Decimal, Decimal)` that specifies the minimum and maximum x-coordinate values
///     for the curve. Operations performed on the curve should ensure they fall within this range.
#[derive(Debug, Clone)]
pub struct Curve {
    pub points: Vec<Point2D>,
    pub x_range: (Decimal, Decimal),
}

impl Curve {
    
    /// Creates a new curve from a vector of points.
    ///
    /// This constructor initializes a `Curve` instance using a list of 2D points
    /// provided as a `Vec<Point2D>`. Additionally, the x-range of the curve is calculated
    /// and stored. The x-range is determined by evaluating the minimum and maximum
    /// x-coordinates among the provided points.
    ///
    /// # Parameters
    ///
    /// - `points` (`Vec<Point2D>`): A vector of points that define the curve in a
    ///   two-dimensional Cartesian coordinate plane.
    ///
    /// # Returns
    ///
    /// - `Curve`: A newly instantiated curve containing the provided points and
    ///   the computed x-range.
    ///
    /// # Behavior
    ///
    /// - Calculates the x-range of the points using `calculate_range()`.
    /// - Stores the provided points for later use in curve-related calculations.
    ///
    /// # See Also
    ///
    /// - [`Point2D`]: The type of points used to define the curve.
    /// - [`calculate_range`]: Computes the x-range of a set of points.
    pub fn new(points: Vec<Point2D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }

    /// Calculates the range of x values in the curve.
    ///
    /// This function computes the minimum and maximum x values from an iterator of `Decimal`
    /// inputs, representing x-coordinates of points. It returns a tuple containing the
    /// minimum and maximum x values. The computation is efficient and involves a single
    /// traversal of the iterator.
    ///
    /// # Parameters
    ///
    /// - `iter` (`Iterator<Item = Decimal>`): An iterator over x-coordinates of points.
    ///
    /// # Returns
    ///
    /// - `(Decimal, Decimal)`: A tuple where:
    ///   - The first value is the minimum x-coordinate.
    ///   - The second value is the maximum x-coordinate.
    ///
    /// # Behavior
    ///
    /// - Iterates over the input to compute the x-range in a fold operation.
    /// - Returns `(Decimal::MAX, Decimal::MIN)` for an empty iterator (although such
    ///   cases are expected to be handled elsewhere).
    fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
    where
        I: Iterator<Item = Decimal>,
    {
        iter.fold((Decimal::MAX, Decimal::MIN), |(min, max), val| {
            (min.min(val), max.max(val))
        })
    }

    /// Constructs a curve using the specified construction method and returns the result.
    ///
    /// This function supports two distinct curve construction strategies:
    /// - **FromData**: Directly constructs a curve using pre-defined 2D points.
    /// - **Parametric**: Algorithmically builds a curve based on a parameterized
    ///   function over a given range and number of steps.
    ///
    /// # Parameters
    ///
    /// - `method` (`CurveConstructionMethod`): Specifies the strategy for constructing the curve.
    ///   Options include `FromData` (explicit points) or `Parametric` (function-based).
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: The successfully constructed curve.
    /// - `Err(CurvesError)`: Indicates errors during construction.
    ///
    /// # Behavior
    ///
    /// ## FromData
    ///
    /// - Validates that the input points vector is not empty.
    /// - Returns an error (`CurvesError::Point2DError`) if the points vector is empty.
    /// - Constructs the curve using the provided points.
    ///
    /// ## Parametric
    ///
    /// - Divides the range `[t_start, t_end]` into `steps` intervals.
    /// - Computes points by evaluating a parameterized function `f` at each step using parallel
    ///   processing (`rayon`).
    /// - Fails gracefully with a `CurvesError` if the function `f` encounters issues.
    ///
    /// # Errors
    ///
    /// - **FromData**:
    ///   - Returns an error if an empty set of points is provided.
    /// - **Parametric**:
    ///   - Generates an error if the function `f` produces invalid results.
    ///
    /// # Details
    ///
    /// - Efficiently computes points in the parametric mode using parallel processing
    ///   provided by the `rayon` crate.
    ///
    /// # See Also
    ///
    /// - [`CurveConstructionMethod`]: Enum defining the supported construction strategies.
    /// - [`CurvesError`]: Represents possible errors encountered during curve construction.
    /// - [`Point2D`]: The data type representing a 2D point.
    pub fn construct(method: CurveConstructionMethod) -> Result<Self, CurvesError> {
        match method {
            CurveConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(CurvesError::Point2DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Curve::new(points))
            }

            CurveConstructionMethod::Parametric {
                f,
                t_start,
                t_end,
                steps,
            } => {
                let step_size = (t_end - t_start) / Decimal::from(steps);

                let points: Result<Vec<Point2D>, CurvesError> = (0..=steps)
                    .into_par_iter()
                    .map(|i| {
                        let t = t_start + step_size * Decimal::from(i);
                        f(t).map_err(|e| CurvesError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(|points| Curve::new(points))
            }
        }
    }
}

/// Implements the `LinearInterpolation` trait for the `Curve` struct.
///
/// This implementation provides linear interpolation functionality for a given set
/// of points on a curve. The interpolation computes the `y` value corresponding 
/// to a given `x` value using the linear interpolation formula. The method ensures 
/// that the input `x` is within the range of the curve's defined points.
///
/// ```text
/// y = p1.y + (x - p1.x) * (p2.y - p1.y) / (p2.x - p1.x)
/// ```
///
/// # Parameters
/// - `x`: A `Decimal` representing the `x`-coordinate for which the corresponding 
///   interpolated `y` value is to be computed.
///
/// # Returns
/// - `Ok(Point2D)`: A `Point2D` instance containing the input `x` value and the
///   interpolated `y` value.
/// - `Err(CurvesError)`: Returns an error of type `CurvesError::InterpolationError`
///   in any of the following cases:
///     - The curve does not have enough points for interpolation.
///     - The provided `x` value is outside the range of the curve's points.
///     - Bracketing points for `x` cannot be found.
///
/// # Working Mechanism
/// 1. The method calls `find_bracket_points` (implemented in the `Interpolate` trait)
///    to locate the index pair `(i, j)` of two points that bracket the `x` value.
/// 2. From the located points `p1` and `p2`, the method calculates the interpolated
///    `y` value using the linear interpolation formula.
/// 3. Finally, a `Point2D` is created and returned with the provided `x` and the computed
///    `y` value.
///
/// # Implementation Details
/// - The function leverages `Decimal` arithmetic for high precision in calculations.
/// - It assumes that the provided points on the curve are sorted in ascending order
///   based on their `x` values.
///
/// # Errors
/// This method returns a `CurvesError` in the following cases:
/// - **Insufficient Points**: When the curve has fewer than two points.
/// - **Out-of-Range `x`**: When the input `x` value lies outside the range of the
///   defined points.
/// - **No Bracketing Points Found**: When the method fails to find two points
///   that bracket the given `x`.
///
/// # Example (How it works internally)
/// Suppose the curve is defined by the following points:
/// - `p1 = (2.0, 4.0)`
/// - `p2 = (5.0, 10.0)`
///
/// Given `x = 3.0`, the method computes:
/// ```text
/// y = 4.0 + (3.0 - 2.0) * (10.0 - 4.0) / (5.0 - 2.0)
///   = 4 + 1 * 6 / 3
///   = 4 + 2
///   = 6.0
/// ```
/// It will return `Point2D { x: 3.0, y: 6.0 }`.
///
/// # See Also
/// - `find_bracket_points`: Finds two points that bracket a value.
/// - `Point2D`: Represents points in 2D space.
/// - `CurvesError`: Represents errors related to curve operations.
impl LinearInterpolation for Curve {
    
    /// # Method
    /// ### `linear_interpolate`
    ///
    /// Performs linear interpolation for a given `x` value by finding two consecutive
    /// points on the curve (`p1` and `p2`) that bracket the provided `x`. The `y` value 
    /// is then calculated using the linear interpolation formula:
    fn linear_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError> {
        let (i, j) = self.find_bracket_points(x)?;

        let p1 = &self.points[i];
        let p2 = &self.points[j];

        // Linear interpolation for y value
        let y = p1.y + (x - p1.x) * (p2.y - p1.y) / (p2.x - p1.x);

        Ok(Point2D::new(x, y))
    }
}

/// Implementation of the `BiLinearInterpolation` trait for the `Curve` struct.
///
/// This function performs bilinear interpolation, which is used to estimate the value
/// of a function at a given point `x` within a grid defined by at least 4 points.
/// Bilinear interpolation combines two linear interpolations: one along the x-axis
/// and another along the y-axis, within the bounds defined by the four surrounding points.
///
/// # Parameters
///
/// - **`x`**: The x-coordinate value for which the interpolation will be performed.
///   Must fall within the range of the x-coordinates of the curve's points.
///
/// # Returns
///
/// - **`Ok(Point2D)`**: A `Point2D` instance representing the interpolated point
///   at the given x-coordinate, with both x and y provided as `Decimal` values.
/// - **`Err(CurvesError)`**: An error if the interpolation cannot be performed due
///   to one of the following reasons:
///   - There are fewer than 4 points in the curve.
///   - The x-coordinate does not fall within a valid range of points.
///   - The bracketing points for the given x-coordinate cannot be determined.
///
/// # Function Details
///
/// 1. **Input Validation**:
///    - Ensures the curve has at least 4 points, as required for bilinear interpolation.
///    - Returns an error if the condition is not met.
///
/// 2. **Exact Match Check**:
///    - If the x-coordinate matches exactly with one of the points in the curve,
///      the corresponding `Point2D` is returned directly.
///
/// 3. **Bracket Point Search**:
///    - Determines the bracketing points (`i` and `j`) for the given x-coordinate
///      using the `find_bracket_points` method.
///
/// 4. **Grid Point Selection**:
///    - Extracts four points from the curve:
///      - `p11`: Bottom-left point.
///      - `p12`: Bottom-right point.
///      - `p21`: Top-left point.
///      - `p22`: Top-right point.
///
/// 5. **x-Normalization**:
///    - Computes a normalized x value (`dx` in the range `[0,1]`), used to perform
///      interpolation along the x-axis within the defined grid.
///
/// 6. **Linear Interpolation**:
///    - First performs interpolation along the x-axis for the bottom and top edges of
///      the grid, resulting in partial y-values `bottom` and `top`.
///    - Then, interpolates along the y-axis between the bottom and top edge values,
///      resulting in the final interpolated y-coordinate.
///
/// 7. **Output**:
///    - Returns the interpolated `Point2D` with the input x-coordinate and the computed y-coordinate.
///
/// # Errors
///
/// - **Insufficient Points**: If the curve contains fewer than 4 points, a `CurvesError`
///   with a relevant message is returned.
/// - **Out-of-Range x**: If the x-coordinate cannot be bracketed by points in the curve,
///   a `CurvesError` is returned with an appropriate message.
///
/// # Related Traits
///
/// - [`BiLinearInterpolation`](crate::curves::interpolation::bilinear::BiLinearInterpolation): The trait defining this method.
/// - [`Interpolate`](crate::curves::interpolation::traits::Interpolate): Ensures compatibility of the curve with multiple interpolation methods.
///
/// # See Also
///
/// - [`Curve`](crate::curves::types::Curve): The overarching structure that represents the curve.
/// - [`Point2D`](crate::curves::types::Point2D): The data type used to represent individual points on the curve.
/// - [`find_bracket_points`](crate::curves::interpolation::traits::Interpolate::find_bracket_points):
///   A helper method used to locate the two points that bracket the given x-coordinate.
impl BiLinearInterpolation for Curve {

    /// Performs bilinear interpolation to find the value of the curve at a given `x` coordinate.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate at which the interpolation is to be performed. This should be a `Decimal` value
    ///        within the range of the curve's known points. 
    ///
    /// # Returns
    /// - On success, returns a `Point2D` instance representing the interpolated point at the given `x` value.
    /// - On failure, returns a `CurvesError`:
    ///   - `CurvesError::InterpolationError`: If there are fewer than four points available for interpolation or
    ///                                         if the required conditions for interpolation are not met.
    ///
    /// # Function Description
    /// - The function retrieves the set of points defining the curve using `self.get_points()`.
    /// - If fewer than four points exist, the function immediately fails with an `InterpolationError`.
    /// - If the exact `x` value is found in the point set, its corresponding `Point2D` is returned directly.
    /// - Otherwise, it determines the bracketing points (two pairs of points forming a square grid) necessary
    ///   for bilinear interpolation using `self.find_bracket_points()`.
    /// - From the bracketing points, it computes:
    ///   - `dx`: A normalized value representing the relative position of `x` between its bracketing x-coordinates
    ///           in the [0,1] interval.
    ///   - `bottom`: The interpolated y-value along the bottom edge of the grid.
    ///   - `top`: The interpolated y-value along the top edge of the grid.
    ///   - `y`: The final interpolated value along the y-dimension from `bottom` to `top`.
    /// - Returns the final interpolated point as `Point2D(x, y)`.
    ///
    /// # Errors
    /// - Returns an error if the curve has fewer than four points, as bilinear interpolation requires at least four.
    /// - Returns an error from `self.find_bracket_points()` if `x` cannot be bracketed.
    ///
    /// # Notes
    /// - The input `x` should be within the bounds of the curve for interpolation to succeed,
    ///   as specified by the bracketing function.
    /// - This function assumes that the points provided by `get_points` are sorted by ascending x-coordinate.
    ///
    /// # Example Use Case
    /// This method is useful for calculating intermediate values on a 2D grid when exact measurements are unavailable.
    /// Bilinear interpolation is particularly applicable for approximating smoother values in a tabular dataset 
    /// or a regularly sampled grid.
    fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError> {
        let points = self.get_points();

        // Need at least 4 points for bilinear interpolation
        if points.len() < 4 {
            return Err(CurvesError::InterpolationError(
                "Need at least four points for bilinear interpolation".to_string(),
            ));
        }

        // For exact points, return the actual point value
        if let Some(point) = points.iter().find(|p| p.x == x) {
            return Ok(*point);
        }

        let (i, j) = self.find_bracket_points(x)?;

        // Get four points forming a grid
        let p11 = &points[i]; // Bottom left
        let p12 = &points[i + 1]; // Bottom right
        let p21 = &points[i + 2]; // Top left
        let p22 = &points[i + 3]; // Top right

        // Normalize x to [0,1] interval
        let dx = (x - p11.x) / (p12.x - p11.x);

        // Interpolate along bottom edge
        let bottom = p11.y + dx * (p12.y - p11.y);

        // Interpolate along top edge
        let top = p21.y + dx * (p22.y - p21.y);

        // Final interpolation in y direction
        let y = bottom + (top - bottom) / dec!(2);

        Ok(Point2D::new(x, y))
    }
}

/// Implements the `CubicInterpolation` trait for the `Curve` struct, 
/// providing an algorithm for cubic interpolation utilizing a Catmull-Rom spline.
///
/// # Method: `cubic_interpolate`
///
/// ## Parameters
/// - **`x`**: The x-value at which the interpolation is performed. This value must
///   be within the range of x-values in the curve's defined points, and it is passed
///   as a `Decimal` to allow for high-precision computation.
///
/// ## Returns
/// - **`Ok(Point2D)`**: Returns a `Point2D` representing the interpolated x and y values.
/// - **`Err(CurvesError)`**: Returns an error if:
///   - There are fewer than 4 points available for interpolation.
///   - The x-value is outside the curve's range, or interpolation fails for any other reason.
///
/// ## Behavior
/// 1. **Point Validation**: Ensures at least four points exist for cubic interpolation,
///    as this is a fundamental requirement for computing the Catmull-Rom spline.
/// 2. **Exact Match Check**: If the x-value matches an existing point in the curve, the
///    method directly returns the corresponding `Point2D` without further computation.
/// 3. **Bracket Points**: Determines the bracketing points (4 points total) around the
///    provided x-value. Depending on the position of the x-value in the curve, the
///    method dynamically adjusts the selected points to ensure they form a proper bracket:
///    - If near the start of the curve, uses the first four points.
///    - If near the end, uses the last four points.
///    - Else, selects points before and after x to define the bracket.
/// 4. **Parameter Calculation**: Computes a normalized parameter `t` that represents
///    the relative position of the target x-value between `p1` and `p2`.
/// 5. **Catmull-Rom Spline**: Performs cubic interpolation using a Catmull-Rom spline,
///    a widely used, smooth spline algorithm. The coefficients are calculated based on
///    the relative x position and the y-values of the four surrounding points.
/// 6. **Interpolation**: Calculates the interpolated y-value using the cubic formula:
///    ```text
///    y(t) = 0.5 * (
///        2 * p1.y + (-p0.y + p2.y) * t
///        + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t^2
///        + (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * t^3
///    )
///    ```
///    Here, `t` is the normalized x position, and `p0`, `p1`, `p2`, `p3` are the four bracketed points.
///
/// ## Errors
/// - Returns an error of type `CurvesError::InterpolationError` if any issues are encountered,
///   such as insufficient points or the inability to locate bracket points.
///
/// ## Example
/// This method is part of the `Curve` struct, which defines a set of points and supports interpolation.
/// It is often used in applications requiring smooth manifolds or animations.
///
/// ## Notes
/// - The computed y-value ensures smooth transitions and continuity between interpolated segments.
/// - Catmull-Rom splines are particularly effective for creating visually smooth transitions,
///   making this method suitable for curves, animations, and numerical analysis.
///
/// # See Also
/// - [`CubicInterpolation`](crate::curves::interpolation::cubic::CubicInterpolation): The trait defining this method.
/// - [`Point2D`](crate::curves::types::Point2D): Represents the points used for interpolation.
/// - [`find_bracket_points`](crate::curves::interpolation::traits::Interpolate::find_bracket_points): Determines the bracketing points required for interpolation.
impl CubicInterpolation for Curve {

    /// Performs cubic interpolation on a set of points to estimate the y-coordinate
    /// for a given x value using a Catmull-Rom spline.
    ///
    /// # Parameters
    ///
    /// - `x`: The x-coordinate for which the interpolation is performed. This value
    ///   should lie within the range of the points on the curve.
    ///
    /// # Returns
    ///
    /// - `Ok(Point2D)`: A `Point2D` instance representing the interpolated position
    ///   `(x, y)`, where `y` is estimated using cubic interpolation.
    /// - `Err(CurvesError)`: An error indicating issues with the interpolation process,
    ///   such as insufficient points or an out-of-range x value.
    ///
    /// # Requirements
    ///
    /// - The number of points in the curve must be at least 4, as cubic interpolation
    ///   requires four points for accurate calculations.
    /// - The specified `x` value should be inside the range defined by the curve's points.
    /// - If the specified x matches an existing point on the curve, the interpolated result
    ///   directly returns that exact point.
    ///
    /// # Functionality
    ///
    /// This method performs cubic interpolation using the general properties of the
    /// Catmull-Rom spline, which is well-suited for smooth curve fitting. It operates as follows:
    ///
    /// 1. **Exact Point Check**: If the x value matches an existing point, the method
    ///    returns that point without further processing.
    ///
    /// 2. **Bracketing Points Selection**:
    ///    - Searches for two points that bracket the given x value (using `find_bracket_points`
    ///      from the `Interpolate` trait). The method ensures that there are always enough
    ///      points before and after the target x value to perform cubic interpolation.
    ///
    /// 3. **Point Selection for Interpolation**:
    ///    - Depending on the position of the target x value, four points (`p0, p1, p2, p3`)
    ///      are selected:
    ///        - When `x` is near the start of the points, select the first four.
    ///        - When `x` is near the end, select the last four.
    ///        - Otherwise, select the two points just before and after the x value and
    ///          include an additional adjacent point on either side.
    ///
    /// 4. **Parameter Calculation**:
    ///    - The `t` parameter is derived, representing the normalized position of x
    ///      between `p1` and `p2`.
    ///
    /// 5. **Cubic Interpolation**:
    ///    - The interpolated y-coordinate is computed using the Catmull-Rom spline formula,
    ///      leveraging the `t`-value and the y-coordinates of the four selected points.
    ///
    /// # Error Handling
    ///
    /// This method returns an error in the following circumstances:
    /// - If fewer than 4 points are available, it returns a `CurvesError::InterpolationError`
    ///   with a corresponding message.
    /// - If the bracketing points cannot be identified (e.g., when `x` is outside the
    ///   range of points), the appropriate interpolation error is propagated.
    ///
    /// # Example
    ///
    /// - Interpolating smoothly along a curve defined by a set of points, avoiding sharp
    ///   transitions between segments.
    ///
    /// - Provides a high degree of precision due to the use of the `Decimal` type for
    ///   `x` and `y` calculations.
    fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError> {
        let points = self.get_points();

        // Need at least 4 points for cubic interpolation
        if points.len() < 4 {
            return Err(CurvesError::InterpolationError(
                "Need at least four points for cubic interpolation".to_string(),
            ));
        }

        // For exact points, return the actual point value
        if let Some(point) = points.iter().find(|p| p.x == x) {
            return Ok(*point);
        }

        let (i, _) = self.find_bracket_points(x)?;

        // Select four points for interpolation
        // Ensuring we always have enough points before and after
        let (p0, p1, p2, p3) = if i == 0 {
            (&points[0], &points[1], &points[2], &points[3])
        } else if i == points.len() - 2 {
            (
                &points[points.len() - 4],
                &points[points.len() - 3],
                &points[points.len() - 2],
                &points[points.len() - 1],
            )
        } else {
            (&points[i - 1], &points[i], &points[i + 1], &points[i + 2])
        };

        // Calculate t parameter (normalized x position between p1 and p2)
        let t = (x - p1.x) / (p2.x - p1.x);

        // Cubic interpolation using Catmull-Rom spline
        let t2 = t * t;
        let t3 = t2 * t;

        let y = dec!(0.5)
            * (dec!(2) * p1.y
                + (-p0.y + p2.y) * t
                + (dec!(2) * p0.y - dec!(5) * p1.y + dec!(4) * p2.y - p3.y) * t2
                + (-p0.y + dec!(3) * p1.y - dec!(3) * p2.y + p3.y) * t3);

        Ok(Point2D::new(x, y))
    }
}

/// Implements the `SplineInterpolation` trait for the `Curve` struct, providing functionality
/// to perform cubic spline interpolation.
///
/// # Overview
/// This method calculates the interpolated `y` value for a given `x` value by using cubic 
/// spline interpolation on the points in the `Curve`. The method ensures a smooth transition 
/// between points by computing second derivatives of the curve at each point, and uses those 
/// derivatives in the spline interpolation formula.
///
/// # Parameters
/// - `x`: The x-coordinate at which the curve should be interpolated. This value is passed as 
///   a `Decimal` for precise calculations.
///
/// # Returns
/// - On success, returns a `Point2D` instance representing the interpolated point.
/// - On error, returns a `CurvesError` indicating the reason for failure (e.g., insufficient points 
///   or an out-of-range x-coordinate).
///
/// # Errors
/// - Returns `CurvesError::InterpolationError` with an appropriate error message in the following cases:
///   - If the curve contains fewer than three points, as spline interpolation requires at least three points.
///   - If the given `x` value lies outside the range of x-coordinates spanned by the points in the curve.
///   - If a valid segment for interpolation cannot be located.
///
/// # Details
/// 1. **Validation**: 
///    - Ensures that there are at least three points in the curve for spline interpolation.
///    - Validates that the provided `x` value is within the range of `x` values of the curve.
/// 2. **Exact Match**: If the `x` value matches the x-coordinate of an existing point, the corresponding 
///    `Point2D` is returned immediately.
/// 3. **Second Derivative Calculation**:
///    - Uses a tridiagonal matrix to compute the second derivatives at each point. This step 
///      involves setting up the system of equations based on the boundary conditions (natural spline) 
///      and solving it using the Thomas algorithm.
/// 4. **Segment Identification**:
///    - Determines the segment (interval between two consecutive points) in which the provided `x` value lies.
/// 5. **Interpolation**:
///    - Computes the interpolated y-coordinate using the cubic spline formula, which is based on 
///      the second derivatives and the positions of the surrounding points.
///
/// # Implementation Notes
/// - This implementation uses `Decimal` from the `rust_decimal` crate to ensure high precision 
///   in calculations, making it suitable for scientific and financial applications.
/// - The Thomas algorithm is employed to solve the tridiagonal matrix system efficiently.
/// - The method assumes natural spline boundary conditions, where the second derivatives at the 
///   endpoints are set to zero, ensuring a smooth and continuous curve shape.
///
/// # Example Usage
/// Refer to the documentation for how to use the `SplineInterpolation` trait, as examples 
/// are not provided inline with this implementation.
///
/// # See Also
/// - [`SplineInterpolation`](crate::curves::interpolation::spline::SplineInterpolation): The trait definition for spline interpolation.
/// - [`Point2D`](crate::curves::types::Point2D): Represents a point in 2D space.
/// - [`Curve`](crate::curves::types::Curve): Represents a mathematical curve made up of points for interpolation.
/// - [`CurvesError`](crate::curves::errors::CurvesError): Enumerates possible errors during curve operations.
impl SplineInterpolation for Curve {

    /// Performs cubic spline interpolation for a given x-coordinate and returns the interpolated 
    /// `Point2D` value. This function computes the second derivatives of the curve points, solves 
    /// a tridiagonal system to derive the interpolation parameters, and evaluates the spline 
    /// function for the provided `x` value.
    ///
    /// # Parameters
    ///
    /// - `x`:
    ///   - The x-coordinate at which the interpolation is to be performed.
    ///   - Must be of type `Decimal`.
    ///
    /// # Returns
    ///
    /// - `Ok(Point2D)`:
    ///   - The `Point2D` instance representing the interpolated point at the given `x` value.
    ///   - The interpolated `y` value is calculated based on the cubic spline interpolation algorithm.
    ///
    /// - `Err(CurvesError)`:
    ///   - Returned when an error occurs during the interpolation process, such as:
    ///     - Insufficient points provided (less than 3 points).
    ///     - The given `x` is outside the valid range of the points.
    ///     - Unable to determine the correct segment for interpolation.
    ///
    /// # Errors
    ///
    /// - `CurvesError::InterpolationError`:
    ///   - Occurs under the following conditions:
    ///     - **"Need at least three points for spline interpolation"**: 
    ///       Requires at least 3 points to perform cubic spline interpolation.
    ///     - **"x is outside the range of points"**: 
    ///       The provided `x` value lies outside the domain of the curve points.
    ///     - **"Could not find valid segment for interpolation"**: 
    ///       Spline interpolation fails due to an invalid segment determination.
    ///
    /// # Pre-conditions
    ///
    /// - The curve must contain at least three points for cubic spline interpolation.
    /// - The `x` value must fall within the range of the curve's x-coordinates.
    ///
    /// # Implementation Details
    ///
    /// - **Inputs**:
    ///   - Uses the `get_points` method of the curve to retrieve the list of `Point2D` instances 
    ///     that define the interpolation curve.
    ///   - Computes the second derivatives (`m`) for each point using the Thomas algorithm to solve 
    ///     a tridiagonal system.
    /// - **Boundary Conditions**:
    ///   - Natural spline boundary conditions are used, with the second derivatives on the boundary 
    ///     set to zero.
    /// - **Interpolation**:
    ///   - Determines the segment `[x_i, x_{i+1}]` to which the input `x` belongs.
    ///   - Uses the cubic spline equation to calculate the interpolated `y` value.
    ///
    /// # Mathematical Formulation
    ///
    /// Let `x_i`, `x_{i+1}`, `y_i`, `y_{i+1}` refer to the points of the segment where `x` lies. 
    /// The cubic spline function at `x` is computed as follows:
    ///
    /// ```text
    /// S(x) = m_i * (x_{i+1} - x)^3 / (6 * h) 
    ///      + m_{i+1} * (x - x_i)^3 / (6 * h) 
    ///      + (y_i / h - h * m_i / 6) * (x_{i+1} - x) 
    ///      + (y_{i+1} / h - h * m_{i+1} / 6) * (x - x_i)
    /// ```
    ///
    /// Where:
    /// - `m_i`, `m_{i+1}` are the second derivatives at `x_i` and `x_{i+1}`.
    /// - `h = x_{i+1} - x_i` is the distance between the two points.
    /// - `(x_{i+1} - x)` and `(x - x_i)` are the relative distances within the segment.
    ///
    /// # Example Usages (Non-code)
    ///
    /// This method is typically used for high-precision curve fitting or graphical rendering where 
    /// smooth transitions between points are essential. Common applications include:
    /// - Signal processing.
    /// - Data interpolation for missing values.
    /// - Smooth graphical representations of mathematical functions.
    ///
    /// # Related Types
    ///
    /// - [`Point2D`](crate::curves::types::Point2D): Represents a 2D point and is used as input/output 
    ///   for this function.
    /// - [`CurvesError`](crate::curves::types::CurvesError): Represents any error encountered during 
    ///   interpolation.
    ///
    /// # Performance
    ///
    /// - The function operates with `O(n)` complexity, where `n` is the number of points. The 
    ///   tridiagonal system is solved efficiently using the Thomas algorithm.
    ///
    /// # Notes
    ///
    /// - Natural spline interpolation may introduce minor deviations beyond the range of existing 
    ///   data points due to its boundary conditions. For strictly constrained results, consider 
    ///   alternative interpolation methods, such as linear or cubic Hermite interpolation.
    fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, CurvesError> {
        let points = self.get_points();

        // Need at least 3 points for spline interpolation
        if points.len() < 3 {
            return Err(CurvesError::InterpolationError(
                "Need at least three points for spline interpolation".to_string(),
            ));
        }

        // Check if x is within the valid range
        if x < points[0].x || x > points[points.len() - 1].x {
            return Err(CurvesError::InterpolationError(
                "x is outside the range of points".to_string(),
            ));
        }

        // For exact points, return the actual point value
        if let Some(point) = points.iter().find(|p| p.x == x) {
            return Ok(*point);
        }

        let n = points.len();

        // Calculate second derivatives
        let mut a = vec![Decimal::ZERO; n];
        let mut b = vec![Decimal::ZERO; n];
        let mut c = vec![Decimal::ZERO; n];
        let mut r = vec![Decimal::ZERO; n];

        // Fill the matrices
        for i in 1..n-1 {
            let hi = points[i].x - points[i-1].x;
            let hi1 = points[i+1].x - points[i].x;

            a[i] = hi;
            b[i] = dec!(2) * (hi + hi1);
            c[i] = hi1;

            r[i] = dec!(6) * (
                (points[i+1].y - points[i].y) / hi1 -
                    (points[i].y - points[i-1].y) / hi
            );
        }

        // Add boundary conditions (natural spline)
        b[0] = dec!(1);
        b[n-1] = dec!(1);

        // Solve tridiagonal system using Thomas algorithm
        let mut m = vec![Decimal::ZERO; n];

        for i in 1..n-1 {
            let w = a[i] / b[i-1];
            b[i] = b[i] - w * c[i-1];
            r[i] = r[i] - w * r[i-1];
        }

        m[n-1] = r[n-1] / b[n-1];
        for i in (1..n-1).rev() {
            m[i] = (r[i] - c[i] * m[i+1]) / b[i];
        }

        // Find segment for interpolation
        let mut segment = None;
        for i in 0..n-1 {
            if points[i].x <= x && x <= points[i+1].x {
                segment = Some(i);
                break;
            }
        }

        let segment = segment.ok_or_else(|| {
            CurvesError::InterpolationError("Could not find valid segment for interpolation".to_string())
        })?;

        // Calculate interpolated value
        let h = points[segment+1].x - points[segment].x;
        let dx = points[segment+1].x - x;
        let dx1 = x - points[segment].x;

        let y = m[segment] * dx * dx * dx / (dec!(6) * h) +
            m[segment+1] * dx1 * dx1 * dx1 / (dec!(6) * h) +
            (points[segment].y / h - m[segment] * h / dec!(6)) * dx +
            (points[segment+1].y / h - m[segment+1] * h / dec!(6)) * dx1;

        Ok(Point2D::new(x, y))
    }
}

/// Implementation of the `Interpolate` trait for the `Curve` struct.
///
/// This implementation integrates the `get_points` method for the `Curve` structure,
/// providing access to its internal points. The `Interpolate` trait ensures compatibility
/// with various interpolation methods such as Linear, BiLinear, Cubic, and Spline
/// interpolations. By implementing this trait, `Curve` gains the ability to perform
/// interpolation operations and access bracketing points.
///
/// # Traits Involved
///
/// The `Interpolate` trait is an aggregation of multiple interpolation-related traits:
/// - [`LinearInterpolation`](crate::curves::interpolation::traits::LinearInterpolation)
/// - [`BiLinearInterpolation`](crate::curves::interpolation::traits::BiLinearInterpolation)
/// - [`CubicInterpolation`](crate::curves::interpolation::traits::CubicInterpolation)
/// - [`SplineInterpolation`](crate::curves::interpolation::traits::SplineInterpolation)
///
/// These underlying traits implement specific interpolation algorithms,
/// enabling `Curve` to support a robust set of interpolation options through the associated methods.
/// Depending on the use case and provided parameters (e.g., interpolation type and target x-coordinate),
/// the appropriate algorithm is invoked.
///
/// # See Also
///
/// - [`Curve`](crate::curves::types::Curve): The underlying mathematical structure being interpolated.
/// - [`Point2D`](crate::curves::types::Point2D): The fundamental data type for the curve's points.
/// - [`Interpolate`](crate::curves::interpolation::traits::Interpolate): The trait defining interpolation operations.
///
impl Interpolate for Curve {


    /// ## `get_points`
    ///
    /// - **Signature**: `fn get_points(&self) -> &[Point2D]`
    /// - **Purpose**: Provides a reference to the collection of points that define the curve.
    /// - **Returns**: A slice of `Point2D` instances contained in the `points` vector of the struct.
    /// - **Usage**: This method is critical for interpolation algorithms, allowing access to the
    ///   ordered list of points necessary for calculations.
    fn get_points(&self) -> &[Point2D] {
        &self.points
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

/// Represents the configuration for constructing or analyzing a curve. 
/// The `CurveConfig` structure encapsulates the necessary details required 
/// to define the type of curve, the interpolation method, construction methodology, 
/// and additional parameters associated with the curve.
///
/// # Fields
///
/// - `curve_type: CurveType`
///    Specifies the type of curve that the configuration applies to.
///    Curve types such as `Volatility`, `Delta`, `Gamma`, etc., are defined 
///    in the `CurveType` enumeration. Different curve types are typically 
///    used in mathematical modeling, financial analysis, or other specialized areas.
///
/// - `interpolation: InterpolationType`  
///    Defines the method of interpolation used for estimating values between 
///    discrete points on the curve. Supported interpolation methods include `Linear`, 
///    `Cubic`, `Spline`, and others, as specified in the `InterpolationType` enum.
///
/// - `construction_method: CurveConstructionMethod`  
///    Specifies how the curve is constructed. This could be based on discrete 
///    data points (`FromData`) or parametrically (`Parametric`), as defined in 
///    the `CurveConstructionMethod` enum. For instance:
///       - `FromData`: Build the curve from a collection of data points.
///       - `Parametric`: Construct the curve using a parametric function, 
///         defining the curve behavior over a range of input values (t_start to t_end) 
///         and the number of intermediate steps in computation.
///
/// - `extra_params: HashMap<String, Decimal>`  
///    Provides additional configuration parameters associated with the curve
///    as a key-value mapping. This field is particularly useful for passing optional
///    metadata or specialized model parameters required during analysis or construction.
///
/// # Example Use Cases
/// This configuration structure can be used in multiple scenarios:
///
/// 1. **Curve Construction:**  
///    A user can specify `curve_type` and `construction_method` to create a custom 
///    curve for financial modeling. The `extra_params` can include details such as
///    scaling factors or normalization parameters.
///
/// 2. **Analysis or Simulation:**  
///    When performing operations like interpolation, slicing, or analyzing
///    statistics of a curve, the `CurveConfig` can store relevant input parameters 
///    (e.g. interpolation type and additional processing rules via `extra_params`).
///
/// 3. **Visualization:**  
///    The configuration can also help define curves for rendering graphical data 
///    with specified interpolation styles, ensuring smoother and more realistic
///    representations of the modeled scenario.
///
/// # Integrations
/// This structure integrates with the following modules and traits:
///
/// - **Curves Module:** Used alongside `CurveType`, `CurveConstructionMethod`, 
///   and `InterpolationType` enums.
/// - **CurveOperations Trait:** Provides operations such as interpolation, 
///   scaling, and slicing that can utilize instances of `CurveConfig`.
/// - **Visualization Module:** Ensures flexibility in configuring graphs 
///   and curve representation when constructing plots of specific curve types.
pub struct CurveConfig {
    pub curve_type: CurveType,
    pub interpolation: InterpolationType,
    pub construction_method: CurveConstructionMethod,
    pub extra_params: HashMap<String, Decimal>,
}

#[cfg(test)]
mod tests_curves {
    use super::*;
    use crate::{pos, Positive};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    fn test_new_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    fn test_to_tuple_with_decimal() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Decimal, Decimal) = point.to_tuple().unwrap();
        assert_eq!(tuple, (dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_to_tuple_with_positive() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Positive, Positive) = point.to_tuple().unwrap();
        assert_eq!(tuple, (pos!(1.5), pos!(2.5)));
    }

    #[test]
    fn test_from_tuple_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_from_tuple_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_new_with_mixed_types() {
        let x = dec!(1.5);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }
}

#[cfg(test)]
mod tests_linear_interpolate {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_linear_interpolation_exact_points() {
        let curve = Curve::new(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]);

        // Test exact input points
        let p0 = curve
            .interpolate(Decimal::ZERO, InterpolationType::Linear)
            .unwrap();
        assert_eq!(p0.x, Decimal::ZERO);
        assert_eq!(p0.y, Decimal::ZERO);

        let p1 = curve
            .interpolate(Decimal::ONE, InterpolationType::Linear)
            .unwrap();
        assert_eq!(p1.x, Decimal::ONE);
        assert_eq!(p1.y, Decimal::TWO);
    }

    #[test]
    fn test_linear_interpolation_midpoint() {
        let curve = Curve::new(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]);

        // Test midpoint
        let mid = curve
            .interpolate(dec!(0.5), InterpolationType::Linear)
            .unwrap();
        assert_eq!(mid.x, dec!(0.5));
        assert_eq!(mid.y, dec!(1.0));
    }

    #[test]
    fn test_linear_interpolation_quarter_points() {
        let curve = Curve::new(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]);

        // Test at x = 0.25
        let p25 = curve
            .interpolate(dec!(0.25), InterpolationType::Linear)
            .unwrap();
        assert_eq!(p25.x, dec!(0.25));
        assert_eq!(p25.y, dec!(0.5));

        // Test at x = 0.75
        let p75 = curve
            .interpolate(dec!(0.75), InterpolationType::Linear)
            .unwrap();
        assert_eq!(p75.x, dec!(0.75));
        assert_eq!(p75.y, dec!(1.5));
    }

    #[test]
    fn test_linear_interpolation_out_of_range() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]);

        assert!(curve
            .interpolate(dec!(-0.1), InterpolationType::Linear)
            .is_err());
        assert!(curve
            .interpolate(dec!(1.1), InterpolationType::Linear)
            .is_err());
    }

    #[test]
    fn test_linear_interpolation_insufficient_points() {
        let curve = Curve::new(vec![Point2D::new(dec!(0.0), dec!(0.0))]);

        assert!(curve
            .interpolate(dec!(0.5), InterpolationType::Linear)
            .is_err());
    }

    #[test]
    fn test_linear_interpolation_non_monotonic() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(1.0)),
        ]);

        let p15 = curve
            .interpolate(dec!(1.5), InterpolationType::Linear)
            .unwrap();
        assert_eq!(p15.x, dec!(1.5));
        assert_eq!(p15.y, dec!(1.5));
    }
}

#[cfg(test)]
mod tests_bilinear_interpolate {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_bilinear_interpolation() {
        let curve = Curve::new(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
            Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
            Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
        ]);

        // Test exact points
        let corner = curve
            .interpolate(Decimal::ZERO, InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(corner.x, Decimal::ZERO);
        assert_eq!(corner.y, Decimal::ZERO);

        // Test midpoint interpolation
        let mid = curve
            .interpolate(dec!(0.5), InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(mid.x, dec!(0.5));
        assert_eq!(mid.y, dec!(1.0));
    }

    #[test]
    fn test_bilinear_interpolation_out_of_range() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]);

        assert!(curve
            .interpolate(dec!(-0.5), InterpolationType::Bilinear)
            .is_err());
        assert!(curve
            .interpolate(dec!(1.5), InterpolationType::Bilinear)
            .is_err());
    }

    #[test]
    fn test_bilinear_interpolation_insufficient_points() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(2.0)),
        ]);

        assert!(curve
            .interpolate(dec!(0.5), InterpolationType::Bilinear)
            .is_err());
    }

    #[test]
    fn test_bilinear_interpolation_quarter_points() {
        let curve = Curve::new(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11 (0,0)
            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12 (1,1)
            Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21 (0,1)
            Point2D::new(Decimal::ONE, Decimal::TWO),   // p22 (1,2)
        ]);

        // At x = 0.25:
        // Bottom edge: 0.25 * (1 - 0) = 0.25
        // Top edge: 0.25 * (2 - 1) + 1 = 1.25
        // Result: 0.25 + (1.25 - 0.25)/2 = 0.75
        let p25 = curve
            .interpolate(dec!(0.25), InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(p25.x, dec!(0.25));
        assert_eq!(p25.y, dec!(0.75));

        // At x = 0.75:
        // Bottom edge: 0.75 * (1 - 0) = 0.75
        // Top edge: 0.75 * (2 - 1) + 1 = 1.75
        // Result: 0.75 + (1.75 - 0.75)/2 = 1.25
        let p75 = curve
            .interpolate(dec!(0.75), InterpolationType::Bilinear)
            .unwrap();
        assert_eq!(p75.x, dec!(0.75));
        assert_eq!(p75.y, dec!(1.25));
    }

    #[test]
    fn test_bilinear_interpolation_boundaries() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]);

        assert!(curve
            .interpolate(dec!(-0.1), InterpolationType::Bilinear)
            .is_err());
        assert!(curve
            .interpolate(dec!(1.1), InterpolationType::Bilinear)
            .is_err());
    }

    #[test]
    fn test_out_of_range() {
        let curve = Curve::new(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::ONE),
            Point2D::new(Decimal::ZERO, Decimal::ONE),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]);

        assert!(curve
            .interpolate(dec!(-1), InterpolationType::Bilinear)
            .is_err());
        assert!(curve
            .interpolate(Decimal::TWO, InterpolationType::Bilinear)
            .is_err());
    }
}

#[cfg(test)]
mod tests_cubic_interpolate {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_cubic_interpolation_exact_points() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        // Test exact points
        let p1 = curve
            .interpolate(dec!(1.0), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(p1.x, dec!(1.0));
        assert_eq!(p1.y, dec!(1.0));
    }

    #[test]
    fn test_cubic_interpolation_midpoints() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        // Test midpoint interpolation
        let mid = curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid.x, dec!(1.5));
        // y value should be continuous and smooth
        assert!(mid.y > dec!(1.0) && mid.y < dec!(4.0));
    }

    #[test]
    fn test_cubic_interpolation_insufficient_points() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]);

        assert!(curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .is_err());
    }

    #[test]
    fn test_cubic_interpolation_out_of_range() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        assert!(curve
            .interpolate(dec!(-0.5), InterpolationType::Cubic)
            .is_err());
        assert!(curve
            .interpolate(dec!(3.5), InterpolationType::Cubic)
            .is_err());
    }

    #[test]
    fn test_cubic_interpolation_monotonicity() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        let p1 = curve
            .interpolate(dec!(0.5), InterpolationType::Cubic)
            .unwrap();
        let p2 = curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .unwrap();
        let p3 = curve
            .interpolate(dec!(2.5), InterpolationType::Cubic)
            .unwrap();

        assert!(p1.y < p2.y);
        assert!(p2.y < p3.y);

        // Print values for debugging
        println!("p1: {:?}, p2: {:?}, p3: {:?}", p1, p2, p3);
    }
}

#[cfg(test)]
mod tests_spline_interpolate {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_spline_interpolation_exact_points() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        let p1 = curve.interpolate(dec!(1.0), InterpolationType::Spline).unwrap();
        assert_eq!(p1.x, dec!(1.0));
        assert_eq!(p1.y, dec!(1.0));
    }

    #[test]
    fn test_spline_interpolation_midpoints() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        let mid = curve.interpolate(dec!(1.5), InterpolationType::Spline).unwrap();
        assert_eq!(mid.x, dec!(1.5));
        // Value should be continuous and between the points
        assert!(mid.y > dec!(1.0) && mid.y < dec!(4.0));
    }

    #[test]
    fn test_spline_interpolation_insufficient_points() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]);

        assert!(curve.interpolate(dec!(0.5), InterpolationType::Spline).is_err());
    }

    #[test]
    fn test_spline_interpolation_out_of_range() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]);

        assert!(curve.interpolate(dec!(-0.5), InterpolationType::Spline).is_err());
        assert!(curve.interpolate(dec!(2.5), InterpolationType::Spline).is_err());
    }

    #[test]
    fn test_spline_interpolation_smoothness() {
        let curve = Curve::new(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]);

        // Test points close together to verify smoothness
        let p1 = curve.interpolate(dec!(1.48), InterpolationType::Spline).unwrap();
        let p2 = curve.interpolate(dec!(1.49), InterpolationType::Spline).unwrap();
        let p3 = curve.interpolate(dec!(1.50), InterpolationType::Spline).unwrap();
        let p4 = curve.interpolate(dec!(1.51), InterpolationType::Spline).unwrap();
        let p5 = curve.interpolate(dec!(1.52), InterpolationType::Spline).unwrap();

        // Verify monotonicity and smooth transitions
        assert!(p1.y < p2.y);
        assert!(p2.y < p3.y);
        assert!(p3.y < p4.y);
        assert!(p4.y < p5.y);

        // Verify that the changes are smooth (second differences are small)
        let d1 = p2.y - p1.y;
        let d2 = p3.y - p2.y;
        let d3 = p4.y - p3.y;
        let d4 = p5.y - p4.y;

        // Second differences should be small
        assert!((d2 - d1).abs() < dec!(0.001));
        assert!((d3 - d2).abs() < dec!(0.001));
        assert!((d4 - d3).abs() < dec!(0.001));
    }
}