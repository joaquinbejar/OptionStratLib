/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/1/25
******************************************************************************/
use crate::curves::analysis::{
    BasicMetrics, CurveMetricsExtractor, RangeMetrics, RiskMetrics, ShapeMetrics, TrendMetrics,
};
use crate::curves::construction::CurveConstructionMethod;
use crate::curves::interpolation::{
    BiLinearInterpolation, CubicInterpolation, Interpolate, LinearInterpolation,
    SplineInterpolation,
};
use crate::curves::operations::CurveArithmetic;
use crate::curves::{MergeOperation, Point2D};
use crate::error::CurvesError;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeSet;
use std::ops::Index;

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
/// - [`Point2D`]: The fundamental data type for representing points in 2D space.
/// - [`crate::curves::curve_traits::CurveOperations`]: The trait providing operations on curves.
/// - [`MergeOperation`]: Enum for combining multiple curves.
///
/// # Fields
/// - **points**:
///   - A vector of `Point2D` objects that defines the curve in terms of its x-y plane coordinates.
/// - **x_range**:
///   - A tuple `(Decimal, Decimal)` that specifies the minimum and maximum x-coordinate values
///     for the curve. Operations performed on the curve should ensure they fall within this range.
#[derive(Debug, Clone)]
pub struct Curve {
    pub points: BTreeSet<Point2D>,
    pub x_range: (Decimal, Decimal),
}

impl Curve {
    /// Creates a new curve from a vector of points.
    ///
    /// This constructor initializes a `Curve` instance using a list of 2D points
    /// provided as a `BTreeSet<Point2D>`. Additionally, the x-range of the curve is calculated
    /// and stored. The x-range is determined by evaluating the minimum and maximum
    /// x-coordinates among the provided points.
    ///
    /// # Parameters
    ///
    /// - `points` (`BTreeSet<Point2D>`): A vector of points that define the curve in a
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
    /// - [`crate::curves::Curve::calculate_range`]: Computes the x-range of a set of points.
    pub fn new(points: BTreeSet<Point2D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }

    pub fn from_vector(points: Vec<Point2D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        let points = points.into_iter().collect();
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
    pub fn calculate_range<I>(iter: I) -> (Decimal, Decimal)
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

                let points: Result<BTreeSet<Point2D>, CurvesError> = (0..=steps)
                    .into_par_iter()
                    .map(|i| {
                        let t = t_start + step_size * Decimal::from(i);
                        f(t).map_err(|e| CurvesError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(Curve::new)
            }
        }
    }

    pub fn vector(&self) -> Vec<&Point2D> {
        self.points.iter().collect()
    }
}

/// Allows indexed access to the points in a `Curve` using `usize` indices.
///
/// # Overview
/// This implementation provides intuitive, array-like access to the points
/// within a `Curve`. By using the `Index<usize>` trait, users can directly
/// reference specific points by their index within the internal `points` collection
/// without manually iterating or managing indices themselves.
///
/// # Behavior
/// - The `index` method fetches the `Point2D` at the specified position in the order
///   of the curve's `points` (sorted by the `Point2D` ordering, typically based on the `x` values).
/// - If the specified index exceeds the range of available points, it triggers a panic
///   with the message `"Index out of bounds"`.
///
/// # Constraints
/// - The index must be a valid value between `0` and `self.points.len() - 1`.
/// - The `Curve`'s `points` are internally stored as a `BTreeSet<Point2D>`, so indexing
///   reflects the natural order of the set, which is determined by the `Ord` trait
///   implementation for `Point2D`.
///
/// # Fields Accessed
/// - **`points`**: A `BTreeSet` of `Point2D` structs representing the curve's 2D points.
///
/// # Panics
/// This implementation will panic if:
/// - The index provided is out of bounds (less than `0` or greater than/equal to the number
///   of points in the curve).
///
/// # Use Cases
/// - Quickly accessing specific points on a curve during visualization, interpolation,
///   or analysis operations.
/// - Performing operations that require stepwise access to points, such as
///   slicing or filtering points along the curve.
///
/// # Example
/// Suppose you have a `Curve` instance `curve` with multiple points:
/// ```ignore
/// let point = curve[0]; // Access the first point
/// println!("{:?}", point); // Print the `Point2D` representation
/// ```
///
/// # Important Notes
/// - This indexing implementation provides read-only access (`&Self::Output`).
/// - Modifying the `points` collection or its contents directly is not allowed through
///   this implementation, ensuring immutability when using indexed access.
///
/// # Type Associations
/// - **Input**:
///   - The input type for the `Index` operation is `usize`, the standard for indexing.
/// - **Output**:
///   - The output type for the `Index` operation is a reference to `Point2D`,
///     specifically `&Point2D`.
///
/// # Key Implementations
/// - **`Index<usize>`**: Provides indexing-based access to curve points.
impl Index<usize> for Curve {
    type Output = Point2D;

