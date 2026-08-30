/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/1/25
******************************************************************************/
use crate::curves::Point2D;
use crate::curves::traits::StatisticalCurve;
use crate::curves::utils::detect_peaks_and_valleys;
use crate::error::decimal::DecimalError;
use crate::error::{CurveError, InterpolationError, MetricsError};
use crate::geometrics::{
    Arithmetic, AxisOperations, BasicMetrics, BiLinearInterpolation, ConstructionMethod,
    ConstructionParams, CubicInterpolation, GeometricObject, GeometricTransformations, Interpolate,
    InterpolationType, LinearInterpolation, MergeAxisInterpolate, MergeOperation, MetricsExtractor,
    RangeMetrics, RiskMetrics, ShapeMetrics, SplineInterpolation, TrendMetrics, powu_checked,
};
use crate::model::decimal::{d_add, d_div, d_mul, d_sub, d_sum_iter};
use crate::utils::Len;
use crate::visualization::{Graph, GraphData};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rayon::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use utoipa::ToSchema;

/// Represents a mathematical curve as a collection of 2D points.
///
/// # Overview
/// The `Curve` struct is a fundamental representation of a curve, defined as a series
/// of points in a two-dimensional Cartesian coordinate system. Each curve is associated
/// with an `x_range`, specifying the inclusive domain of the curve in terms of its x-coordinates.
///
/// This structure supports precise mathematical and computational operations, including
/// interpolation, analysis, transformations, and intersections. The use of `Decimal`
/// for coordinates ensures high-precision calculations, making it particularly suitable
/// for scientific, financial, or mathematical applications.
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
/// # Constraints
/// - All points in the `points` vector must lie within the specified `x_range`.
/// - Methods working with `Curve` data will assume that the `points` vector is ordered
///   by the `x`-coordinate. Non-ordered inputs may lead to undefined behavior in specific
///   operations.
/// - A curve is a function of its abscissa: at most one point per `x`. See
///   [`Curve::new`] for what breaking that rule does to each consumer, and
///   for why no constructor can enforce it.
///
/// # See Also
/// - [`Point2D`]: The fundamental data type for representing points in 2D space.
/// - [`MergeOperation`]: Enum for combining multiple curves.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Curve {
    /// A ordered set of `Point2D` objects that defines the curve in terms of its x-y plane coordinates.
    /// Points are stored in a `BTreeSet` which automatically maintains them in sorted order by their
    /// `(x, y)` pair. The set rejects an exact duplicate of that pair; it does not stop two ordinates
    /// from sharing an abscissa.
    ///
    /// Public, so a struct literal can populate it without passing through
    /// [`Curve::new`]. No constructor can therefore guarantee one point per
    /// abscissa.
    pub points: BTreeSet<Point2D>,

    /// A tuple `(min_x, max_x)` that specifies the minimum and maximum x-coordinate values
    /// for the curve. Operations performed on the curve should ensure they fall within this range.
    /// Both values are of type `Decimal` to ensure high precision in boundary calculations.
    pub x_range: (Decimal, Decimal),
}

impl Display for Curve {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for point in self.points.iter() {
            writeln!(f, "{point}")?;
        }
        Ok(())
    }
}

impl Default for Curve {
    fn default() -> Self {
        Curve {
            points: BTreeSet::new(),
            x_range: (Decimal::ZERO, Decimal::ZERO),
        }
    }
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
    /// # One point per abscissa
    ///
    /// A `Curve` is a function of its abscissa: at most one point per `x`.
    /// Every consumer reads it that way.
    /// [`get_point`](crate::geometrics::AxisOperations::get_point) and
    /// [`contains_point`](crate::geometrics::AxisOperations::contains_point)
    /// answer from the first point matching `x`,
    /// [`merge`](crate::geometrics::Arithmetic::merge) resamples every curve
    /// onto one shared x-grid with one `y` per `x`, and the interpolators in
    /// [`crate::geometrics`] bracket `x` between two consecutive points and
    /// divide by the width of that bracket.
    ///
    /// Nothing enforces the rule. [`Point2D`] orders on the full `(x, y)`
    /// pair, so `points` may hold several ordinates for one abscissa and
    /// `new` stores them all; a curve built that way is outside the contract
    /// and the consumers behave as follows:
    ///
    /// - all four interpolators return
    ///   [`InterpolationError::DegenerateInterval`] when asked about a
    ///   repeated abscissa, and pick no survivor: the bracket around it has
    ///   zero width and the slope across it is undefined;
    /// - `get_point` and `merge_axis_interpolate` read the first match in
    ///   `(x, y)` order, which is the lowest ordinate of the stack, and the
    ///   rest are invisible to the caller. Use
    ///   [`get_values`](crate::geometrics::AxisOperations::get_values) to
    ///   see every ordinate at an abscissa.
    ///
    /// Normalizing here would not help: `points` is a `pub` field, so a
    /// struct literal reaches the same state without going through any
    /// constructor. Rejecting or collapsing a duplicate can only ever be a
    /// convenience on this path, never an invariant of the type.
    ///
    /// A projection of a surface is genuinely multi-valued and is therefore
    /// not a curve: [`crate::surfaces::Surface::project_onto`] returns a
    /// `Vec<Point2D>`, and aggregating it to one ordinate per abscissa is
    /// the caller's job.
    #[must_use]
    pub fn new(points: BTreeSet<Point2D>) -> Self {
        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }

    /// Fetches the point at `index` without going through the panicking
    /// [`Index`] contract.
    ///
    /// `kind` selects the [`InterpolationError`] variant so each algorithm
    /// reports an out-of-bounds window under its own name.
    fn point_at(
        &self,
        index: usize,
        kind: fn(String) -> InterpolationError,
    ) -> Result<&Point2D, InterpolationError> {
        self.points.iter().nth(index).ok_or_else(|| {
            kind(format!(
                "point index {index} is out of bounds for a curve of {} points",
                self.points.len()
            ))
        })
    }

    /// Returns the sample sitting exactly at the abscissa `x`, if the curve
    /// has one there.
    ///
    /// A curve is a function of its abscissa (see [`Curve::new`]), so at most
    /// one point can sit at `x`. When several do, the curve has no value at
    /// `x` and no non-arbitrary way to choose among them exists, so this
    /// reports [`InterpolationError::DegenerateInterval`] instead of
    /// returning the first, which would be the lowest ordinate of the stack.
    /// [`AxisOperations::get_values`] reads all of them.
    fn exact_point_at(&self, x: Decimal) -> Result<Option<Point2D>, InterpolationError> {
        let mut at_x = self.points.iter().filter(|p| p.x == x);
        let Some(point) = at_x.next() else {
            return Ok(None);
        };
        if at_x.next().is_some() {
            return Err(InterpolationError::DegenerateInterval);
        }
        Ok(Some(*point))
    }
}

/// Wraps a checked-arithmetic failure raised while building or transforming a
/// curve.
fn construction_err(err: DecimalError) -> CurveError {
    CurveError::ConstructionError(err.to_string())
}

/// Wraps a checked-arithmetic failure raised while analysing a curve.
fn analysis_err(err: DecimalError) -> CurveError {
    CurveError::AnalysisError(err.to_string())
}

/// Wraps a checked-arithmetic failure in the interpolation variant of the
/// algorithm that raised it.
fn interp_err(
    kind: fn(String) -> InterpolationError,
) -> impl Fn(DecimalError) -> InterpolationError {
    move |err| kind(err.to_string())
}

/// Reads one sample of a sorted metric series, naming the statistic on the
/// out-of-bounds path instead of panicking.
fn sample_at(
    values: &[Decimal],
    index: usize,
    what: &'static str,
) -> Result<Decimal, MetricsError> {
    values.get(index).copied().ok_or_else(|| {
        MetricsError::BasicError(format!(
            "{what}: sample {index} is out of bounds for {} values",
            values.len()
        ))
    })
}

/// Arithmetic mean of a sample. Errors on an empty sample, where the mean is
/// undefined, and on a sum that leaves the representable range.
fn mean_of(values: &[Decimal], op: &'static str) -> Result<Decimal, DecimalError> {
    let sum = d_sum_iter(values.iter().copied(), op)?;
    d_div(sum, Decimal::from(values.len()), op)
}

/// Population variance of a sample about `mean`.
fn variance_of(
    values: &[Decimal],
    mean: Decimal,
    op: &'static str,
) -> Result<Decimal, DecimalError> {
    let mut acc = Decimal::ZERO;
    for &value in values {
        let centered = d_sub(value, mean, op)?;
        let squared = powu_checked(centered, 2, op)?;
        acc = d_add(acc, squared, op)?;
    }
    d_div(acc, Decimal::from(values.len()), op)
}

/// Mean of `(centered / std_dev)^order` over a pre-centered sample: the
/// standardized moment behind skewness (`order = 3`) and kurtosis
/// (`order = 4`).
fn standardized_moment(
    centered: &[Decimal],
    std_dev: Decimal,
    order: u64,
    op: &'static str,
) -> Result<Decimal, DecimalError> {
    let mut acc = Decimal::ZERO;
    for &value in centered {
        let z = d_div(value, std_dev, op)?;
        let moment = powu_checked(z, order, op)?;
        acc = d_add(acc, moment, op)?;
    }
    d_div(acc, Decimal::from(centered.len()), op)
}

/// Reads one slot of a spline working band, naming the band on the
/// out-of-bounds path instead of panicking.
fn band_at(
    band: &[Decimal],
    index: usize,
    name: &'static str,
) -> Result<Decimal, InterpolationError> {
    band.get(index).copied().ok_or_else(|| {
        InterpolationError::Spline(format!(
            "spline band `{name}` has no slot {index} (len {})",
            band.len()
        ))
    })
}

/// Mutable counterpart of [`band_at`].
fn band_at_mut<'a>(
    band: &'a mut [Decimal],
    index: usize,
    name: &'static str,
) -> Result<&'a mut Decimal, InterpolationError> {
    let len = band.len();
    band.get_mut(index).ok_or_else(|| {
        InterpolationError::Spline(format!(
            "spline band `{name}` has no slot {index} (len {len})"
        ))
    })
}

/// Evaluates `moment * d^3 / six_h`, the cubic half of a natural-spline
/// segment, one checked step at a time.
fn cube_scaled(
    moment: Decimal,
    d: Decimal,
    six_h: Decimal,
    op: &'static str,
) -> Result<Decimal, InterpolationError> {
    let scaled = d_mul(moment, d, op).map_err(interp_err(InterpolationError::Spline))?;
    let scaled = d_mul(scaled, d, op).map_err(interp_err(InterpolationError::Spline))?;
    let scaled = d_mul(scaled, d, op).map_err(interp_err(InterpolationError::Spline))?;
    d_div(scaled, six_h, op).map_err(interp_err(InterpolationError::Spline))
}

/// Evaluates `(y / h - moment * h / 6) * d`, the linear half of a
/// natural-spline segment, one checked step at a time.
fn linear_term(
    y: Decimal,
    moment: Decimal,
    h: Decimal,
    d: Decimal,
    op: &'static str,
) -> Result<Decimal, InterpolationError> {
    let level = d_div(y, h, op).map_err(interp_err(InterpolationError::Spline))?;
    let bend = d_mul(moment, h, op).map_err(interp_err(InterpolationError::Spline))?;
    let bend = d_div(bend, dec!(6), op).map_err(interp_err(InterpolationError::Spline))?;
    let slope = d_sub(level, bend, op).map_err(interp_err(InterpolationError::Spline))?;
    d_mul(slope, d, op).map_err(interp_err(InterpolationError::Spline))
}

impl Len for Curve {
    fn len(&self) -> usize {
        self.points.len()
    }

    fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

impl Graph for Curve {
    fn graph_data(&self) -> GraphData {
        self.clone().into()
    }
}

impl Graph for Vec<Curve> {
    fn graph_data(&self) -> GraphData {
        self.clone().into()
    }
}

impl GeometricObject<Point2D, Decimal> for Curve {
    type Error = CurveError;

    fn get_points(&self) -> BTreeSet<&Point2D> {
        self.points.iter().collect()
    }

    /// Builds a curve from a vector of convertible points.
    ///
    /// # One point per abscissa
    ///
    /// Same rule and same non-enforcement as [`Curve::new`]: a curve is a
    /// function of its abscissa, and this constructor does not check it.
    /// `points` is collected into a `BTreeSet`, which drops an exact
    /// duplicate of an `(x, y)` pair but keeps two ordinates that merely
    /// share an `x`. Aggregate before calling if the input can carry
    /// several ordinates per abscissa, as the projection returned by
    /// [`crate::surfaces::Surface::project_onto`] does.
    fn from_vector<T>(points: Vec<T>) -> Self
    where
        T: Into<Point2D> + Clone,
    {
        let points: BTreeSet<Point2D> = points.into_iter().map(|p| p.into()).collect();

        let x_range = Self::calculate_range(points.iter().map(|p| p.x));
        Curve { points, x_range }
    }