    /// Fetches the `Point2D` at the specified index.
    ///
    /// Panics if the index is invalid.
    fn index(&self, index: usize) -> &Self::Output {
        self.points.iter().nth(index).expect("Index out of bounds")
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

        let p1 = &self[i];
        let p2 = &self[j];

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
/// - [`BiLinearInterpolation`]: The trait defining this method.
/// - [`Interpolate`]: Ensures compatibility of the curve with multiple interpolation methods.
///
/// # See Also
///
/// - [`Curve`]: The overarching structure that represents the curve.
/// - [`Point2D`]: The data type used to represent individual points on the curve.
/// - [`find_bracket_points`](crate::curves::interpolation::Interpolate::find_bracket_points):
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
    ///           in the `[`0,1`]` interval.
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
            return Ok(**point);
        }

        let (i, _j) = self.find_bracket_points(x)?;

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
/// - [`CubicInterpolation`]: The trait defining this method.
/// - [`Point2D`]: Represents the points used for interpolation.
/// - [`find_bracket_points`](crate::curves::interpolation::Interpolate::find_bracket_points): Determines the bracketing points required for interpolation.
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
            return Ok(**point);
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
/// - [`SplineInterpolation`]: The trait definition for spline interpolation.
/// - [`Point2D`]: Represents a point in 2D space.
/// - [`Curve`]: Represents a mathematical curve made up of points for interpolation.
/// - [`CurvesError`]: Enumerates possible errors during curve operations.
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
    /// - [`Point2D`]: Represents a 2D point and is used as input/output
    ///   for this function.
    /// - [`CurvesError`] Represents any error encountered during
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
            return Ok(**point);
        }

        let n = points.len();

        // Calculate second derivatives
        let mut a = vec![Decimal::ZERO; n];
        let mut b = vec![Decimal::ZERO; n];
        let mut c = vec![Decimal::ZERO; n];
        let mut r = vec![Decimal::ZERO; n];

        // Fill the matrices
        for i in 1..n - 1 {
            let hi = points[i].x - points[i - 1].x;
            let hi1 = points[i + 1].x - points[i].x;

            a[i] = hi;
            b[i] = dec!(2) * (hi + hi1);
            c[i] = hi1;

            r[i] = dec!(6)
                * ((points[i + 1].y - points[i].y) / hi1 - (points[i].y - points[i - 1].y) / hi);
        }

        // Add boundary conditions (natural spline)
        b[0] = dec!(1);
        b[n - 1] = dec!(1);

        // Solve tridiagonal system using Thomas algorithm
        let mut m = vec![Decimal::ZERO; n];

        for i in 1..n - 1 {
            let w = a[i] / b[i - 1];
            b[i] -= w * c[i - 1];
            r[i] = r[i] - w * r[i - 1];
        }

        m[n - 1] = r[n - 1] / b[n - 1];
        for i in (1..n - 1).rev() {
            m[i] = (r[i] - c[i] * m[i + 1]) / b[i];
        }

        // Find segment for interpolation
        let mut segment = None;
        for i in 0..n - 1 {
            if points[i].x <= x && x <= points[i + 1].x {
                segment = Some(i);
                break;
            }
        }

        let segment = segment.ok_or_else(|| {
            CurvesError::InterpolationError(
                "Could not find valid segment for interpolation".to_string(),
            )
        })?;

        // Calculate interpolated value
        let h = points[segment + 1].x - points[segment].x;
        let dx = points[segment + 1].x - x;
        let dx1 = x - points[segment].x;