    fn construct<T>(method: T) -> Result<Self, Self::Error>
    where
        Self: Sized,
        T: Into<ConstructionMethod<Point2D, Decimal>>,
    {
        let method = method.into();
        match method {
            ConstructionMethod::FromData { points } => {
                if points.is_empty() {
                    return Err(CurveError::Point2DError {
                        reason: "Empty points array",
                    });
                }
                Ok(Curve::new(points))
            }
            ConstructionMethod::Parametric { f, params } => {
                let (t_start, t_end, steps) = match params {
                    ConstructionParams::D2 {
                        t_start,
                        t_end,
                        steps,
                    } => (t_start, t_end, steps),
                    _ => {
                        return Err(CurveError::ConstructionError(
                            "Invalid parameters".to_string(),
                        ));
                    }
                };
                if steps == 0 {
                    return Err(CurveError::ConstructionError(
                        "Parametric construction needs at least one step".to_string(),
                    ));
                }
                let op = "Curve::construct::step_size";
                let span = d_sub(t_end, t_start, op).map_err(construction_err)?;
                let step_size = d_div(span, Decimal::from(steps), op).map_err(construction_err)?;

                let points: Result<BTreeSet<Point2D>, CurveError> = (0..=steps)
                    .into_par_iter()
                    .map(|i| {
                        let offset = d_mul(step_size, Decimal::from(i), "Curve::construct::offset")
                            .map_err(construction_err)?;
                        let t = d_add(t_start, offset, "Curve::construct::t")
                            .map_err(construction_err)?;
                        f(t).map_err(|e| CurveError::ConstructionError(e.to_string()))
                    })
                    .collect();

                points.map(Curve::new)
            }
        }
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
    /// # Panics
    ///
    /// Panics if `index >= self.points.len()`. This matches the
    /// documented contract of [`std::ops::Index`].
    fn index(&self, index: usize) -> &Self::Output {
        // INVARIANT: `std::ops::Index` requires returning `&Self::Output`
        // by value, so there is no safe way to signal an out-of-bounds
        // index other than panicking. The contract mirrors `Vec<T>::index`
        // and is documented above.
        match self.points.iter().nth(index) {
            Some(p) => p,
            // scan-banned: allow -- `std::ops::Index` returns `&Self::Output`
            // and has no fallible channel; the contract mirrors `Vec::index`.
            // No library code reaches this arm: every internal lookup goes
            // through `Curve::point_at`.
            None => panic!(
                "Curve::index: out of bounds (index = {index}, len = {})",
                self.points.len()
            ),
        }
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
impl Interpolate<Point2D, Decimal> for Curve {}

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
/// - **Degenerate Bracket**: When several points share the requested
///   abscissa. The curve has no value there and the bracket around it has
///   zero width, so the method reports
///   `InterpolationError::DegenerateInterval` instead of picking one of the
///   ordinates stacked there.
///
/// # One point per abscissa
///
/// A curve is a function of its abscissa; see [`Curve::new`]. Asking for a
/// repeated abscissa returns `InterpolationError::DegenerateInterval`; no
/// ordinate of the stack is chosen.
///
/// Away from the stack the method still answers, reading the stack as a
/// vertical jump: an `x` strictly inside an interval bounded by a repeated
/// abscissa brackets against the point on its own side, the highest
/// ordinate when the stack bounds the interval on the left and the lowest
/// when it bounds it on the right.
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
impl LinearInterpolation<Point2D, Decimal> for Curve {
    /// # Method
    /// ### `linear_interpolate`
    ///
    /// Performs linear interpolation for a given `x` value by finding two consecutive
    /// points on the curve (`p1` and `p2`) that bracket the provided `x`. The `y` value
    /// is then calculated using the linear interpolation formula:
    fn linear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        // A sample sitting exactly at `x` is the answer, and a stack of
        // ordinates at `x` has no answer. Without this branch the first
        // bracket containing `x` ends at the lowest of the stack and the
        // formula reproduces its ordinate, which is a silent pick.
        if let Some(point) = self.exact_point_at(x)? {
            return Ok(point);
        }

        let (i, j) = self.find_bracket_points(x)?;

        let p1 = self.point_at(i, InterpolationError::Linear)?;
        let p2 = self.point_at(j, InterpolationError::Linear)?;

        // A curve is meant to be a function of its abscissa, but nothing
        // enforces it: a `BTreeSet<Point2D>` holds two points sharing an
        // abscissa, and the slope across a zero-width bracket is undefined
        // rather than infinite. Report it instead of picking an ordinate.
        let run = d_sub(p2.x, p1.x, "Curve::linear_interpolate::run")
            .map_err(interp_err(InterpolationError::Linear))?;
        if run.is_zero() {
            return Err(InterpolationError::DegenerateInterval);
        }

        // Linear interpolation for y value
        let dx = d_sub(x, p1.x, "Curve::linear_interpolate::dx")
            .map_err(interp_err(InterpolationError::Linear))?;
        let rise = d_sub(p2.y, p1.y, "Curve::linear_interpolate::rise")
            .map_err(interp_err(InterpolationError::Linear))?;
        let scaled = d_mul(dx, rise, "Curve::linear_interpolate::scaled")
            .map_err(interp_err(InterpolationError::Linear))?;
        let ratio = d_div(scaled, run, "Curve::linear_interpolate::ratio")
            .map_err(interp_err(InterpolationError::Linear))?;
        let y = d_add(p1.y, ratio, "Curve::linear_interpolate::y")
            .map_err(interp_err(InterpolationError::Linear))?;

        Ok(Point2D::new(x, y))
    }
}

/// Implementation of the `BiLinearInterpolation` trait for the `Curve` struct.
///
/// # What it computes
///
/// Bilinear interpolation is defined on a cell of a two-dimensional grid: the
/// answer is the mean of four corner samples, weighted by the query's
/// fractional position along each axis. A curve has one axis, so the cell is
/// built out of two of its segments and the second fraction is fixed at one
/// half:
///
/// - the *near edge* is the segment that brackets `x`, from sample `i` to
///   sample `i + 1`;
/// - the *far edge* is the segment two positions further along, from sample
///   `f` to sample `f + 1`;
/// - the answer is the mean of the two edges read at the same fraction.
///
/// ```text
/// t    = (x - x[i]) / (x[i+1] - x[i])
/// near = y[i] + t * (y[i+1] - y[i])
/// far  = y[f] + t * (y[f+1] - y[f])
/// y    = (near + far) / 2
/// ```
///
/// # Behaviour near the upper boundary
///
/// The far edge is `f = i + 2` while such a segment exists. Over the last two
/// segments of the curve it does not, and `f` is clamped to `len - 2`, the
/// curve's last segment. Two consequences a caller can act on:
///
/// - on the final segment the two edges coincide, `near` equals `far`, and
///   the answer is the linear interpolant there. Not always to the last
///   digit: this method normalises `x` before scaling by the rise and
///   [`LinearInterpolation::linear_interpolate`] scales first, so where the
///   fraction along the segment has no finite decimal expansion the two
///   round at scale 28 on different quantities and can part company in the
///   28th place;
/// - on the second-to-last segment the far edge is the segment immediately
///   after the near one instead of the one after that.
///
/// # Why not clamp the window start
///
/// The obvious repair, and the one
/// [`CubicInterpolation::cubic_interpolate`] applies at its own boundary, is
/// to slide the whole window back to the last four samples. Do not reach for
/// it here. The window start is also the denominator of `t`, so moving it
/// back off the segment holding `x` pushes `t` past `1`. Written out, the
/// answer weights the four ordinates it reads by
///
/// ```text
/// y = (1 - t)/2 * y[i] + t/2 * y[i+1] + (1 - t)/2 * y[f] + t/2 * y[f+1]
/// ```
///
/// which sums to one for any `t`, but has every weight non-negative only
/// while `t` is in `[0, 1]`. Past `1` the two `(1 - t)/2` weights go
/// negative, the answer stops being a convex combination of the cell's
/// corners, and what is left is an extrapolation of a cell that does not
/// contain `x`: on `(0,0), (1,1), (2,4), (3,9)` the window-start clamp
/// returns `9.5` at `x = 2.5`, above every ordinate the curve has. Being a
/// convex combination of its cell's corners is what makes bilinear
/// interpolation bilinear interpolation, so it is the near edge that has to
/// stay put and the far edge that gives way. That is why the two boundary
/// rules differ.
///
/// Before this rule existed the last two segments returned
/// [`InterpolationError::Bilinear`] because the window ran off the end.
///
/// # What this is not
///
/// The far edge sits elsewhere on the curve, so the result does not follow
/// the samples between knots and this is not an interpolant through them: on
/// `(0,0), (1,1), (2,4), (3,9)` the value at `x = 0.5` is `3.5`, not the
/// `0.5` a straight reading of the bracketing segment gives.
///
/// An `x` landing exactly on a sample returns that sample, which neither
/// one-sided limit approaching it matches, so the result jumps at every
/// sample. That jump is as old as the method and owes nothing to the
/// boundary rule above: on `(0,0), (1,1), (2,4), (3,9), (4,16)` both limits
/// at `x = 1` are `5` against a sample of `1`, and both segments meeting
/// there are far from the clamp. What the clamp does add is that at the last
/// two samples before the end of the curve the two limits no longer agree
/// with each other either, because the segments on either side get different
/// far edges.
///
/// Reach for [`LinearInterpolation`] or [`SplineInterpolation`] when a curve
/// through the samples is what is wanted.
///
/// # Parameters
///
/// - **`x`**: The x-coordinate to interpolate at. Must lie within the range
///   of the curve's x-coordinates.
///
/// # Returns
///
/// - **`Ok(Point2D)`**: The interpolated point at `x`, both coordinates as
///   `Decimal`.
/// - **`Err(InterpolationError)`**: See the error list below.
///
/// # Errors
///
/// - [`InterpolationError::Bilinear`]: the curve holds fewer than four
///   samples, or a checked-arithmetic step left the representable `Decimal`
///   range.
/// - [`InterpolationError::OutOfRange`]: `x` falls outside the curve's
///   x-range.
/// - [`InterpolationError::DegenerateInterval`]: several samples share the
///   abscissa `x`, so the curve has no value there.
///
/// # One point per abscissa
///
/// A curve is a function of its abscissa; see [`Curve::new`]. Asking for a
/// repeated abscissa returns [`InterpolationError::DegenerateInterval`], as
/// it does for the three sibling algorithms; no ordinate of the stack is
/// chosen. A stack elsewhere on the curve does not stop this method
/// answering: only the near edge's width is divided by, and a stack sitting
/// on the far edge is read for its two ordinates alone.
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
/// - [`find_bracket_points`](crate::geometrics::Interpolate::find_bracket_points):
///   A helper method used to locate the two points that bracket the given x-coordinate.
impl BiLinearInterpolation<Point2D, Decimal> for Curve {
    /// Reads the curve at `x` as the mean of the bracketing segment and the
    /// segment two positions further along, both at the same fraction.
    ///
    /// The far segment is clamped to the curve's last one where it would run
    /// off the end, which makes the final segment a plain linear
    /// interpolation. The impl-level documentation states the rule and its
    /// consequences in full.
    ///
    /// # Errors
    ///
    /// - [`InterpolationError::Bilinear`] when the curve holds fewer than
    ///   four samples, or when a checked-arithmetic step leaves the
    ///   representable `Decimal` range.
    /// - [`InterpolationError::OutOfRange`] when `x` is outside the curve's
    ///   x-range.
    /// - [`InterpolationError::DegenerateInterval`] when several samples
    ///   share the abscissa `x`.
    fn bilinear_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let len = self.len();

        // The cell is four samples wide: two segments, one edge each.
        if len < 4 {
            return Err(InterpolationError::Bilinear(
                "Need at least four points for bilinear interpolation".to_string(),
            ));
        }

        // For exact points, return the actual point value. A stack of
        // ordinates at `x` has no single value, so it errors out instead of
        // yielding the lowest of them, which is what the three sibling
        // algorithms do.
        if let Some(point) = self.exact_point_at(x)? {
            return Ok(point);
        }

        let (i, _j) = self.find_bracket_points(x)?;

        // The near edge is the segment bracketing `x`, so `dx` below stays in
        // `[0, 1]`. The far edge is the segment two positions on;
        // `find_bracket_points` only guarantees `i + 1` exists, so over the
        // last two segments that one runs past the end and is clamped to the
        // curve's last segment, `len - 2`. The four-sample check above keeps
        // that subtraction in range.
        let far = (i + 2).min(len - 2);

        let p11 = self.point_at(i, InterpolationError::Bilinear)?; // Near edge, left
        let p12 = self.point_at(i + 1, InterpolationError::Bilinear)?; // Near edge, right
        let p21 = self.point_at(far, InterpolationError::Bilinear)?; // Far edge, left
        let p22 = self.point_at(far + 1, InterpolationError::Bilinear)?; // Far edge, right

        let span = d_sub(p12.x, p11.x, "Curve::bilinear_interpolate::span")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        if span.is_zero() {
            return Err(InterpolationError::DegenerateInterval);
        }

        // Normalize x to [0,1] interval
        let offset = d_sub(x, p11.x, "Curve::bilinear_interpolate::offset")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let dx = d_div(offset, span, "Curve::bilinear_interpolate::dx")
            .map_err(interp_err(InterpolationError::Bilinear))?;

        // Interpolate along the near edge
        let bottom_rise = d_sub(p12.y, p11.y, "Curve::bilinear_interpolate::bottom_rise")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let bottom_step = d_mul(dx, bottom_rise, "Curve::bilinear_interpolate::bottom_step")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let bottom = d_add(p11.y, bottom_step, "Curve::bilinear_interpolate::bottom")
            .map_err(interp_err(InterpolationError::Bilinear))?;

        // Interpolate along the far edge, at the same fraction
        let top_rise = d_sub(p22.y, p21.y, "Curve::bilinear_interpolate::top_rise")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let top_step = d_mul(dx, top_rise, "Curve::bilinear_interpolate::top_step")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let top = d_add(p21.y, top_step, "Curve::bilinear_interpolate::top")
            .map_err(interp_err(InterpolationError::Bilinear))?;

        // Mean of the two edges
        let edge_gap = d_sub(top, bottom, "Curve::bilinear_interpolate::edge_gap")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let half_gap = d_div(edge_gap, dec!(2), "Curve::bilinear_interpolate::half_gap")
            .map_err(interp_err(InterpolationError::Bilinear))?;
        let y = d_add(bottom, half_gap, "Curve::bilinear_interpolate::y")
            .map_err(interp_err(InterpolationError::Bilinear))?;

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
/// - [`find_bracket_points`](crate::geometrics::Interpolate::find_bracket_points): Determines the bracketing points required for interpolation.
impl CubicInterpolation<Point2D, Decimal> for Curve {
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
    /// - If the curve is not a function of its abscissa at the point it is
    ///   asked about, it returns `InterpolationError::DegenerateInterval`:
    ///   several points at `x` leave the exact-match branch with no single
    ///   value to return, and two consecutive points sharing an abscissa
    ///   leave the interpolation bracket with zero width. Neither case picks
    ///   an ordinate. See [`Curve::new`] for the rule.
    ///
    /// # Example
    ///
    /// - Interpolating smoothly along a curve defined by a set of points, avoiding sharp
    ///   transitions between segments.
    ///
    /// - Provides a high degree of precision due to the use of the `Decimal` type for
    ///   `x` and `y` calculations.
    fn cubic_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let len = self.len();

        // Need at least 4 points for cubic interpolation
        if len < 4 {
            return Err(InterpolationError::Cubic(
                "Need at least four points for cubic interpolation".to_string(),
            ));
        }

        // For exact points, return the actual point value. A stack of
        // ordinates at `x` has no single value, so it errors out instead of
        // yielding the lowest of them.
        if let Some(point) = self.exact_point_at(x)? {
            return Ok(point);
        }

        let (i, _) = self.find_bracket_points(x)?;

        // Select four points for interpolation
        // Ensuring we always have enough points before and after
        let window = if i == 0 {
            [0, 1, 2, 3]
        } else if i == len - 2 {
            [len - 4, len - 3, len - 2, len - 1]
        } else {
            [i - 1, i, i + 1, i + 2]
        };
        let p0 = self.point_at(window[0], InterpolationError::Cubic)?;
        let p1 = self.point_at(window[1], InterpolationError::Cubic)?;
        let p2 = self.point_at(window[2], InterpolationError::Cubic)?;
        let p3 = self.point_at(window[3], InterpolationError::Cubic)?;

        let span = d_sub(p2.x, p1.x, "Curve::cubic_interpolate::span")
            .map_err(interp_err(InterpolationError::Cubic))?;
        if span.is_zero() {
            return Err(InterpolationError::DegenerateInterval);
        }

        // Calculate t parameter (normalized x position between p1 and p2)
        let offset = d_sub(x, p1.x, "Curve::cubic_interpolate::offset")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let t = d_div(offset, span, "Curve::cubic_interpolate::t")
            .map_err(interp_err(InterpolationError::Cubic))?;

        // Cubic interpolation using Catmull-Rom spline
        let t2 = d_mul(t, t, "Curve::cubic_interpolate::t2")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let t3 = d_mul(t2, t, "Curve::cubic_interpolate::t3")
            .map_err(interp_err(InterpolationError::Cubic))?;

        let term0 = d_mul(dec!(2), p1.y, "Curve::cubic_interpolate::term0")
            .map_err(interp_err(InterpolationError::Cubic))?;

        let linear_coeff = d_sub(p2.y, p0.y, "Curve::cubic_interpolate::linear_coeff")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let term1 = d_mul(linear_coeff, t, "Curve::cubic_interpolate::term1")
            .map_err(interp_err(InterpolationError::Cubic))?;

        let two_p0 = d_mul(dec!(2), p0.y, "Curve::cubic_interpolate::two_p0")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let five_p1 = d_mul(dec!(5), p1.y, "Curve::cubic_interpolate::five_p1")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let four_p2 = d_mul(dec!(4), p2.y, "Curve::cubic_interpolate::four_p2")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let quad_coeff = d_sub(two_p0, five_p1, "Curve::cubic_interpolate::quad_coeff_a")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let quad_coeff = d_add(
            quad_coeff,
            four_p2,
            "Curve::cubic_interpolate::quad_coeff_b",
        )
        .map_err(interp_err(InterpolationError::Cubic))?;
        let quad_coeff = d_sub(quad_coeff, p3.y, "Curve::cubic_interpolate::quad_coeff_c")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let term2 = d_mul(quad_coeff, t2, "Curve::cubic_interpolate::term2")
            .map_err(interp_err(InterpolationError::Cubic))?;

        let three_p1 = d_mul(dec!(3), p1.y, "Curve::cubic_interpolate::three_p1")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let three_p2 = d_mul(dec!(3), p2.y, "Curve::cubic_interpolate::three_p2")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let cubic_coeff = d_sub(three_p1, p0.y, "Curve::cubic_interpolate::cubic_coeff_a")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let cubic_coeff = d_sub(
            cubic_coeff,
            three_p2,
            "Curve::cubic_interpolate::cubic_coeff_b",
        )
        .map_err(interp_err(InterpolationError::Cubic))?;
        let cubic_coeff = d_add(cubic_coeff, p3.y, "Curve::cubic_interpolate::cubic_coeff_c")
            .map_err(interp_err(InterpolationError::Cubic))?;
        let term3 = d_mul(cubic_coeff, t3, "Curve::cubic_interpolate::term3")
            .map_err(interp_err(InterpolationError::Cubic))?;

        let sum = d_sum_iter(
            [term0, term1, term2, term3],
            "Curve::cubic_interpolate::sum",
        )
        .map_err(interp_err(InterpolationError::Cubic))?;
        let y = d_mul(dec!(0.5), sum, "Curve::cubic_interpolate::y")
            .map_err(interp_err(InterpolationError::Cubic))?;

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
/// - [`CurveError`]: Enumerates possible errors during curve operations.
impl SplineInterpolation<Point2D, Decimal> for Curve {
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
    /// - [`CurveError`] Represents any error encountered during
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
    /// - The knots must be a function of their abscissa. Several points at
    ///   the requested `x` leave the exact-match branch with no single value
    ///   to return, and any repeated abscissa collapses a knot interval, so
    ///   both return `InterpolationError::DegenerateInterval` rather than
    ///   picking an ordinate. See [`Curve::new`] for the rule.
    fn spline_interpolate(&self, x: Decimal) -> Result<Point2D, InterpolationError> {
        let len = self.len();

        // Need at least 3 points for spline interpolation
        if len < 3 {
            return Err(InterpolationError::Spline(
                "Need at least three points for spline interpolation".to_string(),
            ));
        }

        // Check if x is within the valid range
        let first = self.point_at(0, InterpolationError::Spline)?;
        let last = self.point_at(len - 1, InterpolationError::Spline)?;
        if x < first.x || x > last.x {
            return Err(InterpolationError::Spline(
                "x is outside the range of points".to_string(),
            ));
        }

        // For exact points, return the actual point value. A stack of
        // ordinates at `x` has no single value, so it errors out instead of
        // yielding the lowest of them.
        if let Some(point) = self.exact_point_at(x)? {
            return Ok(point);
        }

        let n = len;
        let pts: Vec<&Point2D> = self.points.iter().collect();
        let at = |index: usize| -> Result<&Point2D, InterpolationError> {
            pts.get(index).copied().ok_or_else(|| {
                InterpolationError::Spline(format!(
                    "point index {index} is out of bounds for a curve of {n} points"
                ))
            })
        };

        // Calculate second derivatives. The three interior bands and the
        // right-hand side are built in order, with the natural-spline
        // boundary rows (`b[0] = b[n-1] = 1`) pushed at the ends.
        let mut a = vec![Decimal::ZERO];
        let mut b = vec![Decimal::ONE];
        let mut c = vec![Decimal::ZERO];
        let mut r = vec![Decimal::ZERO];

        // Fill the matrices
        for i in 1..n - 1 {
            let prev = at(i - 1)?;
            let curr = at(i)?;
            let next = at(i + 1)?;

            let hi = d_sub(curr.x, prev.x, "Curve::spline_interpolate::hi")
                .map_err(interp_err(InterpolationError::Spline))?;
            let hi1 = d_sub(next.x, curr.x, "Curve::spline_interpolate::hi1")
                .map_err(interp_err(InterpolationError::Spline))?;
            // A repeated abscissa collapses a knot interval; the second
            // derivative there is undefined rather than infinite.
            if hi.is_zero() || hi1.is_zero() {
                return Err(InterpolationError::DegenerateInterval);
            }

            let band = d_add(hi, hi1, "Curve::spline_interpolate::band")
                .map_err(interp_err(InterpolationError::Spline))?;
            let diag = d_mul(dec!(2), band, "Curve::spline_interpolate::diag")
                .map_err(interp_err(InterpolationError::Spline))?;

            let rise_next = d_sub(next.y, curr.y, "Curve::spline_interpolate::rise_next")
                .map_err(interp_err(InterpolationError::Spline))?;
            let slope_next = d_div(rise_next, hi1, "Curve::spline_interpolate::slope_next")
                .map_err(interp_err(InterpolationError::Spline))?;
            let rise_prev = d_sub(curr.y, prev.y, "Curve::spline_interpolate::rise_prev")
                .map_err(interp_err(InterpolationError::Spline))?;
            let slope_prev = d_div(rise_prev, hi, "Curve::spline_interpolate::slope_prev")
                .map_err(interp_err(InterpolationError::Spline))?;
            let curvature = d_sub(
                slope_next,
                slope_prev,
                "Curve::spline_interpolate::curvature",
            )
            .map_err(interp_err(InterpolationError::Spline))?;
            let rhs = d_mul(dec!(6), curvature, "Curve::spline_interpolate::rhs")
                .map_err(interp_err(InterpolationError::Spline))?;

            a.push(hi);
            b.push(diag);
            c.push(hi1);
            r.push(rhs);
        }

        // Add boundary conditions (natural spline)
        a.push(Decimal::ZERO);
        b.push(Decimal::ONE);
        c.push(Decimal::ZERO);
        r.push(Decimal::ZERO);

        // Solve tridiagonal system using Thomas algorithm
        let mut m = vec![Decimal::ZERO; n];

        for i in 1..n - 1 {
            let a_i = band_at(&a, i, "a")?;
            let b_prev = band_at(&b, i - 1, "b")?;
            if b_prev.is_zero() {
                return Err(InterpolationError::DegenerateInterval);
            }
            let w = d_div(a_i, b_prev, "Curve::spline_interpolate::w")
                .map_err(interp_err(InterpolationError::Spline))?;

            let c_prev = band_at(&c, i - 1, "c")?;
            let wc = d_mul(w, c_prev, "Curve::spline_interpolate::wc")
                .map_err(interp_err(InterpolationError::Spline))?;
            let b_i = band_at(&b, i, "b")?;
            *band_at_mut(&mut b, i, "b")? = d_sub(b_i, wc, "Curve::spline_interpolate::b_i")
                .map_err(interp_err(InterpolationError::Spline))?;

            let r_prev = band_at(&r, i - 1, "r")?;
            let wr = d_mul(w, r_prev, "Curve::spline_interpolate::wr")
                .map_err(interp_err(InterpolationError::Spline))?;
            let r_i = band_at(&r, i, "r")?;
            *band_at_mut(&mut r, i, "r")? = d_sub(r_i, wr, "Curve::spline_interpolate::r_i")
                .map_err(interp_err(InterpolationError::Spline))?;
        }

        let b_last = band_at(&b, n - 1, "b")?;
        if b_last.is_zero() {
            return Err(InterpolationError::DegenerateInterval);
        }
        let r_last = band_at(&r, n - 1, "r")?;
        *band_at_mut(&mut m, n - 1, "m")? =
            d_div(r_last, b_last, "Curve::spline_interpolate::m_last")
                .map_err(interp_err(InterpolationError::Spline))?;
        for i in (1..n - 1).rev() {
            let c_i = band_at(&c, i, "c")?;
            let m_next = band_at(&m, i + 1, "m")?;
            let cm = d_mul(c_i, m_next, "Curve::spline_interpolate::cm")
                .map_err(interp_err(InterpolationError::Spline))?;
            let r_i = band_at(&r, i, "r")?;
            let numerator = d_sub(r_i, cm, "Curve::spline_interpolate::m_numerator")
                .map_err(interp_err(InterpolationError::Spline))?;
            let b_i = band_at(&b, i, "b")?;
            if b_i.is_zero() {
                return Err(InterpolationError::DegenerateInterval);
            }
            *band_at_mut(&mut m, i, "m")? = d_div(numerator, b_i, "Curve::spline_interpolate::m_i")
                .map_err(interp_err(InterpolationError::Spline))?;
        }

        // Find segment for interpolation
        let mut segment = None;
        for i in 0..n - 1 {
            if at(i)?.x <= x && x <= at(i + 1)?.x {
                segment = Some(i);
                break;
            }
        }

        let segment = segment.ok_or_else(|| {
            InterpolationError::Spline("Could not find valid segment for interpolation".to_string())
        })?;

        // Calculate interpolated value
        let left = at(segment)?;
        let right = at(segment + 1)?;
        let h = d_sub(right.x, left.x, "Curve::spline_interpolate::h")
            .map_err(interp_err(InterpolationError::Spline))?;
        if h.is_zero() {
            return Err(InterpolationError::DegenerateInterval);
        }
        let dx = d_sub(right.x, x, "Curve::spline_interpolate::dx")
            .map_err(interp_err(InterpolationError::Spline))?;
        let dx1 = d_sub(x, left.x, "Curve::spline_interpolate::dx1")
            .map_err(interp_err(InterpolationError::Spline))?;

        let m_left = band_at(&m, segment, "m")?;
        let m_right = band_at(&m, segment + 1, "m")?;
        let six_h = d_mul(dec!(6), h, "Curve::spline_interpolate::six_h")
            .map_err(interp_err(InterpolationError::Spline))?;

        let left_cube = cube_scaled(m_left, dx, six_h, "Curve::spline_interpolate::left_cube")?;
        let right_cube = cube_scaled(m_right, dx1, six_h, "Curve::spline_interpolate::right_cube")?;
        let left_linear = linear_term(left.y, m_left, h, dx, "Curve::spline_interpolate::left")?;
        let right_linear =
            linear_term(right.y, m_right, h, dx1, "Curve::spline_interpolate::right")?;

        let y = d_sum_iter(
            [left_cube, right_cube, left_linear, right_linear],
            "Curve::spline_interpolate::y",
        )
        .map_err(interp_err(InterpolationError::Spline))?;

        Ok(Point2D::new(x, y))
    }
}

impl StatisticalCurve for Curve {
    fn get_x_values(&self) -> Vec<Decimal> {
        self.points.iter().map(|p| p.x).collect()
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
impl MetricsExtractor for Curve {
    fn compute_basic_metrics(&self) -> Result<BasicMetrics, MetricsError> {
        let y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();

        // Handle empty curve
        if y_values.is_empty() {
            return Ok(BasicMetrics {
                mean: Decimal::ZERO,
                median: Decimal::ZERO,
                mode: Decimal::ZERO,
                std_dev: Decimal::ZERO,
            });
        }

        // Mean
        let mean = mean_of(&y_values, "Curve::compute_basic_metrics::mean")
            .map_err(|e| MetricsError::BasicError(e.to_string()))?;

        // Median
        let mut sorted_values = y_values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = sorted_values.len() / 2;
        let median = if sorted_values.len().is_multiple_of(2) {
            let below = mid.checked_sub(1).ok_or_else(|| {
                MetricsError::BasicError(
                    "median: empty sample after the emptiness guard".to_string(),
                )
            })?;
            let lower = sample_at(&sorted_values, below, "median")?;
            let upper = sample_at(&sorted_values, mid, "median")?;
            let pair = d_add(lower, upper, "Curve::compute_basic_metrics::median_pair")
                .map_err(|e| MetricsError::BasicError(e.to_string()))?;
            d_div(pair, Decimal::TWO, "Curve::compute_basic_metrics::median")
                .map_err(|e| MetricsError::BasicError(e.to_string()))?
        } else {
            sample_at(&sorted_values, mid, "median")?
        };

        // Mode (most frequent value)
        let mode = {
            let mut freq_map = std::collections::HashMap::new();
            for &val in &y_values {
                *freq_map.entry(val).or_insert(0) += 1;
            }
            freq_map
                .into_iter()
                .max_by_key(|&(_, count)| count)
                .map(|(val, _)| val)
                .unwrap_or(Decimal::ZERO)
        };

        // Standard Deviation
        let variance = variance_of(&y_values, mean, "Curve::compute_basic_metrics::variance")
            .map_err(|e| MetricsError::BasicError(e.to_string()))?;
        let std_dev = variance.sqrt().unwrap_or(Decimal::ZERO);

        Ok(BasicMetrics {
            mean,
            median,
            mode,
            std_dev,
        })
    }

    fn compute_shape_metrics(&self) -> Result<ShapeMetrics, MetricsError> {
        let y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();

        // Handle empty or single-point curve
        if y_values.len() < 2 {
            return Ok(ShapeMetrics {
                skewness: Decimal::ZERO,
                kurtosis: Decimal::ZERO,
                peaks: vec![],
                valleys: vec![],
                inflection_points: vec![],
            });
        }

        // Mean and Standard Deviation
        let mean = mean_of(&y_values, "Curve::compute_shape_metrics::mean")
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;

        // Compute centered and scaled values
        let centered_values: Vec<Decimal> = y_values
            .iter()
            .map(|&x| d_sub(x, mean, "Curve::compute_shape_metrics::centered"))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;

        // Compute variance
        let variance = variance_of(&y_values, mean, "Curve::compute_shape_metrics::variance")
            .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let std_dev = variance.sqrt().unwrap_or(Decimal::ONE);
        if std_dev.is_zero() || std_dev < dec!(1e-9) {
            return Err(MetricsError::ShapeError(format!(
                "standard deviation ({std_dev}) is too small to compute skewness/kurtosis; the curve is degenerate"
            )));
        }

        // Skewness calculation (Fisher-Pearson standardized moment)
        let skewness = standardized_moment(
            &centered_values,
            std_dev,
            3,
            "Curve::compute_shape_metrics::skewness",
        )
        .map_err(|e| MetricsError::ShapeError(e.to_string()))?;

        // Kurtosis calculation (Fisher's definition - adjust to excess kurtosis)
        let raw_kurtosis = standardized_moment(
            &centered_values,
            std_dev,
            4,
            "Curve::compute_shape_metrics::kurtosis",
        )
        .map_err(|e| MetricsError::ShapeError(e.to_string()))?;
        let kurtosis = d_sub(
            raw_kurtosis,
            Decimal::from(3),
            "Curve::compute_shape_metrics::excess_kurtosis",
        )
        .map_err(|e| MetricsError::ShapeError(e.to_string()))?;

        // Peaks and Valleys detection
        let (peaks, valleys) = detect_peaks_and_valleys(&self.points, dec!(0.1), 2);

        Ok(ShapeMetrics {
            skewness,
            kurtosis,
            peaks,
            valleys,
            inflection_points: vec![],
        })
    }

    fn compute_range_metrics(&self) -> Result<RangeMetrics, MetricsError> {
        // Handle empty curve
        if self.points.is_empty() {
            return Ok(RangeMetrics {
                min: Point2D::new(Decimal::ZERO, Decimal::ZERO),
                max: Point2D::new(Decimal::ZERO, Decimal::ZERO),
                range: Decimal::ZERO,
                quartiles: (Decimal::ZERO, Decimal::ZERO, Decimal::ZERO),
                interquartile_range: Decimal::ZERO,
            });
        }

        let mut y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();
        y_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let len = y_values.len();
        let min_point = self
            .points
            .iter()
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| MetricsError::BasicError("empty curve in min_point".to_string()))?;
        let max_point = self
            .points
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| MetricsError::BasicError("empty curve in max_point".to_string()))?;

        let range = d_sub(
            max_point.y,
            min_point.y,
            "Curve::compute_range_metrics::range",
        )
        .map_err(|e| MetricsError::RangeError(e.to_string()))?;

        // Quartiles
        let q1 = sample_at(&y_values, len / 4, "first quartile")?;
        let q2 = sample_at(&y_values, len / 2, "median")?;
        let q3 = sample_at(&y_values, 3 * len / 4, "third quartile")?;

        let interquartile_range = d_sub(q3, q1, "Curve::compute_range_metrics::iqr")
            .map_err(|e| MetricsError::RangeError(e.to_string()))?;

        Ok(RangeMetrics {
            min: min_point,
            max: max_point,
            range,
            quartiles: (q1, q2, q3),
            interquartile_range,
        })
    }

    fn compute_trend_metrics(&self) -> Result<TrendMetrics, MetricsError> {
        let points: Vec<Point2D> = self.points.clone().into_iter().collect();

        // Handle insufficient points
        if points.len() < 2 {
            return Ok(TrendMetrics {
                slope: Decimal::ZERO,
                intercept: Decimal::ZERO,
                r_squared: Decimal::ZERO,
                moving_average: vec![],
            });
        }

        // Linear Regression Calculation
        let n = Decimal::from(points.len());
        let x_vals: Vec<Decimal> = points.iter().map(|p| p.x).collect();
        let y_vals: Vec<Decimal> = points.iter().map(|p| p.y).collect();

        let trend = || -> Result<(Decimal, Decimal, Decimal), DecimalError> {
            let op = "Curve::compute_trend_metrics";
            let sum_x = d_sum_iter(x_vals.iter().copied(), op)?;
            let sum_y = d_sum_iter(y_vals.iter().copied(), op)?;
            let mut sum_xy = Decimal::ZERO;
            let mut sum_xx = Decimal::ZERO;
            for (x, y) in x_vals.iter().zip(&y_vals) {
                sum_xy = d_add(sum_xy, d_mul(*x, *y, op)?, op)?;
                sum_xx = d_add(sum_xx, d_mul(*x, *x, op)?, op)?;
            }

            let numerator = d_sub(d_mul(n, sum_xy, op)?, d_mul(sum_x, sum_y, op)?, op)?;
            let denominator = d_sub(d_mul(n, sum_xx, op)?, d_mul(sum_x, sum_x, op)?, op)?;
            let slope = d_div(numerator, denominator, op)?;
            let intercept = d_div(d_sub(sum_y, d_mul(slope, sum_x, op)?, op)?, n, op)?;

            // R-squared Calculation
            let mean_y = d_div(sum_y, n, op)?;
            let mut sst = Decimal::ZERO;
            for y in &y_vals {
                let centered = d_sub(*y, mean_y, op)?;
                sst = d_add(sst, powu_checked(centered, 2, op)?, op)?;
            }

            let mut ssr = Decimal::ZERO;
            for (y, x) in y_vals.iter().zip(&x_vals) {
                let y_predicted = d_add(d_mul(slope, *x, op)?, intercept, op)?;
                let residual = d_sub(*y, y_predicted, op)?;
                ssr = d_add(ssr, powu_checked(residual, 2, op)?, op)?;
            }

            let r_squared = if sst == Decimal::ZERO {
                Decimal::ONE
            } else {
                d_sub(Decimal::ONE, d_div(ssr, sst, op)?, op)?
            };

            Ok((slope, intercept, r_squared))
        };

        // A sample whose abscissas all collapse to one value (or whose squares
        // underflow the `Decimal` scale) leaves the ordinary-least-squares
        // denominator at zero, where the slope is undefined rather than
        // infinite.
        let (slope, intercept, r_squared) =
            trend().map_err(|e| MetricsError::TrendError(e.to_string()))?;

        // Moving Average Calculation
        let window_sizes = [3, 5, 7];
        let mut moving_average: Vec<Point2D> = Vec::new();
        for window in window_sizes {
            if window > points.len() {
                continue;
            }
            for window_points in points.windows(window) {
                let xs: Vec<Decimal> = window_points.iter().map(|p| p.x).collect();
                let ys: Vec<Decimal> = window_points.iter().map(|p| p.y).collect();
                let avg_x = mean_of(&xs, "Curve::compute_trend_metrics::moving_average_x")
                    .map_err(|e| MetricsError::TrendError(e.to_string()))?;
                let avg_y = mean_of(&ys, "Curve::compute_trend_metrics::moving_average_y")
                    .map_err(|e| MetricsError::TrendError(e.to_string()))?;
                moving_average.push(Point2D::new(avg_x, avg_y));
            }
        }

        Ok(TrendMetrics {
            slope,
            intercept,
            r_squared,
            moving_average,
        })
    }

    fn compute_risk_metrics(&self) -> Result<RiskMetrics, MetricsError> {
        let y_values: Vec<Decimal> = self.points.iter().map(|p| p.y).collect();

        if y_values.is_empty() {
            return Ok(RiskMetrics {
                volatility: Decimal::ZERO,
                value_at_risk: Decimal::ZERO,
                expected_shortfall: Decimal::ZERO,
                beta: Decimal::ZERO,
                sharpe_ratio: Decimal::ZERO,
            });
        }

        let op = "Curve::compute_risk_metrics";
        let mean = mean_of(&y_values, op).map_err(|e| MetricsError::RiskError(e.to_string()))?;
        // Note the grouping: the sum of squared deviations is divided by
        // `sqrt(n)`, not by `sqrt(sum / n)`. Preserved as-is; only the
        // arithmetic is made checked.
        let mut squared_deviations = Decimal::ZERO;
        for &value in &y_values {
            let centered =
                d_sub(value, mean, op).map_err(|e| MetricsError::RiskError(e.to_string()))?;
            let squared = powu_checked(centered, 2, op)
                .map_err(|e| MetricsError::RiskError(e.to_string()))?;
            squared_deviations = d_add(squared_deviations, squared, op)
                .map_err(|e| MetricsError::RiskError(e.to_string()))?;
        }
        let sqrt_n = Decimal::from(y_values.len())
            .sqrt()
            .unwrap_or(Decimal::ZERO);
        // `d_div` rejects the zero denominator, which the emptiness guard
        // above already rules out.
        let volatility = d_div(squared_deviations, sqrt_n, op)
            .map_err(|e| MetricsError::RiskError(e.to_string()))?;

        // Value at Risk (95% confidence) using parametric method. At zero
        // dispersion this is `mean - 1.645 * 0 = mean`, a deterministic level
        // rather than an absence of value, so it is computed from the formula
        // on every path instead of being short-circuited to zero.
        let z_score = dec!(1.645);
        let scaled_vol =
            d_mul(z_score, volatility, op).map_err(|e| MetricsError::RiskError(e.to_string()))?;
        let var =
            d_sub(mean, scaled_vol, op).map_err(|e| MetricsError::RiskError(e.to_string()))?;

        let tail: Vec<Decimal> = y_values.iter().copied().filter(|&x| x < var).collect();
        let expected_shortfall = if tail.is_empty() {
            Decimal::ZERO
        } else {
            mean_of(&tail, op).map_err(|e| MetricsError::RiskError(e.to_string()))?
        };

        // `volatility / mean` is already zero at zero dispersion, so this
        // guard only covers the undefined `x / 0`.
        let beta = if mean != Decimal::ZERO {
            d_div(volatility, mean, op).map_err(|e| MetricsError::RiskError(e.to_string()))?
        } else {
            Decimal::ZERO
        };

        // Sharpe Ratio (assuming risk-free rate of 0). A flat curve has no
        // dispersion to divide by, which makes this the one field that is
        // genuinely undefined at `volatility == 0`; the others keep their
        // deterministic limits.
        let sharpe_ratio = if volatility.is_zero() {
            Decimal::ZERO
        } else {
            d_div(mean, volatility, op).map_err(|e| MetricsError::RiskError(e.to_string()))?
        };

        Ok(RiskMetrics {
            volatility,
            value_at_risk: var,
            expected_shortfall,
            beta,
            sharpe_ratio,
        })
    }
}

/// Implements the `CurveArithmetic` trait for the `Curve` type, providing
/// functionality for merging multiple curves using a specified mathematical
/// operation and performing arithmetic operations between two curves.
impl Arithmetic<Curve> for Curve {
    type Error = CurveError;

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
    ///
    /// # One point per abscissa
    ///
    /// This assumes the one-point-per-abscissa rule of [`Curve::new`] and
    /// re-establishes it in the result: the merged curve is sampled on an
    /// evenly spaced grid over the common x-range, with one `y` per grid
    /// point, so the original abscissae do not survive the merge.
    ///
    /// An input carrying several ordinates at one abscissa never has one of
    /// them silently chosen. Each sample is produced by cubic interpolation,
    /// which reports [`InterpolationError::DegenerateInterval`], wrapped in
    /// a [`CurveError`], whenever the sample lands on the repeated abscissa
    /// or brackets across it. A sample far enough from the stack is
    /// unaffected and reads the ordinate on its own side, so a merge of such
    /// a curve can still succeed on a grid that misses the stack. Aggregate
    /// the curve to one ordinate per abscissa before merging it.
    fn merge(curves: &[&Curve], operation: MergeOperation) -> Result<Curve, CurveError> {
        if curves.is_empty() {
            return Err(CurveError::invalid_parameters(
                "merge_curves",
                "No curves provided for merging",
            ));
        }

        // If only one curve, return a clone
        if let [only] = curves {
            return Ok((*only).clone());
        }

        // Find the intersection of x-ranges
        let min_x = curves
            .iter()
            .map(|c| c.x_range.0)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(Decimal::ZERO);

        let max_x = curves
            .iter()
            .map(|c| c.x_range.1)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(Decimal::ZERO);

        // Check if ranges are compatible
        if min_x >= max_x {
            return Err(CurveError::invalid_parameters(
                "merge_curves",
                "Curves have incompatible x-ranges",
            ));
        }

        // Determine number of interpolation steps
        let steps = 100; // Configurable number of interpolation points
        let op = "Curve::merge";
        let span = d_sub(max_x, min_x, op).map_err(construction_err)?;
        let step_size = d_div(span, Decimal::from(steps), op).map_err(construction_err)?;

        // Interpolate and perform operation using parallel iterator
        let result_points: Result<Vec<Point2D>, CurveError> = (0..=steps)
            .into_par_iter()
            .map(|i| {
                let offset = d_mul(step_size, Decimal::from(i), op).map_err(construction_err)?;
                let x = d_add(min_x, offset, op).map_err(construction_err)?;

                // Interpolate y values for each curve
                let y_values: Result<Vec<Decimal>, CurveError> = curves
                    .iter()
                    .map(|curve| {
                        curve
                            .interpolate(x, InterpolationType::Cubic)
                            .map(|point| point.y)
                            .map_err(CurveError::from)
                    })
                    .collect();

                let y_values = y_values?;

                // Perform the specified operation on interpolated y values
                let result_y: Decimal = match operation {
                    MergeOperation::Add => {
                        d_sum_iter(y_values.iter().copied(), op).map_err(construction_err)?
                    }
                    MergeOperation::Subtract => {
                        let signed = y_values
                            .iter()
                            .enumerate()
                            .map(|(i, &val)| if i == 0 { val } else { -val });
                        d_sum_iter(signed, op).map_err(construction_err)?
                    }
                    MergeOperation::Multiply => y_values.par_iter().copied().map(Ok).reduce(
                        || Ok(Decimal::ONE),
                        |a, b| d_mul(a?, b?, op).map_err(construction_err),
                    )?,
                    MergeOperation::Divide => y_values
                        .par_iter()
                        .enumerate()
                        .map(|(i, &val)| {
                            // `Decimal::MAX` is the pre-existing sentinel for a
                            // zero divisor; only the arithmetic is made checked.
                            if i == 0 {
                                Ok(val)
                            } else if val == Decimal::ZERO {
                                Ok(Decimal::MAX)
                            } else {
                                d_div(Decimal::ONE, val, op).map_err(construction_err)
                            }
                        })
                        .reduce(
                            || Ok(Decimal::ONE),
                            |a, b| d_mul(a?, b?, op).map_err(construction_err),
                        )?,
                    MergeOperation::Max => y_values
                        .par_iter()
                        .cloned()
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                        .unwrap_or(Decimal::ZERO),
                    MergeOperation::Min => y_values
                        .par_iter()
                        .cloned()
                        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
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
    fn merge_with(&self, other: &Curve, operation: MergeOperation) -> Result<Curve, CurveError> {
        Self::merge(&[self, other], operation)
    }
}

impl AxisOperations<Point2D, Decimal> for Curve {
    type Error = CurveError;

    /// Reports whether the curve has a point at the abscissa `x`.
    ///
    /// Answers on `x` alone, so it says nothing about how many ordinates sit
    /// there. On a curve honouring the one-point-per-abscissa rule of
    /// [`Curve::new`] there is at most one.
    fn contains_point(&self, x: &Decimal) -> bool {
        self.points.iter().any(|p| &p.x == x)
    }

    /// Returns the abscissa of every point, in `(x, y)` order.
    ///
    /// A curve holding several ordinates at one abscissa yields that
    /// abscissa once per ordinate; the duplicates are visible here rather
    /// than dropped.
    fn get_index_values(&self) -> Vec<Decimal> {
        self.points.iter().map(|p| p.x).collect()
    }

    /// Returns every ordinate at the abscissa `x`, in ascending order.
    ///
    /// This is the multi-valued reader: unlike [`Self::get_point`], it never
    /// picks a survivor. A curve honouring the one-point-per-abscissa rule
    /// of [`Curve::new`] returns at most one value; anything longer says the
    /// curve is not a function of its abscissa.
    fn get_values(&self, x: Decimal) -> Vec<&Decimal> {
        self.points
            .iter()
            .filter(|p| p.x == x)
            .map(|p| &p.y)
            .collect()
    }

    fn get_closest_point(&self, x: &Decimal) -> Result<&Point2D, Self::Error> {
        // The distance is folded explicitly rather than computed inside a
        // comparator: `a.x - x` overflows for abscissas at opposite ends of
        // the `Decimal` range, and `min_by` has no channel for that.
        let mut closest: Option<(&Point2D, Decimal)> = None;
        for point in &self.points {
            let distance = d_sub(point.x, *x, "Curve::get_closest_point")
                .map_err(analysis_err)?
                .abs();
            // `min_by` keeps the first of several equal minima; `<=` here
            // preserves that.
            match closest {
                Some((_, best)) if best <= distance => {}
                _ => closest = Some((point, distance)),
            }
        }

        closest
            .map(|(point, _)| point)
            .ok_or(CurveError::Point2DError {
                reason: "No points available",
            })
    }

    /// Returns the point at the abscissa `x`, if there is one.
    ///
    /// Assumes the one-point-per-abscissa rule of [`Curve::new`]. If the
    /// curve breaks it, this returns the *first* match in `(x, y)` order,
    /// which is the lowest ordinate of the stack, and the rest are invisible
    /// to the caller. Use [`Self::get_values`] to see all of them.
    fn get_point(&self, x: &Decimal) -> Option<&Point2D> {
        if self.contains_point(x) {
            self.points.iter().find(|p| p.x == *x)
        } else {
            None
        }
    }
}

impl MergeAxisInterpolate<Point2D, Decimal> for Curve
where
    Self: Sized,
{
    /// Resamples both curves onto the shared x-grid of their merged indices.
    ///
    /// Each result holds one point per abscissa of that grid, so it is a
    /// function of its abscissa whatever the inputs were: where a curve
    /// already has a point at an abscissa, the one kept is the first match
    /// in `(x, y)` order, so a stack of ordinates collapses to its lowest,
    /// and elsewhere the point is interpolated. Aggregate such a curve
    /// before merging if a different ordinate is wanted.
    fn merge_axis_interpolate(
        &self,
        other: &Self,
        interpolation: InterpolationType,
    ) -> Result<(Self, Self), Self::Error> {
        // Get merged unique x-coordinates
        let merged_x_values = self.merge_axis_index(other);

        // Sort the merged x values
        let mut sorted_x_values: Vec<Decimal> = merged_x_values.into_iter().collect();
        sorted_x_values.sort();

        let mut interpolated_self_points = BTreeSet::new();
        let mut interpolated_other_points = BTreeSet::new();

        for x in &sorted_x_values {
            if self.contains_point(x) {
                let pt = self.get_point(x).ok_or_else(|| {
                    CurveError::InterpolationError(format!(
                        "missing self point at x={x} despite contains_point()"
                    ))
                })?;
                interpolated_self_points.insert(*pt);
            } else {
                let interpolated_point = self.interpolate(*x, interpolation)?;
                interpolated_self_points.insert(interpolated_point);
            }
            if other.contains_point(x) {
                let pt = other.get_point(x).ok_or_else(|| {
                    CurveError::InterpolationError(format!(
                        "missing other point at x={x} despite contains_point()"
                    ))
                })?;
                interpolated_other_points.insert(*pt);
            } else {
                let interpolated_point = other.interpolate(*x, interpolation)?;
                interpolated_other_points.insert(interpolated_point);
            }
        }
        Ok((
            Curve::new(interpolated_self_points),
            Curve::new(interpolated_other_points),
        ))
    }
}

impl GeometricTransformations<Point2D> for Curve {
    type Error = CurveError;

    fn translate(&self, deltas: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if deltas.len() != 2 {
            return Err(CurveError::invalid_parameters(
                "translate",
                "Expected 2 deltas for 2D translation",
            ));
        }

        let (Some(dx), Some(dy)) = (deltas.first(), deltas.get(1)) else {
            return Err(CurveError::invalid_parameters(
                "translate",
                "Expected 2 deltas for 2D translation",
            ));
        };

        let translated_points = self
            .points
            .iter()
            .map(|point| {
                let x = d_add(point.x, **dx, "Curve::translate::x")?;
                let y = d_add(point.y, **dy, "Curve::translate::y")?;
                Ok(Point2D::new(x, y))
            })
            .collect::<Result<BTreeSet<Point2D>, DecimalError>>()
            .map_err(construction_err)?;

        Ok(Curve::new(translated_points))
    }

    fn scale(&self, factors: Vec<&Decimal>) -> Result<Self, Self::Error> {
        if factors.len() != 2 {
            return Err(CurveError::invalid_parameters(
                "scale",
                "Expected 2 factors for 2D scaling",
            ));
        }

        let (Some(fx), Some(fy)) = (factors.first(), factors.get(1)) else {
            return Err(CurveError::invalid_parameters(
                "scale",
                "Expected 2 factors for 2D scaling",
            ));
        };

        let scaled_points = self
            .points
            .iter()
            .map(|point| {
                let x = d_mul(point.x, **fx, "Curve::scale::x")?;
                let y = d_mul(point.y, **fy, "Curve::scale::y")?;
                Ok(Point2D::new(x, y))
            })
            .collect::<Result<BTreeSet<Point2D>, DecimalError>>()
            .map_err(construction_err)?;

        Ok(Curve::new(scaled_points))
    }

    fn intersect_with(&self, other: &Self) -> Result<Vec<Point2D>, Self::Error> {
        let mut intersections = Vec::new();
        let epsilon = Decimal::new(1, 6);

        // Use existing pairs iterator for efficiency
        for p1 in self.get_points() {
            for p2 in other.get_points() {
                // Find points with small distance between them
                let dx = d_sub(p1.x, p2.x, "Curve::intersect_with::dx")
                    .map_err(analysis_err)?
                    .abs();
                if dx >= epsilon {
                    continue;
                }
                let dy = d_sub(p1.y, p2.y, "Curve::intersect_with::dy")
                    .map_err(analysis_err)?
                    .abs();
                if dy < epsilon {
                    intersections.push(*p1);
                }
            }
        }

        Ok(intersections)
    }

    fn derivative_at(&self, point: &Point2D) -> Result<Vec<Decimal>, Self::Error> {
        let (i, j) = self.find_bracket_points(point.x)?;

        let p0 = self.point_at(i, InterpolationError::Linear)?;
        let p1 = self.point_at(j, InterpolationError::Linear)?;

        let op = "Curve::derivative_at";
        let rise = d_sub(p1.y, p0.y, op).map_err(analysis_err)?;
        let x1_sq = d_mul(p1.x, p1.x, op).map_err(analysis_err)?;
        let x0_sq = d_mul(p0.x, p0.x, op).map_err(analysis_err)?;
        let run = d_sub(x1_sq, x0_sq, op).map_err(analysis_err)?;
        if run.is_zero() {
            // The parabola fitted through the bracket is degenerate when the
            // two abscissas have the same square, so its coefficient is
            // undefined rather than infinite.
            return Err(CurveError::AnalysisError(
                "derivative_at: bracketing abscissas have equal squares".to_string(),
            ));
        }

        let a = d_div(rise, run, op).map_err(analysis_err)?;
        let slope = d_mul(dec!(2.0), a, op).map_err(analysis_err)?;
        let derivative = d_mul(slope, point.x, op).map_err(analysis_err)?;

        Ok(vec![derivative])
    }

    fn extrema(&self) -> Result<(Point2D, Point2D), Self::Error> {
        if self.points.is_empty() {
            return Err(CurveError::invalid_parameters(
                "extrema",
                "Curve has no points",
            ));
        }

        let min_point = self
            .points
            .iter()
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| {
                CurveError::AnalysisError("extrema: empty curve in min_by".to_string())
            })?;

        let max_point = self
            .points
            .iter()
            .max_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
            .ok_or_else(|| {
                CurveError::AnalysisError("extrema: empty curve in max_by".to_string())
            })?;

        Ok((min_point, max_point))
    }

    fn measure_under(&self, base_value: &Decimal) -> Result<Decimal, Self::Error> {
        if self.points.len() < 2 {
            return Ok(Decimal::ZERO);
        }

        let mut area = Decimal::ZERO;
        let points: Vec<_> = self.points.iter().collect();

        let op = "Curve::measure_under";

        // Approximate area using trapezoidal rule
        for pair in points.windows(2) {
            let (Some(left), Some(right)) = (pair.first(), pair.get(1)) else {
                return Err(CurveError::AnalysisError(
                    "measure_under: trapezoid window is shorter than two points".to_string(),
                ));
            };
            let width = d_sub(right.x, left.x, op).map_err(analysis_err)?;
            let left_height = d_sub(left.y, *base_value, op).map_err(analysis_err)?;
            let right_height = d_sub(right.y, *base_value, op).map_err(analysis_err)?;
            let sum_heights = d_add(left_height, right_height, op).map_err(analysis_err)?;
            let height = d_div(sum_heights, Decimal::TWO, op).map_err(analysis_err)?;
            let slice = d_mul(width, height, op).map_err(analysis_err)?;
            area = d_add(area, slice, op).map_err(analysis_err)?;
        }

        Ok(area.abs())
    }
}

#[cfg(test)]
mod tests_curves {
    use super::*;

    use crate::curves::utils::{create_constant_curve, create_linear_curve};
    use Decimal;
    use positive::{Positive, pos_or_panic};
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
        let x = pos_or_panic!(1.5_f64);
        let y = pos_or_panic!(2.5_f64);
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
        assert_eq!(tuple, (pos_or_panic!(1.5), pos_or_panic!(2.5)));
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
        let x = pos_or_panic!(1.5_f64);
        let y = pos_or_panic!(2.5_f64);
        let point = Point2D::from_tuple(x, y).unwrap();
        assert_eq!(point, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    #[test]
    fn test_new_with_mixed_types() {
        let x = dec!(1.5);
        let y = pos_or_panic!(2.5_f64);
        let point = Point2D::new(x, y);
        assert_eq!(point.x, dec!(1.5));
        assert_eq!(point.y, dec!(2.5));
    }

    #[test]
    fn test_create_constant_curve() {
        let curve = create_constant_curve(dec!(1.0), dec!(2.0), dec!(5.0)).unwrap();
        assert_eq!(curve.get_points().len(), 11);
        for point in curve.get_points() {
            assert_eq!(point.y, dec!(5.0));
        }
    }

    #[test]
    fn test_create_linear_curve() {
        let curve = create_linear_curve(dec!(1.0), dec!(2.0), dec!(2.0)).unwrap();
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
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
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
    fn test_linear_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.1), InterpolationType::Linear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(1.1), InterpolationType::Linear)
                .is_err()
        );
    }

    #[test]
    fn test_linear_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(0.0),
            dec!(0.0),
        )]));

        assert!(
            curve
                .interpolate(dec!(0.5), InterpolationType::Linear)
                .is_err()
        );
    }

    #[test]
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
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
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
    fn test_bilinear_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.5), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(1.5), InterpolationType::Bilinear)
                .is_err()
        );
    }

    #[test]
    fn test_bilinear_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(0.5), InterpolationType::Bilinear)
                .is_err()
        );
    }

    #[test]
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
    fn test_bilinear_interpolation_boundaries() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.1), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(1.1), InterpolationType::Bilinear)
                .is_err()
        );
    }

    #[test]
    fn test_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(Decimal::ZERO, Decimal::ZERO),
            Point2D::new(Decimal::ONE, Decimal::ONE),
            Point2D::new(Decimal::ZERO, Decimal::ONE),
            Point2D::new(Decimal::ONE, Decimal::TWO),
        ]));

        assert!(
            curve
                .interpolate(dec!(-1), InterpolationType::Bilinear)
                .is_err()
        );
        assert!(
            curve
                .interpolate(Decimal::TWO, InterpolationType::Bilinear)
                .is_err()
        );
    }

    /// The three curves the reference table was built on. `quadratic_*` are
    /// the samples of `y = x^2`; `irregular` has neither uniform spacing nor
    /// a monotone ordinate, so the far edge of a cell is a different width
    /// from the near one.
    fn quadratic_four() -> Curve {
        Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(2), dec!(4)),
            Point2D::new(dec!(3), dec!(9)),
        ]))
    }

    fn quadratic_five() -> Curve {
        Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(2), dec!(4)),
            Point2D::new(dec!(3), dec!(9)),
            Point2D::new(dec!(4), dec!(16)),
        ]))
    }

    fn irregular() -> Curve {
        Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(1)),
            Point2D::new(dec!(0.5), dec!(2.25)),
            Point2D::new(dec!(2), dec!(-1.5)),
            Point2D::new(dec!(3.25), dec!(4)),
            Point2D::new(dec!(5), dec!(0.75)),
        ]))
    }

    /// The two abscissas that used to abort and then, after #446, returned
    /// `InterpolationError::Bilinear`. Reference values computed in exact
    /// rational arithmetic from the rule stated on the impl: mean of the
    /// bracketing segment and the segment two on, the second clamped to the
    /// curve's last.
    ///
    /// `x = 1.5`: near edge `(1,1)-(2,4)` gives `2.5`, far edge clamped to
    /// `(2,4)-(3,9)` gives `6.5`, mean `4.5`.
    /// `x = 2.5`: both edges are `(2,4)-(3,9)`, mean of `6.5` with itself.
    #[test]
    fn test_bilinear_interpolate_four_point_upper_segments_match_reference() {
        let curve = quadratic_four();

        let mid = curve.bilinear_interpolate(dec!(1.5)).unwrap();
        assert_eq!(mid.x, dec!(1.5));
        assert_eq!(mid.y, dec!(4.5));

        let last = curve.bilinear_interpolate(dec!(2.5)).unwrap();
        assert_eq!(last.x, dec!(2.5));
        assert_eq!(last.y, dec!(6.5));
    }

    /// On a five-point curve the clamp bites on the last two segments only.
    /// `x = 2.5`: near `(2,4)-(3,9)` gives `6.5`, far clamped to
    /// `(3,9)-(4,16)` gives `12.5`, mean `9.5`.
    /// `x = 3.5`: both edges are `(3,9)-(4,16)`, mean of `12.5` with itself.
    #[test]
    fn test_bilinear_interpolate_five_point_upper_segments_match_reference() {
        let curve = quadratic_five();

        let second_to_last = curve.bilinear_interpolate(dec!(2.5)).unwrap();
        assert_eq!(second_to_last.y, dec!(9.5));

        let last = curve.bilinear_interpolate(dec!(3.5)).unwrap();
        assert_eq!(last.y, dec!(12.5));
    }

    /// The clamp only reaches the last two segments, so every abscissa the
    /// method already answered keeps its answer. These are the values the
    /// pre-#451 code produced.
    #[test]
    fn test_bilinear_interpolate_interior_is_unchanged_by_the_clamp() {
        let four = quadratic_four();
        assert_eq!(four.bilinear_interpolate(dec!(0.5)).unwrap().y, dec!(3.5));

        let five = quadratic_five();
        assert_eq!(five.bilinear_interpolate(dec!(0.5)).unwrap().y, dec!(3.5));
        assert_eq!(five.bilinear_interpolate(dec!(1.5)).unwrap().y, dec!(7.5));
    }

    /// On the final segment the two edges of the cell are the same segment,
    /// so the mean of the two collapses to the linear interpolant.
    #[test]
    fn test_bilinear_interpolate_final_segment_equals_linear() {
        for (curve, x) in [(quadratic_four(), dec!(2.5)), (quadratic_five(), dec!(3.5))] {
            let bilinear = curve.bilinear_interpolate(x).unwrap();
            let linear = curve.linear_interpolate(x).unwrap();
            assert_eq!(bilinear.y, linear.y);
        }
    }

    /// The same identity where the fraction along the segment has no finite
    /// decimal expansion. The two methods divide at different points of the
    /// chain, `bilinear` normalising `x` before scaling by the rise and
    /// `linear` scaling first, so each rounds at scale 28 on a different
    /// quantity and the two answers can part company in the last place.
    #[test]
    fn test_bilinear_interpolate_final_segment_equals_linear_to_the_last_place() {
        let curve = irregular();
        let bilinear = curve.bilinear_interpolate(dec!(4)).unwrap().y;
        let linear = curve.linear_interpolate(dec!(4)).unwrap().y;

        assert_ne!(bilinear, linear);
        assert!((bilinear - linear).abs() < dec!(0.0000000000000000000000001));
    }

    /// Reference values for the irregular curve, exact rational arithmetic:
    ///
    /// ```text
    /// x = 0.25  near seg 0, far seg 2  ->  1.4375
    /// x = 1     near seg 1, far seg 3  ->  47/24     = 1.95833...
    /// x = 2.5   near seg 2, far seg 3  ->  1.7
    /// x = 4     near seg 3, far seg 3  ->  73/28     = 2.60714...
    /// ```
    ///
    /// `x = 1` and `x = 4` land on a fraction with no finite decimal
    /// expansion, so `d_div` rounds it at scale 28 and the result carries the
    /// rounding forward; the bound below is far above the observed error of
    /// roughly `1.7e-28` and far below anything a caller could care about.
    #[test]
    fn test_bilinear_interpolate_irregular_curve_matches_reference() {
        const TOLERANCE: Decimal = dec!(0.0000000000000000000000001);
        let curve = irregular();

        assert_eq!(
            curve.bilinear_interpolate(dec!(0.25)).unwrap().y,
            dec!(1.4375)
        );
        assert_eq!(curve.bilinear_interpolate(dec!(2.5)).unwrap().y, dec!(1.7));

        let recurring = curve.bilinear_interpolate(dec!(1)).unwrap().y;
        assert!((recurring - dec!(1.9583333333333333333333333333)).abs() < TOLERANCE);

        let last = curve.bilinear_interpolate(dec!(4)).unwrap().y;
        assert!((last - dec!(2.6071428571428571428571428571)).abs() < TOLERANCE);
    }

    /// An abscissa carrying a sample returns that sample, on both ends of the
    /// curve and in the middle.
    #[test]
    fn test_bilinear_interpolate_sample_abscissa_returns_the_sample() {
        let curve = quadratic_four();
        for (x, y) in [
            (dec!(0), dec!(0)),
            (dec!(1), dec!(1)),
            (dec!(2), dec!(4)),
            (dec!(3), dec!(9)),
        ] {
            let point = curve.bilinear_interpolate(x).unwrap();
            assert_eq!(point.x, x);
            assert_eq!(point.y, y);
        }
    }

    /// Several ordinates at the queried abscissa leave the curve with no
    /// value there, and the exact-match branch now reports that instead of
    /// returning the lowest of them, which is what the three sibling
    /// algorithms do.
    #[test]
    fn test_bilinear_interpolate_stacked_abscissa_is_degenerate() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0), dec!(0)),
            Point2D::new(dec!(1), dec!(1)),
            Point2D::new(dec!(1), dec!(7)),
            Point2D::new(dec!(2), dec!(4)),
            Point2D::new(dec!(3), dec!(9)),
        ]));

        let err = curve.bilinear_interpolate(dec!(1)).unwrap_err();
        assert!(matches!(err, InterpolationError::DegenerateInterval));
    }

    /// The jump at a sample predates the boundary rule. At `x = 1` both
    /// segments meeting there take their far edge unclamped, so all three
    /// values below are the ones the pre-#451 code produced: the two
    /// one-sided limits agree with each other on `(4 + 16) / 2` and neither
    /// agrees with the sample.
    #[test]
    fn test_bilinear_interpolate_jump_at_a_sample_predates_the_clamp() {
        const EPSILON: Decimal = dec!(0.0000000001);
        const SLACK: Decimal = dec!(0.00000001);
        let curve = quadratic_five();

        let from_left = curve.bilinear_interpolate(dec!(1) - EPSILON).unwrap().y;
        assert!((from_left - dec!(5)).abs() < SLACK);

        let from_right = curve.bilinear_interpolate(dec!(1) + EPSILON).unwrap().y;
        assert!((from_right - dec!(5)).abs() < SLACK);

        assert_eq!(curve.bilinear_interpolate(dec!(1)).unwrap().y, dec!(1));
    }

    /// The clamp gives the segments on either side of `x = 2` different far
    /// edges, so the one-sided limits there disagree, and neither matches the
    /// sample. Approaching from the left the far edge is the segment
    /// `(3,9)-(4,16)` and the limit is the mean of `4` and `16`; from the
    /// right it is clamped back to that same segment read at fraction zero,
    /// and the limit is the mean of `4` and `9`. The sample itself is `4`.
    /// The doc comment on the impl calls this jump out; the test pins it.
    #[test]
    fn test_bilinear_interpolate_jumps_where_the_clamp_starts() {
        const EPSILON: Decimal = dec!(0.0000000001);
        const SLACK: Decimal = dec!(0.00000001);
        let curve = quadratic_five();

        let from_left = curve.bilinear_interpolate(dec!(2) - EPSILON).unwrap().y;
        assert!((from_left - dec!(10)).abs() < SLACK);

        let from_right = curve.bilinear_interpolate(dec!(2) + EPSILON).unwrap().y;
        assert!((from_right - dec!(6.5)).abs() < SLACK);

        assert_eq!(curve.bilinear_interpolate(dec!(2)).unwrap().y, dec!(4));
    }

    /// The near edge is never clamped, so the fraction along it stays in
    /// `[0, 1]` and the answer is a convex combination of four sample
    /// ordinates: it cannot leave the range the curve's ordinates span. This
    /// is what a clamp of the whole window to the last four samples would
    /// give up.
    #[test]
    fn test_bilinear_interpolate_stays_within_the_ordinate_range() {
        for curve in [quadratic_four(), quadratic_five(), irregular()] {
            let ordinates: Vec<Decimal> = curve.points.iter().map(|p| p.y).collect();
            let low = ordinates.iter().copied().fold(Decimal::MAX, Decimal::min);
            let high = ordinates.iter().copied().fold(Decimal::MIN, Decimal::max);

            let first = curve.points.iter().next().unwrap().x;
            let last = curve.points.iter().next_back().unwrap().x;
            let step = (last - first) / dec!(64);

            let mut x = first;
            while x <= last {
                let y = curve.bilinear_interpolate(x).unwrap().y;
                assert!(y >= low, "{y} below the sample range at x = {x}");
                assert!(y <= high, "{y} above the sample range at x = {x}");
                x += step;
            }
        }
    }
}