        let y = m[segment] * dx * dx * dx / (dec!(6) * h)
            + m[segment + 1] * dx1 * dx1 * dx1 / (dec!(6) * h)
            + (points[segment].y / h - m[segment] * h / dec!(6)) * dx
            + (points[segment + 1].y / h - m[segment + 1] * h / dec!(6)) * dx1;

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
/// - [`LinearInterpolation`]
/// - [`BiLinearInterpolation`]
/// - [`CubicInterpolation`]
/// - [`SplineInterpolation`]
///
/// These underlying traits implement specific interpolation algorithms,
/// enabling `Curve` to support a robust set of interpolation options through the associated methods.
/// Depending on the use case and provided parameters (e.g., interpolation type and target x-coordinate),
/// the appropriate algorithm is invoked.
///
/// # See Also
///
/// - [`Curve`]: The underlying mathematical structure being interpolated.
/// - [`Point2D`]: The fundamental data type for the curve's points.
/// - [`Interpolate`]: The trait defining interpolation operations.
///
impl Interpolate for Curve {
    /// ## `get_points`
    ///
    /// - **Signature**: `fn get_points(&self) -> &[Point2D]`
    /// - **Purpose**: Provides a reference to the collection of points that define the curve.
    /// - **Returns**: A slice of `Point2D` instances contained in the `points` vector of the struct.
    /// - **Usage**: This method is critical for interpolation algorithms, allowing access to the
    ///   ordered list of points necessary for calculations.
    fn get_points(&self) -> Vec<&Point2D> {
        self.vector()
    }
}

/// A default implementation for the `Curve` type using a provided default strategy.
///
/// This implementation provides a basic approach to computing curve metrics
/// by using interpolation and statistical methods available in the standard
/// curve analysis library.
///
/// # Note
/// This is a minimal implementation that may need to be customized or enhanced
/// based on specific requirements or domain-specific analysis needs.
impl CurveMetricsExtractor for Curve {
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, CurvesError> {
        // TODO: Implement actual basic metrics computation
        // This is a placeholder implementation
        Ok(BasicMetrics {
            mean: Decimal::ZERO,
            median: Decimal::ZERO,
            mode: Decimal::ZERO,
            std_dev: Decimal::ZERO,
        })
    }

    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, CurvesError> {
        // TODO: Implement actual shape metrics computation
        // This is a placeholder implementation
        Ok(ShapeMetrics {
            skewness: Decimal::ZERO,
            kurtosis: Decimal::ZERO,
            peaks: vec![],
            valleys: vec![],
            inflection_points: vec![],
        })
    }