#[cfg(test)]
mod tests_cubic_interpolate {
    use super::*;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;
    use tracing::info;

    #[test]
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
    fn test_cubic_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(1.5), InterpolationType::Cubic)
                .is_err()
        );
    }

    #[test]
    fn test_cubic_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.5), InterpolationType::Cubic)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(3.5), InterpolationType::Cubic)
                .is_err()
        );
    }

    #[test]
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
        info!("p1: {:?}, p2: {:?}, p3: {:?}", p1, p2, p3);
    }
}

#[cfg(test)]
mod tests_spline_interpolate {
    use super::*;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
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
    fn test_spline_interpolation_insufficient_points() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(0.5), InterpolationType::Spline)
                .is_err()
        );
    }

    #[test]
    fn test_spline_interpolation_out_of_range() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        assert!(
            curve
                .interpolate(dec!(-0.5), InterpolationType::Spline)
                .is_err()
        );
        assert!(
            curve
                .interpolate(dec!(2.5), InterpolationType::Spline)
                .is_err()
        );
    }

    #[test]
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
    use crate::curves::utils::{create_constant_curve, create_linear_curve};
    use crate::geometrics::InterpolationType;

    #[test]
    fn test_merge_curves_add() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Add).unwrap();

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
    fn test_merge_curves_subtract() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0)).unwrap();
        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Subtract).unwrap();
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
    fn test_merge_curves_multiply() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0)).unwrap();

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Multiply).unwrap();

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
    fn test_merge_curves_divide() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(6.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Divide).unwrap();

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

    /// Division is not associative, so every divisor has to reach the result.
    /// `Curve::merge` reaches them by turning each element after the first
    /// into its reciprocal and folding the whole set with a combining
    /// reducer, so no partial is discarded. Three curves at 8, 2 and 2 must
    /// give `8 / 2 / 2 = 2`, never `8 / 2` and never 8.
    #[test]
    fn test_merge_curves_divide_folds_every_divisor() {
        let eight = create_constant_curve(dec!(0.0), dec!(10.0), dec!(8.0)).unwrap();
        let two_a = create_constant_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let two_b = create_constant_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();

        let result = Curve::merge(&[&eight, &two_a, &two_b], MergeOperation::Divide).unwrap();

        for x in [dec!(0.0), dec!(5.0), dec!(10.0)] {
            let y = result.interpolate(x, InterpolationType::Cubic).unwrap().y;
            assert!(
                (y - dec!(2)).abs() < dec!(0.0001),
                "expected 8 / 2 / 2 = 2 at x = {x}, got {y}"
            );
        }
    }

    #[test]
    fn test_merge_curves_max() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0)).unwrap();

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Max).unwrap();

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
    fn test_merge_curves_min() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0)).unwrap();

        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Min).unwrap();

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
    fn test_merge_with_single_operation() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0)).unwrap();

        let result = curve1.merge_with(&curve2, MergeOperation::Add).unwrap();

        // Verify that merge_with is equivalent to merge_curves with two curves
        let merged_result = Curve::merge(&[&curve1, &curve2], MergeOperation::Add).unwrap();

        // Compare points of both results
        assert_eq!(result.points.len(), merged_result.points.len());

        for i in 0..result.points.len() {
            assert!((result[i].x - merged_result[i].x).abs() < dec!(0.001));
            assert!((result[i].y - merged_result[i].y).abs() < dec!(0.001));
        }
    }

    #[test]
    fn test_merge_curves_error_handling() {
        // Test with empty slice
        let result = Curve::merge(&[], MergeOperation::Add);
        assert!(result.is_err());

        // Test with curves of incompatible ranges
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0)).unwrap();
        let curve2 = create_linear_curve(dec!(5.0), dec!(15.0), dec!(2.0)).unwrap();

        // Verify that the merge operation works even with partially overlapping ranges
        let result = Curve::merge(&[&curve1, &curve2], MergeOperation::Add);
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_multiple_curves() {
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(1.0)).unwrap();
        let curve2 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(2.0)).unwrap();
        let curve3 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(3.0)).unwrap();

        let result = Curve::merge(&[&curve1, &curve2, &curve3], MergeOperation::Add).unwrap();

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
    use super::*;
    use crate::error::CurveError::OperationError;
    use crate::error::{ChainError, OperationErrorKind};
    use crate::geometrics::{ConstructionMethod, ConstructionParams};

    #[test]
    fn test_construct_from_data_empty() {
        let result = Curve::construct(ConstructionMethod::FromData {
            points: BTreeSet::new(),
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::Point2DError { reason } => {
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
        let params = ConstructionParams::D2 {
            t_start: Decimal::ZERO,
            t_end: dec!(10.0),
            steps: 10,
        };
        let result = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(f),
            params,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_construct_parametric_invalid_function() {
        let f = |_t: Decimal| -> Result<Point2D, ChainError> {
            Err(ChainError::invalid_parameters(
                "parametric_f",
                "Function evaluation failed",
            ))
        };
        let params = ConstructionParams::D2 {
            t_start: Decimal::ZERO,
            t_end: dec!(10.0),
            steps: 10,
        };
        let result = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(f),
            params,
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::ConstructionError(reason) => {
                assert!(reason.contains("Function evaluation failed"));
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_segment_not_found_error() {
        let segment: Option<Point2D> = None;
        let result: Result<Point2D, CurveError> = segment.ok_or_else(|| {
            CurveError::ConstructionError(
                "Could not find valid segment for interpolation".to_string(),
            )
        });
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            CurveError::ConstructionError(reason) => {
                assert_eq!(reason, "Could not find valid segment for interpolation");
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }

    #[test]
    fn test_compute_basic_metrics_placeholder() {
        let curve = Curve {
            points: BTreeSet::new(),
            x_range: (Default::default(), Default::default()),
        };
        let metrics = curve.compute_basic_metrics();
        assert!(metrics.is_ok());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.mean, Decimal::ZERO);
    }

    #[test]
    fn test_single_curve_return() {
        let curve = Curve {
            points: BTreeSet::new(),
            x_range: (Default::default(), Default::default()),
        };
        let result = if vec![curve.clone()].len() == 1 {
            Ok(curve.clone())
        } else {
            Err(CurveError::invalid_parameters(
                "merge_curves",
                "Invalid state",
            ))
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_curves_invalid_x_range() {
        let min_x = dec!(10.0);
        let max_x = dec!(5.0);
        let result = if min_x >= max_x {
            Err(CurveError::invalid_parameters(
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
            }
            _ => {
                panic!("Unexpected error type");
            }
        }
    }
}

#[cfg(test)]
mod tests_curve_metrics {
    use super::*;
    use crate::assert_decimal_eq;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    // Helper function to create test curves
    fn create_linear_curve() -> Curve {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(6.0)),
            Point2D::new(dec!(4.0), dec!(8.0)),
        ]);
        Curve::new(points)
    }

    fn create_non_linear_curve() -> Curve {
        Curve {
            points: (0..=20)
                .map(|x| Point2D {
                    x: Decimal::from(x),
                    y: Decimal::from(x * x % 7), // Ejemplo no lineal
                })
                .collect(),
            x_range: (Default::default(), Default::default()),
        }
    }

    fn create_constant_curve() -> Curve {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(5.0)),
            Point2D::new(dec!(1.0), dec!(5.0)),
            Point2D::new(dec!(2.0), dec!(5.0)),
        ]);
        Curve::new(points)
    }

    #[test]
    fn test_basic_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let basic_metrics = linear_curve.compute_basic_metrics().unwrap();

        // Expected values for linear curve
        assert_decimal_eq!(basic_metrics.mean, dec!(4.0), dec!(0.001));
        assert_decimal_eq!(basic_metrics.median, dec!(4.0), dec!(0.001));
        assert_decimal_eq!(basic_metrics.std_dev, dec!(2.82842712), dec!(0.001));

        // Constant curve
        let constant_curve = create_constant_curve();
        let constant_metrics = constant_curve.compute_basic_metrics().unwrap();

        assert_decimal_eq!(constant_metrics.mean, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_metrics.median, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_metrics.std_dev, dec!(0.0), dec!(0.001));
    }

    #[test]
    fn test_shape_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let shape_metrics = linear_curve.compute_shape_metrics().unwrap();

        // More lenient check for linear curve
        assert!(
            shape_metrics.skewness.abs() < dec!(0.5),
            "Skewness for linear curve should be very close to 0, got {}",
            shape_metrics.skewness
        );

        // Allow a wider range for kurtosis of a linear curve
        assert!(
            shape_metrics.kurtosis.abs() < dec!(2.0),
            "Kurtosis for linear curve should be close to 0, got {}",
            shape_metrics.kurtosis
        );

        // Non-linear curve
        let non_linear_curve = create_non_linear_curve();
        let non_linear_metrics = non_linear_curve.compute_shape_metrics().unwrap();

        // More nuanced checks for non-linear curve
        assert!(
            non_linear_metrics.skewness.abs() > dec!(0.003),
            "Non-linear curve should have significant skewness, got {}",
            non_linear_metrics.skewness
        );

        // Ensure the non-linear curve has a meaningfully different kurtosis
        assert!(
            non_linear_metrics.kurtosis.abs() > dec!(1.0),
            "Non-linear curve should have significant kurtosis, got {}",
            non_linear_metrics.kurtosis
        );

        // Check peaks and valleys
        assert!(
            !non_linear_metrics.peaks.is_empty(),
            "Peaks should be detected"
        );
        assert!(
            !non_linear_metrics.valleys.is_empty(),
            "Valleys should be detected"
        );
    }

    #[test]
    fn test_range_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let range_metrics = linear_curve.compute_range_metrics().unwrap();

        assert_decimal_eq!(range_metrics.min.y, dec!(0.0), dec!(0.001));
        assert_decimal_eq!(range_metrics.max.y, dec!(8.0), dec!(0.001));
        assert_decimal_eq!(range_metrics.range, dec!(8.0), dec!(0.001));

        // Constant curve
        let constant_curve = create_constant_curve();
        let constant_range_metrics = constant_curve.compute_range_metrics().unwrap();

        assert_decimal_eq!(constant_range_metrics.min.y, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_range_metrics.max.y, dec!(5.0), dec!(0.001));
        assert_decimal_eq!(constant_range_metrics.range, dec!(0.0), dec!(0.001));
    }

    #[test]
    fn test_trend_metrics() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let trend_metrics = linear_curve.compute_trend_metrics().unwrap();

        // Expected values for a perfectly linear curve
        assert_decimal_eq!(trend_metrics.slope, dec!(2.0), dec!(0.001));
        assert_decimal_eq!(trend_metrics.intercept, dec!(0.0), dec!(0.001));
        assert_decimal_eq!(trend_metrics.r_squared, dec!(1.0), dec!(0.001));

        // Non-linear curve
        let non_linear_curve = create_non_linear_curve();
        let non_linear_trend_metrics = non_linear_curve.compute_trend_metrics().unwrap();

        // R-squared should be less than 1
        assert!(non_linear_trend_metrics.r_squared < dec!(1.0));

        // Moving average should exist
        assert!(!non_linear_trend_metrics.moving_average.is_empty());
    }

    #[test]
    fn test_constant_curve_risk_metrics() {
        let constant_curve = create_constant_curve();
        let risk_metrics = constant_curve.compute_risk_metrics().unwrap();

        assert_eq!(risk_metrics.volatility, dec!(0.0));
        assert_eq!(risk_metrics.beta, dec!(0.0));
        assert_eq!(risk_metrics.sharpe_ratio, dec!(0.0));
    }

    /// A flat curve has no uncertainty, but it is not worthless. Only the
    /// fields that are genuinely undefined at zero dispersion may be zeroed;
    /// the parametric VaR still has its deterministic limit at the mean.
    #[test]
    fn test_risk_metrics_flat_curve_keeps_deterministic_var() {
        let curve = create_constant_curve();
        let metrics = curve.compute_risk_metrics().unwrap();

        // Measured, and genuinely zero.
        assert_eq!(metrics.volatility, Decimal::ZERO);
        // `mean - 1.645 * 0` is the mean, not zero: the curve is worth 5.
        assert_eq!(metrics.value_at_risk, dec!(5));
        // No sample falls below the VaR, so the conditional mean has an empty
        // tail and the function's own empty-tail rule gives zero.
        assert_eq!(metrics.expected_shortfall, Decimal::ZERO);
        // `volatility / mean` is already zero here; no special case needed.
        assert_eq!(metrics.beta, Decimal::ZERO);
        // `mean / 0` is the one genuinely undefined field.
        assert_eq!(metrics.sharpe_ratio, Decimal::ZERO);
    }

    /// The same shape with a negative mean: the VaR limit follows the mean
    /// wherever it sits, so a sign error cannot hide behind a positive value.
    #[test]
    fn test_risk_metrics_flat_negative_curve_keeps_deterministic_var() {
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(-2.5)),
            Point2D::new(dec!(1.0), dec!(-2.5)),
            Point2D::new(dec!(2.0), dec!(-2.5)),
        ]);
        let metrics = Curve::new(points).compute_risk_metrics().unwrap();

        assert_eq!(metrics.volatility, Decimal::ZERO);
        assert_eq!(metrics.value_at_risk, dec!(-2.5));
        assert_eq!(metrics.sharpe_ratio, Decimal::ZERO);
    }

    #[test]
    fn test_risk_metrics() {
        // Curva lineal
        let linear_curve = create_linear_curve();
        let risk_metrics = linear_curve.compute_risk_metrics().unwrap();

        assert!(
            risk_metrics.volatility > dec!(0.0),
            "Volatility debe ser mayor a cero."
        );
        assert!(
            risk_metrics.value_at_risk != dec!(0.0),
            "Value at Risk no debe ser cero."
        );
        assert!(risk_metrics.beta != dec!(0.0), "Beta no debe ser cero.");
    }

    #[test]
    fn test_risk_metrics_bis() {
        // Linear curve
        let linear_curve = create_linear_curve();
        let risk_metrics = linear_curve.compute_risk_metrics().unwrap();

        // Volatility and risk metrics should be non-zero
        assert!(risk_metrics.volatility > dec!(0.0));
        assert!(risk_metrics.value_at_risk != dec!(0.0));
        assert!(risk_metrics.beta != dec!(0.0));

        // Constant curve
        let constant_curve = create_constant_curve();
        let constant_risk_metrics = constant_curve.compute_risk_metrics().unwrap();

        // Volatility should be zero for a constant curve
        assert_decimal_eq!(constant_risk_metrics.volatility, dec!(0.0), dec!(0.001));
    }

    #[test]
    fn test_edge_cases() {
        // Empty curve
        let empty_curve = Curve::new(BTreeSet::new());

        assert!(empty_curve.compute_basic_metrics().is_ok());
        assert!(empty_curve.compute_shape_metrics().is_ok());
        assert!(empty_curve.compute_range_metrics().is_ok());
        assert!(empty_curve.compute_trend_metrics().is_ok());
        assert!(empty_curve.compute_risk_metrics().is_ok());

        // Single point curve
        let single_point_curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(1.0),
            dec!(1.0),
        )]));

        assert!(single_point_curve.compute_basic_metrics().is_ok());
        assert!(single_point_curve.compute_shape_metrics().is_ok());
        assert!(single_point_curve.compute_range_metrics().is_ok());
        assert!(single_point_curve.compute_trend_metrics().is_ok());
        assert!(single_point_curve.compute_risk_metrics().is_ok());
    }
}