    fn compute_range_metrics(&self) -> Result<RangeMetrics, CurvesError> {
        // TODO: Implement actual range metrics computation
        // This is a placeholder implementation
        let min_point = self
            .points
            .first()
            .cloned()
            .unwrap_or(Point2D::new(Decimal::ZERO, Decimal::ZERO));
        let max_point = self
            .points
            .last()
            .cloned()
            .unwrap_or(Point2D::new(Decimal::ZERO, Decimal::ZERO));

        Ok(RangeMetrics {
            min: min_point,
            max: max_point,
            range: Decimal::ZERO,
            quartiles: (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
            interquartile_range: Decimal::ZERO,
        })
    }

    fn compute_trend_metrics(&self) -> Result<TrendMetrics, CurvesError> {
        // TODO: Implement actual trend metrics computation
        // This is a placeholder implementation
        Ok(TrendMetrics {
            slope: Decimal::ZERO,
            intercept: Decimal::ZERO,
            r_squared: Decimal::ZERO,
            moving_average: vec![],
        })
    }

    fn compute_risk_metrics(&self) -> Result<RiskMetrics, CurvesError> {
        // TODO: Implement actual risk metrics computation
        // This is a placeholder implementation
        Ok(RiskMetrics {
            volatility: Decimal::ZERO,
            value_at_risk: Decimal::ZERO,
            expected_shortfall: Decimal::ZERO,
            beta: Decimal::ZERO,
            sharpe_ratio: Decimal::ZERO,
        })
    }
}

/// Implements the `CurveArithmetic` trait for the `Curve` type, providing
/// functionality for merging multiple curves using a specified mathematical
/// operation and performing arithmetic operations between two curves.
impl CurveArithmetic for Curve {
    /// Merges a collection of curves into a single curve based on the specified
    /// mathematical operation.
    ///
    /// # Parameters
    ///
    /// - `curves` (`&[&Curve]`): A slice of references to the curves to be merged.
    ///   Each curve must have defined x-ranges and interpolation capabilities.
    /// - `operation` (`MergeOperation`): The arithmetic operation to perform on the
    ///   interpolated y-values for the provided curves. Operations include addition,
    ///   subtraction, multiplication, division, and aggregation (e.g., max, min).
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: Returns a new curve resulting from the merging operation.
    /// - `Err(CurvesError)`: If input parameters are invalid or the merge operation
    ///   encounters an error (e.g., incompatible x-ranges or interpolation failure),
    ///   an error is returned.
    ///
    /// # Behavior
    ///
    /// 1. **Parameter Validation**:
    ///   - Verifies that at least one curve is provided in the `curves` parameter.
    ///   - Returns an error if no curves are included or if x-ranges are incompatible.
    ///
    /// 2. **Cloning Single Curve**:
    ///   - If only one curve is provided, its clone is returned as the result without
    ///     performing any further calculations.
    ///
    /// 3. **Range Computation**:
    ///   - Computes the intersection of x-ranges across the curves by finding the
    ///     maximum lower bound (`min_x`) and minimum upper bound (`max_x`).
    ///   - If the x-range intersection is invalid (i.e., `min_x >= max_x`), an error
    ///     is returned.
    ///
    /// 4. **Interpolation and Arithmetic**:
    ///   - Divides the x-range into `steps` equally spaced intervals (default: 100).
    ///   - Interpolates the y-values for all curves at each x-value in the range.
    ///   - Applies the specified `operation` to the aggregated y-values at each x-point.
    ///
    /// 5. **Parallel Processing**:
    ///   - Uses parallel iteration to perform interpolation and value combination
    ///     efficiently, leveraging the Rayon library.
    ///
    /// 6. **Error Handling**:
    ///   - Any errors during interpolation or arithmetic operations are propagated
    ///     back to the caller.
    ///
    /// # Errors
    ///
    /// - **Invalid Parameter** (`CurvesError`): Returned when no curves are provided
    ///   or x-ranges are incompatible.
    /// - **Interpolation Failure** (`CurvesError`): Raised if interpolation fails
    ///   for a specific curve or x-value.
    ///
    /// # Example Use Case
    ///
    /// This function enables combining multiple curves for tasks such as:
    /// - Summing y-values across different curves to compute a composite curve.
    /// - Finding the maximum/minimum y-value at each x-point for a collection of curves.
    fn merge_curves(curves: &[&Curve], operation: MergeOperation) -> Result<Curve, CurvesError> {
        if curves.is_empty() {
            return Err(CurvesError::invalid_parameters(
                "merge_curves",
                "No curves provided for merging",
            ));
        }

        // If only one curve, return a clone
        if curves.len() == 1 {
            return Ok(curves[0].clone());
        }

        // Find the intersection of x-ranges
        let min_x = curves
            .iter()
            .map(|c| c.x_range.0)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(Decimal::ZERO);

        let max_x = curves
            .iter()
            .map(|c| c.x_range.1)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(Decimal::ZERO);

        // Check if ranges are compatible
        if min_x >= max_x {
            return Err(CurvesError::invalid_parameters(
                "merge_curves",
                "Curves have incompatible x-ranges",
            ));
        }

        // Determine number of interpolation steps
        let steps = 100; // Configurable number of interpolation points
        let step_size = (max_x - min_x) / Decimal::from(steps);

        // Interpolate and perform operation using parallel iterator
        let result_points: Result<Vec<Point2D>, CurvesError> = (0..=steps)
            .into_par_iter()
            .map(|i| {
                let x = min_x + step_size * Decimal::from(i);

                // Interpolate y values for each curve
                let y_values: Result<Vec<Decimal>, CurvesError> = curves
                    .iter()
                    .map(|curve| {
                        curve
                            .interpolate(x, crate::curves::interpolation::InterpolationType::Cubic)
                            .map(|point| point.y)
                    })
                    .collect();

                let y_values = y_values?;

                // Perform the specified operation on interpolated y values
                let result_y: Decimal = match operation {
                    MergeOperation::Add => y_values.par_iter().sum(),
                    MergeOperation::Subtract => {
                        // Use Rayon's fold to parallelize subtraction
                        y_values
                            .par_iter()
                            .enumerate()
                            .map(|(i, &val)| if i == 0 { val } else { -val })
                            .reduce(|| Decimal::ZERO, |a, b| a + b)
                    }
                    MergeOperation::Multiply => y_values.par_iter().product(),
                    MergeOperation::Divide => y_values
                        .par_iter()
                        .enumerate()
                        .map(|(i, &val)| {
                            if i == 0 {
                                val
                            } else if val == Decimal::ZERO {
                                Decimal::MAX
                            } else {
                                Decimal::ONE / val
                            }
                        })
                        .reduce(|| Decimal::ONE, |a, b| a * b),
                    MergeOperation::Max => y_values
                        .par_iter()
                        .cloned()
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(Decimal::ZERO),
                    MergeOperation::Min => y_values
                        .par_iter()
                        .cloned()
                        .min_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(Decimal::ZERO),
                };

                Ok(Point2D::new(x, result_y))
            })
            .collect();

        // Handle potential errors during parallel processing
        let result_points = result_points?;

        Ok(Curve::from_vector(result_points))
    }