#[cfg(test)]
mod tests_merge_axis_interpolate {
    use super::*;
    use crate::curves::utils::create_linear_curve;
    use crate::geometrics::InterpolationType;
    use rust_decimal_macros::dec;

    #[test]
    fn test_merge_axis_interpolate_linear() {
        // Create two curves with different x ranges and points
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(0.5)).unwrap();
        let curve2 = create_linear_curve(dec!(4.0), dec!(20.0), dec!(1.0)).unwrap();

        // Merge and interpolate using linear interpolation
        let result = curve1.merge_axis_interpolate(&curve2, InterpolationType::Linear);

        assert!(result.is_ok());
        let (interpolated_curve1, interpolated_curve2) = result.unwrap();

        // Verify that both interpolated curves have the same x range
        assert_eq!(interpolated_curve1.x_range.0, interpolated_curve2.x_range.0);
        assert_eq!(interpolated_curve1.x_range.1, interpolated_curve2.x_range.1);

        // Verify number of points (should cover full merged x range)
        assert_eq!(interpolated_curve1.points.len(), 10);
        assert_eq!(interpolated_curve2.points.len(), 10);
        assert_eq!(interpolated_curve1.x_range, interpolated_curve2.x_range);
        assert_eq!(
            interpolated_curve1.get_index_values(),
            interpolated_curve2.get_index_values()
        );
    }

    #[test]
    fn test_merge_axis_interpolate_cubic() {
        // Create two curves with different x ranges and points
        let curve1 = create_linear_curve(dec!(0.0), dec!(10.0), dec!(0.5)).unwrap();
        let curve2 = create_linear_curve(dec!(4.0), dec!(20.0), dec!(1.0)).unwrap();

        // Merge and interpolate using cubic interpolation
        let result = curve1.merge_axis_interpolate(&curve2, InterpolationType::Cubic);

        assert!(result.is_ok());
        let (interpolated_curve1, interpolated_curve2) = result.unwrap();

        // Verify that both interpolated curves have the same x range
        assert_eq!(interpolated_curve1.x_range.0, interpolated_curve2.x_range.0);
        assert_eq!(interpolated_curve1.x_range.1, interpolated_curve2.x_range.1);

        // Verify number of points (should cover full merged x range)
        assert_eq!(interpolated_curve1.points.len(), 10);
        assert_eq!(interpolated_curve2.points.len(), 10);
        assert_eq!(interpolated_curve1.x_range, interpolated_curve2.x_range);
        assert_eq!(
            interpolated_curve1.get_index_values(),
            interpolated_curve2.get_index_values()
        );
    }
}

#[cfg(test)]
mod tests_geometric_transformations {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_curve() -> Curve {
        Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]))
    }

    mod test_translate {
        use super::*;

        #[test]
        fn test_translate_positive() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(2.0), &dec!(3.0)]).unwrap();

            let translated_points: Vec<_> = result.points.iter().collect();
            assert_eq!(translated_points[0].x, dec!(2.0));
            assert_eq!(translated_points[0].y, dec!(3.0));

            assert_eq!(translated_points[1].x, dec!(3.0));
            assert_eq!(translated_points[1].y, dec!(4.0));

            assert_eq!(translated_points[2].x, dec!(4.0));
            assert_eq!(translated_points[2].y, dec!(7.0));

            assert_eq!(translated_points[3].x, dec!(5.0));
            assert_eq!(translated_points[3].y, dec!(12.0));
        }

        #[test]
        fn test_translate_negative() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(-1.0), &dec!(-2.0)]).unwrap();

            let translated_points: Vec<_> = result.points.iter().collect();
            assert_eq!(translated_points[0].x, dec!(-1.0));
            assert_eq!(translated_points[0].y, dec!(-2.0));
        }

        #[test]
        fn test_translate_zero() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(0.0), &dec!(0.0)]).unwrap();
            assert_eq!(curve.points, result.points);
        }

        #[test]
        fn test_translate_wrong_dimensions() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(1.0)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_translate_preserves_shape() {
            let curve = create_test_curve();
            let result = curve.translate(vec![&dec!(1.0), &dec!(1.0)]).unwrap();

            let original_diffs: Vec<Decimal> = curve
                .points
                .iter()
                .zip(curve.points.iter().skip(1))
                .map(|(a, b)| b.y - a.y)
                .collect();

            let translated_diffs: Vec<Decimal> = result
                .points
                .iter()
                .zip(result.points.iter().skip(1))
                .map(|(a, b)| b.y - a.y)
                .collect();

            assert_eq!(original_diffs, translated_diffs);
        }
    }

    mod test_scale {
        use super::*;

        #[test]
        fn test_scale_uniform() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(2.0), &dec!(2.0)]).unwrap();

            let scaled_points: Vec<_> = result.points.iter().collect();

            assert_eq!(scaled_points[0].x, dec!(0.0));
            assert_eq!(scaled_points[0].y, dec!(0.0));

            assert_eq!(scaled_points[1].x, dec!(2.0));
            assert_eq!(scaled_points[1].y, dec!(2.0));

            assert_eq!(scaled_points[2].x, dec!(4.0));
            assert_eq!(scaled_points[2].y, dec!(8.0));

            assert_eq!(scaled_points[3].x, dec!(6.0));
            assert_eq!(scaled_points[3].y, dec!(18.0));
        }

        #[test]
        fn test_scale_non_uniform() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(2.0), &dec!(3.0)]).unwrap();

            let scaled_points: Vec<_> = result.points.iter().collect();
            assert_eq!(scaled_points[1].x, dec!(2.0));
            assert_eq!(scaled_points[1].y, dec!(3.0));
        }

        #[test]
        fn test_scale_zero() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(0.0), &dec!(0.0)]).unwrap();

            assert!(
                result
                    .points
                    .iter()
                    .all(|p| p.x == dec!(0.0) && p.y == dec!(0.0))
            );
        }

        #[test]
        fn test_scale_wrong_dimensions() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(2.0)]);
            assert!(result.is_err());
        }

        #[test]
        fn test_scale_negative() {
            let curve = create_test_curve();
            let result = curve.scale(vec![&dec!(-1.0), &dec!(-1.0)]).unwrap();

            assert_eq!(result[1].x, dec!(-2.0));
            assert_eq!(result[1].y, dec!(-4.0));

            assert_eq!(result[3].x, dec!(0.0));
            assert_eq!(result[3].y, dec!(0.0));
        }
    }

    mod test_intersect_with {
        use super::*;

        #[test]
        fn test_curves_intersect() {
            let curve1 = create_test_curve();
            let curve2 = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(2.0)),
            ]));

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert_eq!(intersections.len(), 1);
        }

        #[test]
        fn test_no_intersection() {
            let curve1 = create_test_curve();
            let curve2 = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(10.0), dec!(10.0)),
                Point2D::new(dec!(11.0), dec!(11.0)),
            ]));

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert!(intersections.is_empty());
        }

        #[test]
        fn test_multiple_intersections() {
            let curve1 = create_test_curve();
            let curve2 = create_test_curve();

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert_eq!(intersections.len(), curve1.points.len());
        }

        #[test]
        fn test_self_intersection() {
            let curve = create_test_curve();
            let intersections = curve.intersect_with(&curve).unwrap();
            assert_eq!(intersections.len(), curve.points.len());
        }

        #[test]
        fn test_empty_curves() {
            let curve1 = Curve::new(BTreeSet::new());
            let curve2 = Curve::new(BTreeSet::new());

            let intersections = curve1.intersect_with(&curve2).unwrap();
            assert!(intersections.is_empty());
        }
    }

    mod test_derivative_at {
        use super::*;

        #[test]
        fn test_linear_derivative() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let derivative = curve
                .derivative_at(&Point2D::new(dec!(0.5), dec!(0.5)))
                .unwrap();
            assert_eq!(derivative[0], dec!(1.0));
        }

        #[test]
        fn test_quadratic_derivative() {
            let curve = create_test_curve();
            let derivative = curve
                .derivative_at(&Point2D::new(dec!(1.0), dec!(1.0)))
                .unwrap();
            assert_eq!(derivative[0], dec!(2.0));
            let derivative2 = curve
                .derivative_at(&Point2D::new(dec!(2.0), dec!(4.0)))
                .unwrap();
            assert_eq!(derivative2[0], dec!(4.0));
        }

        #[test]
        fn test_out_of_range() {
            let curve = create_test_curve();
            let result = curve.derivative_at(&Point2D::new(dec!(10.0), dec!(0.0)));
            assert!(result.is_err());
        }

        #[test]
        fn test_at_endpoint() {
            let curve = create_test_curve();
            let derivative = curve
                .derivative_at(&Point2D::new(dec!(0.0), dec!(0.0)))
                .unwrap();
            assert!(derivative[0] == dec!(0.0));
        }

        #[test]
        fn test_vertical_line() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(1.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let result = curve.derivative_at(&Point2D::new(dec!(1.0), dec!(0.5)));
            assert!(result.is_err());
        }
    }

    mod test_extrema {
        use super::*;

        #[test]
        fn test_find_extrema() {
            let curve = create_test_curve();
            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min.y, dec!(0.0));
            assert_eq!(max.y, dec!(9.0));
        }

        #[test]
        fn test_empty_curve() {
            let curve = Curve::new(BTreeSet::new());
            let result = curve.extrema();
            assert!(result.is_err());
        }

        #[test]
        fn test_single_point() {
            let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
                dec!(1.0),
                dec!(1.0),
            )]));

            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min, max);
        }

        #[test]
        fn test_flat_curve() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(1.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min.y, max.y);
        }

        #[test]
        fn test_multiple_extrema() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
                Point2D::new(dec!(2.0), dec!(0.0)),
            ]));

            let (min, max) = curve.extrema().unwrap();
            assert_eq!(min.y, dec!(0.0));
            assert_eq!(max.y, dec!(1.0));
        }
    }

    mod test_measure_under {
        use super::*;

        #[test]
        fn test_area_under_linear() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(0.0)),
                Point2D::new(dec!(1.0), dec!(1.0)),
            ]));

            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(area, dec!(0.5));
        }

        #[test]
        fn test_area_empty_curve() {
            let curve = Curve::new(BTreeSet::new());
            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(area, dec!(0.0));
        }

        #[test]
        fn test_area_single_point() {
            let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
                dec!(1.0),
                dec!(1.0),
            )]));

            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert_eq!(area, dec!(0.0));
        }

        #[test]
        fn test_area_with_base_value() {
            let curve = create_test_curve();
            let area1 = curve.measure_under(&dec!(0.0)).unwrap();
            let area2 = curve.measure_under(&dec!(1.0)).unwrap();
            assert!(area1 > area2);
        }

        #[test]
        fn test_negative_area() {
            let curve = Curve::new(BTreeSet::from_iter(vec![
                Point2D::new(dec!(0.0), dec!(-1.0)),
                Point2D::new(dec!(1.0), dec!(-2.0)),
            ]));

            let area = curve.measure_under(&dec!(0.0)).unwrap();
            assert!(area > dec!(0.0));
        }
    }
}