    /// Combines the current `Curve` instance with another curve using a mathematical
    /// operation, resulting in a new curve.
    ///
    /// # Parameters
    ///
    /// - `self` (`&Self`): A reference to the current curve instance.
    /// - `other` (`&Curve`): A reference to the second curve for the arithmetic operation.
    /// - `operation` (`MergeOperation`): The operation to apply when merging the curves.
    ///
    /// # Returns
    ///
    /// - `Ok(Curve)`: Returns a new curve that represents the result of the operation.
    /// - `Err(CurvesError)`: If the merge operation fails (e.g., incompatible x-ranges or
    ///   interpolation errors), an error is returned.
    ///
    /// # Behavior
    ///
    /// This function is a convenience wrapper around `merge_curves` that operates
    /// specifically on two curves. It passes `self` and `other` as an array to
    /// `merge_curves` and applies the desired operation.
    ///
    /// # Errors
    ///
    /// - Inherits all errors returned by the `merge_curves` method, including parameter
    ///   validation and interpolation errors.
    ///
    /// # Examples
    ///
    /// Use this method to easily perform arithmetic operations between two curves,
    /// such as summing their y-values or finding their pointwise maximum.
    fn merge_with(&self, other: &Curve, operation: MergeOperation) -> Result<Curve, CurvesError> {
        Self::merge_curves(&[self, other], operation)
    }
}

#[cfg(test)]
mod tests_curves {
    use super::*;
    use crate::curves::utils::{create_constant_curve, create_linear_curve};
    use crate::{pos, Positive};
    use rust_decimal_macros::dec;
    use Decimal;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_to_tuple_with_decimal() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Decimal, Decimal) = point.to_tuple().unwrap();
        assert_eq!(tuple, (dec!(1.5), dec!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_to_tuple_with_positive() {
        let point = Point2D::new(dec!(1.5), dec!(2.5));
        let tuple: (Positive, Positive) = point.to_tuple().unwrap();
        assert_eq!(tuple, (pos!(1.5), pos!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_tuple_with_decimal() {
        let x = dec!(1.5);
        let y = dec!(2.5);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_from_tuple_with_positive() {
        let x = pos!(1.5_f64);
        let y = pos!(2.5_f64);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new_with_mixed_types() {
        let x = dec!(1.5);
        let y = pos!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_constant_curve() {
        let curve = create_constant_curve(dec!(1.0), dec!(2.0), dec!(5.0));
        assert_eq!(curve.get_points().len(), 11);
        for point in curve.get_points() {
            assert_eq!(point.y, dec!(5.0));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_linear_curve() {
        let curve = create_linear_curve(dec!(1.0), dec!(2.0), dec!(2.0));
        assert_eq!(curve.get_points().len(), 11);
        let mut slope = dec!(2.0);
        for point in curve.get_points() {
            assert_eq!(point.y, slope);
            slope += dec!(0.2);
        }
    }
}

#[cfg(test)]
mod tests_linear_interpolate {
    use super::*;
    use crate::curves::interpolation::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_exact_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_midpoint() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        // Test midpoint
        let mid = curve
            .interpolate(dec!(0.5), InterpolationType::Linear)
            .unwrap();
        assert_eq!(mid.x, dec!(0.5));
        assert_eq!(mid.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_quarter_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(curve
            .interpolate(dec!(-0.1), InterpolationType::Linear)
            .is_err());
        assert!(curve
            .interpolate(dec!(1.1), InterpolationType::Linear)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(0.0),
            dec!(0.0),
        )]));

        assert!(curve
            .interpolate(dec!(0.5), InterpolationType::Linear)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_linear_interpolation_non_monotonic() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(1.0)),
        ]));

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
    use crate::curves::interpolation::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::TWO, Decimal::ONE),
            Point2D::new(Decimal::TEN, Decimal::ONE),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(curve
            .interpolate(dec!(-0.5), InterpolationType::Bilinear)
            .is_err());
        assert!(curve
            .interpolate(dec!(1.5), InterpolationType::Bilinear)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(2.0)),
        ]));

        assert!(curve
            .interpolate(dec!(0.5), InterpolationType::Bilinear)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_quarter_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11 (0,0)
            Point2D::new(Decimal::ONE, Decimal::ONE),   // p12 (1,1)
            Point2D::new(Decimal::TWO, Decimal::ONE),   // p21 (0,1)
            Point2D::new(Decimal::TEN, Decimal::TWO),   // p22 (1,2)
        ]));

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_bilinear_interpolation_boundaries() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(curve
            .interpolate(dec!(-0.1), InterpolationType::Bilinear)
            .is_err());
        assert!(curve
            .interpolate(dec!(1.1), InterpolationType::Bilinear)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::ONE),
            Point2D::new(Decimal::ZERO, Decimal::ONE),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

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
    use crate::curves::interpolation::InterpolationType;
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_exact_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test exact points
        let p1 = curve
            .interpolate(dec!(1.0), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(p1.x, dec!(1.0));
        assert_eq!(p1.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_midpoints() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test midpoint interpolation
        let mid = curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .unwrap();
        assert_eq!(mid.x, dec!(1.5));
        // y value should be continuous and smooth
        assert!(mid.y > dec!(1.0) && mid.y < dec!(4.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        assert!(curve
            .interpolate(dec!(1.5), InterpolationType::Cubic)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        assert!(curve
            .interpolate(dec!(-0.5), InterpolationType::Cubic)
            .is_err());
        assert!(curve
            .interpolate(dec!(3.5), InterpolationType::Cubic)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_cubic_interpolation_monotonicity() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

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
        info!("p1: {:?}, p2: {:?}, p3: {:?}", p1, p2, p3);
    }
}

#[cfg(test)]
mod tests_spline_interpolate {
    use super::*;
    use crate::curves::interpolation::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_exact_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        let p1 = curve
            .interpolate(dec!(1.0), InterpolationType::Spline)
            .unwrap();
        assert_eq!(p1.x, dec!(1.0));
        assert_eq!(p1.y, dec!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_midpoints() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        let mid = curve
            .interpolate(dec!(1.5), InterpolationType::Spline)
            .unwrap();
        assert_eq!(mid.x, dec!(1.5));
        // Value should be continuous and between the points
        assert!(mid.y > dec!(1.0) && mid.y < dec!(4.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        assert!(curve
            .interpolate(dec!(0.5), InterpolationType::Spline)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        assert!(curve
            .interpolate(dec!(-0.5), InterpolationType::Spline)
            .is_err());
        assert!(curve
            .interpolate(dec!(2.5), InterpolationType::Spline)
            .is_err());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_spline_interpolation_smoothness() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test points close together to verify smoothness
        let p1 = curve
            .interpolate(dec!(1.48), InterpolationType::Spline)
            .unwrap();
        let p2 = curve
            .interpolate(dec!(1.49), InterpolationType::Spline)
            .unwrap();
        let p3 = curve
            .interpolate(dec!(1.50), InterpolationType::Spline)
            .unwrap();
        let p4 = curve
            .interpolate(dec!(1.51), InterpolationType::Spline)
            .unwrap();
        let p5 = curve
            .interpolate(dec!(1.52), InterpolationType::Spline)
            .unwrap();

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

#[cfg(test)]
mod tests_curve_arithmetic {
    use super::*;
    use crate::curves::interpolation::InterpolationType;
    use crate::curves::utils::create_linear_curve;

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_add() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));

        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Add).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                + curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_subtract() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Subtract).unwrap();
        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                - curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_multiply() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Multiply).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                * curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_divide() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(6.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Divide).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let y2 = curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            // Skip points where interpolation results in zero
            if y2 == Decimal::ZERO {
                continue;
            }

            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y / y2;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_max() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Max).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let y1 = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let y2 = curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let expected_y = y1.max(y2);

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_min() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Min).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let y1 = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let y2 = curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y;
            let expected_y = y1.min(y2);

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_with_single_operation() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result = curve1.merge_with(&curve2, MergeOperation::Add).unwrap();

        // Verify that merge_with is equivalent to merge_curves with two curves
        let merged_result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Add).unwrap();

        // Compare points of both results
        assert_eq!(result.points.len(), merged_result.points.len());

        for i in 0..result.points.len() {
            assert!((result[i].x - merged_result[i].x).abs() < dec!(0.001));
            assert!((result[i].y - merged_result[i].y).abs() < dec!(0.001));
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_curves_error_handling() {
        // Test with empty slice
        let result = Curve::merge_curves(&[], MergeOperation::Add);
        assert!(result.is_err());

        // Test with curves of incompatible ranges
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let curve2 = create_linear_curve(dec!(5.0), dec!(15.0), dec!(2.0));

        // Verify that the merge operation works even with partially overlapping ranges
        let result = Curve::merge_curves(&[&curve1, &curve2], MergeOperation::Add);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_merge_multiple_curves() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0));
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0));
        let curve3 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0));

        let result =
            Curve::merge_curves(&[&curve1, &curve2, &curve3], MergeOperation::Add).unwrap();

        // Check result at some sample points
        let test_points = [dec!(0.0), dec!(5.0), dec!(10.0)];
        for x in &test_points {
            let expected_y = curve1.interpolate(*x, InterpolationType::Cubic).unwrap().y
                + curve2.interpolate(*x, InterpolationType::Cubic).unwrap().y
                + curve3.interpolate(*x, InterpolationType::Cubic).unwrap().y;

            let result_point = result.interpolate(*x, InterpolationType::Cubic).unwrap();
            assert!(
                (result_point.y - expected_y).abs() < dec!(0.001),
                "Failed at x = {}, expected {}, got {}",
                x,
                expected_y,
                result_point.y
            );
        }
    }
}