#[cfg(test)]
mod tests_curve_serde {
    use super::*;
    use rust_decimal_macros::dec;

    // Helper function to create a test curve
    fn create_test_curve() -> Curve {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: dec!(1.0),
            y: dec!(2.0),
        });
        points.insert(Point2D {
            x: dec!(3.0),
            y: dec!(4.0),
        });
        points.insert(Point2D {
            x: dec!(5.0),
            y: dec!(6.0),
        });

        Curve {
            points,
            x_range: (dec!(1.0), dec!(5.0)),
        }
    }

    #[test]
    fn test_basic_serialization() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }

    #[test]
    fn test_pretty_print() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string_pretty(&curve).unwrap();

        // Verify pretty print format
        assert!(serialized.contains('\n'));
        assert!(serialized.contains("  "));

        // Verify deserialization still works
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();
        assert_eq!(curve.points, deserialized.points);
    }

    #[test]
    fn test_empty_curve() {
        let curve = Curve {
            points: BTreeSet::new(),
            x_range: (dec!(0.0), dec!(0.0)),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert!(deserialized.points.is_empty());
        assert_eq!(deserialized.x_range, (dec!(0.0), dec!(0.0)));
    }

    #[test]
    fn test_curve_with_negative_values() {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: dec!(-1.0),
            y: dec!(-2.0),
        });
        points.insert(Point2D {
            x: dec!(-3.0),
            y: dec!(-4.0),
        });

        let curve = Curve {
            points,
            x_range: (dec!(-3.0), dec!(-1.0)),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }

    #[test]
    fn test_curve_with_high_precision() {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: dec!(1.12345678901234567890),
            y: dec!(2.12345678901234567890),
        });
        points.insert(Point2D {
            x: dec!(3.12345678901234567890),
            y: dec!(4.12345678901234567890),
        });

        let curve = Curve {
            points,
            x_range: (dec!(1.12345678901234567890), dec!(3.12345678901234567890)),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }

    #[test]
    fn test_invalid_json() {
        // Missing required fields
        let json_str = r#"{"points": []}"#;
        let result = serde_json::from_str::<Curve>(json_str);
        assert!(result.is_err());

        // Invalid points format
        let json_str = r#"{"points": [1, 2, 3], "x_range": [0, 1]}"#;
        let result = serde_json::from_str::<Curve>(json_str);
        assert!(result.is_err());

        // Invalid x_range format
        let json_str = r#"{"points": [], "x_range": "invalid"}"#;
        let result = serde_json::from_str::<Curve>(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_structure() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string(&curve).unwrap();
        let json: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Check structure
        assert!(json.is_object());
        assert!(json.get("points").is_some());
        assert!(json.get("x_range").is_some());

        // Check points is an array
        assert!(json.get("points").unwrap().is_array());

        // Check x_range is an array of 2 elements
        let x_range = json.get("x_range").unwrap().as_array().unwrap();
        assert_eq!(x_range.len(), 2);
    }

    #[test]
    fn test_multiple_curves() {
        let curve1 = create_test_curve();
        let mut curve2 = create_test_curve();
        curve2.x_range = (dec!(6.0), dec!(10.0));

        let curves = vec![curve1, curve2];
        let serialized = serde_json::to_string(&curves).unwrap();
        let deserialized: Vec<Curve> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curves.len(), deserialized.len());
        assert_eq!(curves[0].points, deserialized[0].points);
        assert_eq!(curves[1].points, deserialized[1].points);
    }

    #[test]
    fn test_ordering_preservation() {
        let curve = create_test_curve();
        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        // Convert points to vectors to check ordering
        let original_points: Vec<_> = curve.points.into_iter().collect();
        let deserialized_points: Vec<_> = deserialized.points.into_iter().collect();

        // Check if points maintain their order
        assert_eq!(original_points, deserialized_points);
    }

    #[test]
    fn test_curve_with_extremes() {
        let mut points = BTreeSet::new();
        points.insert(Point2D {
            x: Decimal::MAX,
            y: Decimal::MAX,
        });
        points.insert(Point2D {
            x: Decimal::MIN,
            y: Decimal::MIN,
        });

        let curve = Curve {
            points,
            x_range: (Decimal::MIN, Decimal::MAX),
        };

        let serialized = serde_json::to_string(&curve).unwrap();
        let deserialized: Curve = serde_json::from_str(&serialized).unwrap();

        assert_eq!(curve.points, deserialized.points);
        assert_eq!(curve.x_range, deserialized.x_range);
    }
}