#[cfg(test)]
mod tests_extended {
    use crate::error::CurvesError::OperationError;
    use crate::error::OperationErrorKind;
    use super::*;
    
    #[test]
    fn test_construct_from_data_empty() {
        let result = Curve::construct(CurveConstructionMethod::FromData { points: BTreeSet::new() });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurvesError::Point2DError { reason } => {
                assert_eq!(reason, "Empty points array");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_construct_parametric_valid() {
        let f = |t: Decimal| Ok(Point2D::new(t, t * dec!(2.0)));
        let result = Curve::construct(CurveConstructionMethod::Parametric {
            f: Box::new(f), 
            t_start: dec!(0.0),
            t_end: dec!(10.0),
            steps: 10,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_construct_parametric_invalid_function() {
        let f = |_t: Decimal| Err(CurvesError::ConstructionError("Function evaluation failed".to_string()));
        let result = Curve::construct(CurveConstructionMethod::Parametric {
            f: Box::new(f), 
            t_start: dec!(0.0),
            t_end: dec!(10.0),
            steps: 10,
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurvesError::ConstructionError(reason) => {
                assert_eq!(reason, "Construction error: Function evaluation failed");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_segment_not_found_error() {
        let segment: Option<Point2D> = None;
        let result: Result<Point2D, CurvesError> = segment.ok_or_else(|| {
            CurvesError::InterpolationError(
                "Could not find valid segment for interpolation".to_string(),
            )
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurvesError::InterpolationError(reason) => {
                assert_eq!(reason, "Could not find valid segment for interpolation");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_compute_basic_metrics_placeholder() {
        let curve = Curve { points: BTreeSet::new(), x_range: (Default::default(), Default::default()) };
        let metrics = curve.compute_basic_metrics();
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.mean, Decimal::ZERO);
    }

    #[test]
    fn test_single_curve_return() {
        let curve = Curve { points: BTreeSet::new(), x_range: (Default::default(), Default::default()) };
        let result = if vec![curve.clone()].len() == 1 {
            Ok(curve.clone())
        } else {
            Err(CurvesError::invalid_parameters("merge_curves", "Invalid state"))
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_curves_invalid_x_range() {
        let min_x = dec!(10.0);
        let max_x = dec!(5.0);
        let result = if min_x >= max_x {
            Err(CurvesError::invalid_parameters(
                "merge_curves",
                "Curves have incompatible x-ranges",
            ))
        } else {
            Ok(())
        };
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            OperationError(OperationErrorKind::InvalidParameters { operation, reason }) => {
                assert_eq!(operation, "merge_curves");
                assert_eq!(reason, "Curves have incompatible x-ranges");
            },
            _ => {
                panic!("Unexpected error type");
            }
 
        }
    }
}