#[cfg(test)]
mod tests_curve_display_and_default {
    use crate::curves::{Curve, Point2D};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    #[test]
    fn test_curve_display() {
        let point1 = Point2D::new(dec!(1.0), dec!(2.0));
        let point2 = Point2D::new(dec!(3.0), dec!(4.0));

        let mut points = BTreeSet::new();
        points.insert(point1);
        points.insert(point2);

        let curve = Curve::new(points);

        let display_string = format!("{curve}");
        assert!(display_string.contains("(x: 1.0, y: 2.0)"));
        assert!(display_string.contains("(x: 3.0, y: 4.0)"));
    }

    #[test]
    fn test_curve_default() {
        // Test the Default implementation (line 83, 85-86)
        let curve = Curve::default();
        assert!(curve.points.is_empty());
        assert_eq!(curve.x_range, (Decimal::ZERO, Decimal::ZERO));
    }
}

#[cfg(test)]
mod tests_curve_len_and_geometric {
    use crate::curves::{Curve, Point2D};
    use crate::error::CurveError;
    use crate::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};
    use crate::utils::Len;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    #[test]
    fn test_curve_len() {
        // Test Len implementation (lines 129-130)
        let curve = Curve::default();
        assert_eq!(curve.len(), 0);
        assert!(curve.is_empty());

        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(1.0), dec!(2.0)));
        let curve_with_point = Curve::new(points);
        assert_eq!(curve_with_point.len(), 1);
        assert!(!curve_with_point.is_empty());
    }

    #[test]
    fn test_curve_get_points() {
        // Test GeometricObject.get_points (line 164)
        let mut points = BTreeSet::new();
        points.insert(Point2D::new(dec!(1.0), dec!(2.0)));
        points.insert(Point2D::new(dec!(3.0), dec!(4.0)));

        let curve = Curve::new(points);
        let retrieved_points = curve.get_points();

        assert_eq!(retrieved_points.len(), 2);
        assert!(
            retrieved_points
                .iter()
                .any(|p| p.x == dec!(1.0) && p.y == dec!(2.0))
        );
        assert!(
            retrieved_points
                .iter()
                .any(|p| p.x == dec!(3.0) && p.y == dec!(4.0))
        );
    }

    #[test]
    fn test_construct_method_error() {
        // Test ConstructionMethod errors (lines 168-175, 179, 181, 189)
        let result = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(|_| {
                Err(crate::error::ChainError::invalid_parameters(
                    "parametric_f",
                    "Test error",
                ))
            }),
            params: ConstructionParams::D2 {
                t_start: dec!(0.0),
                t_end: dec!(1.0),
                steps: 10,
            },
        });

        assert!(result.is_err());
        match result {
            Err(CurveError::ConstructionError(msg)) => {
                assert!(msg.contains("Test error"));
            }
            _ => panic!("Expected ConstructionError"),
        }

        // Test invalid params
        let result = Curve::construct(ConstructionMethod::Parametric {
            f: Box::new(|t| Ok(Point2D::new(t, t * dec!(2.0)))),
            params: ConstructionParams::D3 {
                x_start: dec!(0.0),
                x_end: dec!(1.0),
                y_start: dec!(0.0),
                y_end: dec!(1.0),
                x_steps: 10,
                y_steps: 10,
            },
        });

        assert!(result.is_err());
        match result {
            Err(CurveError::ConstructionError(msg)) => {
                assert_eq!(msg, "Invalid parameters");
            }
            _ => panic!("Expected ConstructionError"),
        }
    }
}

#[cfg(test)]
mod tests_interpolation_edge_cases {
    use crate::curves::{Curve, Point2D};
    use crate::geometrics::{AxisOperations, Interpolate, InterpolationType};
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;
    use tracing::info;

    #[test]
    fn test_cubic_interpolation_edge_cases() {
        // Test edge cases for cubic interpolation (line 924)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test interpolation at the start of the curve
        let start_result = curve.interpolate(dec!(0.25), InterpolationType::Cubic);
        assert!(start_result.is_ok());

        // Test interpolation at the end of the curve
        let end_result = curve.interpolate(dec!(2.75), InterpolationType::Cubic);
        assert!(end_result.is_ok());
    }

    #[test]
    fn test_spline_interpolation_edge_cases() {
        // Test edge cases for spline interpolation (lines 942-943, 1037)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        // Test with exact match to a point
        let exact_result = curve.interpolate(dec!(1.0), InterpolationType::Spline);
        assert!(exact_result.is_ok());
        let exact_point = exact_result.unwrap();
        assert_eq!(exact_point.x, dec!(1.0));
        assert_eq!(exact_point.y, dec!(1.0));

        // Test spline system solving at the midpoint
        let mid_result = curve.interpolate(dec!(1.5), InterpolationType::Spline);
        assert!(mid_result.is_ok());
    }

    #[test]
    fn test_axis_operations() {
        // Test AxisOperations (line 1155)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        // Test get_index_values
        let indices = curve.get_index_values();
        assert_eq!(indices, vec![dec!(0.0), dec!(1.0), dec!(2.0)]);

        // Test get_values
        let values = curve.get_values(dec!(1.0));
        assert_eq!(values.len(), 1);
        assert_eq!(*values[0], dec!(1.0));

        // Test get_closest_point
        let closest = curve.get_closest_point(&dec!(0.9)).unwrap();
        assert_eq!(closest.x, dec!(1.0));
        assert_eq!(closest.y, dec!(1.0));

        info!("{:?}", curve);
        // Test get_point
        let point = curve.get_point(&dec!(1.0));
        assert!(point.is_some());
        assert_eq!(point.unwrap().y, dec!(1.0));

        // Test get_point with non-existent x value
        let non_existent = curve.get_point(&dec!(1.5));
        assert!(non_existent.is_none());
    }
}

#[cfg(test)]
mod tests_axis_merge_and_transformations {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_merge_axis_index() {
        // Test merge_axis_index (lines 1227-1228, 1236)
        let curve1 = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        let curve2 = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ]));

        // Get merged x-values
        let merged = curve1.merge_axis_index(&curve2);
        assert_eq!(merged.len(), 1);
        assert!(merged.contains(&dec!(1.0)));
    }

    #[test]
    fn test_translate_with_negative_values() {
        // Test translate with negative values (line 1326)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        let result = curve.translate(vec![&dec!(-2.0), &dec!(-3.0)]).unwrap();

        assert_eq!(result.points.len(), 2);

        let translated_points: Vec<_> = result.points.iter().collect();
        assert_eq!(translated_points[0].x, dec!(-2.0));
        assert_eq!(translated_points[0].y, dec!(-3.0));
        assert_eq!(translated_points[1].x, dec!(-1.0));
        assert_eq!(translated_points[1].y, dec!(-2.0));
    }

    #[test]
    fn test_intersect_with_empty_curves() {
        // Test intersect_with empty curves (lines 1344-1346)
        let curve1 = Curve::new(BTreeSet::new());
        let curve2 = Curve::new(BTreeSet::new());

        let result = curve1.intersect_with(&curve2).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_derivative_edge_cases() {
        // Test derivative_at edge cases (lines 1467-1468, 1470-1471)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        // Test derivative at endpoint
        let result = curve
            .derivative_at(&Point2D::new(dec!(0.0), dec!(0.0)))
            .unwrap();
        assert_eq!(result[0], dec!(0.0));

        // Test derivative at midpoint
        let result = curve
            .derivative_at(&Point2D::new(dec!(0.5), dec!(0.5)))
            .unwrap();
        assert_eq!(result[0], dec!(1.0));
    }

    #[test]
    fn test_derivative_vertical_line() {
        // Test derivative for vertical line (lines 1475-1476)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(1.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        // This should return an error as the derivative is undefined for a vertical line
        let result = curve.derivative_at(&Point2D::new(dec!(1.0), dec!(0.5)));
        assert!(result.is_err());
    }

    #[test]
    fn test_extrema_single_point() {
        // Test extrema with single point (lines 1478-1481)
        let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(1.0),
            dec!(1.0),
        )]));

        let (min, max) = curve.extrema().unwrap();
        assert_eq!(min, max);
        assert_eq!(min.x, dec!(1.0));
        assert_eq!(min.y, dec!(1.0));
    }

    #[test]
    fn test_extrema_flat_curve() {
        // Test extrema with flat curve (lines 1483-1484)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
        ]));

        let (min, max) = curve.extrema().unwrap();
        assert_eq!(min.y, max.y);
        assert_eq!(min.y, dec!(1.0));
    }

    #[test]
    fn test_measure_under_single_point() {
        // Test measure_under with single point (lines 1488-1490)
        let curve = Curve::new(BTreeSet::from_iter(vec![Point2D::new(
            dec!(1.0),
            dec!(1.0),
        )]));

        let area = curve.measure_under(&dec!(0.0)).unwrap();
        assert_eq!(area, dec!(0.0));
    }

    #[test]
    fn test_measure_under_negative_area() {
        // Test measure_under with negative area (line 1492)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(-1.0)),
            Point2D::new(dec!(1.0), dec!(-2.0)),
        ]));

        let area = curve.measure_under(&dec!(0.0)).unwrap();
        assert!(area > dec!(0.0));
        assert_eq!(area, dec!(1.5)); // |0.5 * 1 * (-1.5)| = 0.75 * 2 = 1.5
    }

    #[test]
    fn test_extrema_multiple_extrema() {
        // Test extrema with multiple local extrema (lines 1518, 1524)
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(1.0)),
            Point2D::new(dec!(3.0), dec!(3.0)),
        ]));

        let (min, max) = curve.extrema().unwrap();
        assert_eq!(min.y, dec!(0.0));
        assert_eq!(max.y, dec!(3.0));
    }
}

/// A `Curve` is a function of its abscissa, and nothing enforces it. These
/// tests pin what each consumer does when the rule is broken, so that no
/// consumer picks a survivor from a stack of ordinates without saying so.
#[cfg(test)]
mod tests_duplicate_abscissa {
    use crate::curves::{Curve, Point2D};
    use crate::error::InterpolationError;
    use crate::geometrics::{AxisOperations, Interpolate, InterpolationType};
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    /// Four abscissae, with two ordinates stacked on `x = 1`: enough points
    /// for every interpolator, and the stack sits in the interior.
    fn stacked_curve() -> Curve {
        Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(3.0)),
            Point2D::new(dec!(2.0), dec!(2.0)),
            Point2D::new(dec!(3.0), dec!(5.0)),
        ]))
    }

    #[test]
    fn test_curve_new_stacked_abscissa_keeps_both_points() {
        let curve = stacked_curve();

        // `Point2D` orders and compares on the full pair, so both survive.
        assert_eq!(curve.points.len(), 5);
        assert_eq!(curve.x_range, (dec!(0.0), dec!(3.0)));
    }

    #[test]
    fn test_curve_get_values_stacked_abscissa_returns_every_ordinate() {
        let curve = stacked_curve();

        let values: Vec<_> = curve.get_values(dec!(1.0)).into_iter().copied().collect();
        assert_eq!(values, vec![dec!(1.0), dec!(3.0)]);
    }

    /// `get_point` is documented as reading the first match. This pins that
    /// it is the lowest ordinate, so the doc is checkable.
    #[test]
    fn test_curve_get_point_stacked_abscissa_returns_lowest_ordinate() {
        let curve = stacked_curve();

        let point = curve.get_point(&dec!(1.0)).expect("x = 1 is present");
        assert_eq!(*point, Point2D::new(dec!(1.0), dec!(1.0)));
        assert!(curve.contains_point(&dec!(1.0)));
    }

    #[test]
    fn test_curve_linear_interpolate_stacked_abscissa_is_degenerate() {
        let curve = stacked_curve();

        assert!(matches!(
            curve.interpolate(dec!(1.0), InterpolationType::Linear),
            Err(InterpolationError::DegenerateInterval)
        ));
    }

    #[test]
    fn test_curve_cubic_interpolate_stacked_abscissa_is_degenerate() {
        let curve = stacked_curve();

        assert!(matches!(
            curve.interpolate(dec!(1.0), InterpolationType::Cubic),
            Err(InterpolationError::DegenerateInterval)
        ));
    }

    #[test]
    fn test_curve_spline_interpolate_stacked_abscissa_is_degenerate() {
        let curve = stacked_curve();

        assert!(matches!(
            curve.interpolate(dec!(1.0), InterpolationType::Spline),
            Err(InterpolationError::DegenerateInterval)
        ));
    }

    /// Away from the stack the curve is still a function, and linear
    /// interpolation reads the ordinate on the same side of the jump.
    #[test]
    fn test_curve_linear_interpolate_away_from_stack_reads_its_own_side() {
        let curve = stacked_curve();

        let left = curve
            .interpolate(dec!(0.5), InterpolationType::Linear)
            .expect("0.5 brackets between (0, 0) and the lowest ordinate at x = 1");
        assert_eq!(left, Point2D::new(dec!(0.5), dec!(0.5)));

        let right = curve
            .interpolate(dec!(1.5), InterpolationType::Linear)
            .expect("1.5 brackets between the highest ordinate at x = 1 and (2, 2)");
        assert_eq!(right, Point2D::new(dec!(1.5), dec!(2.5)));
    }

    /// A curve honouring the rule is unaffected by the exact-match guard:
    /// asking for a stored abscissa returns the stored point.
    #[test]
    fn test_curve_interpolate_single_ordinate_returns_the_stored_point() {
        let curve = Curve::new(BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
            Point2D::new(dec!(3.0), dec!(9.0)),
        ]));

        for interpolation in [
            InterpolationType::Linear,
            InterpolationType::Cubic,
            InterpolationType::Spline,
        ] {
            let point = curve
                .interpolate(dec!(2.0), interpolation)
                .expect("x = 2 is a stored abscissa");
            assert_eq!(point, Point2D::new(dec!(2.0), dec!(4.0)));
        }
    }
}
