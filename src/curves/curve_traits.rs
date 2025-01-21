/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/

use crate::curves::analysis::CurveAnalysisResult;
use crate::curves::types::CurveType;
use crate::curves::{Curve, Point2D};
use crate::error::CurvesError;
use crate::geometrics::{InterpolationType, MergeOperation};
use rust_decimal::Decimal;

/// The `CurveOperations` trait defines a comprehensive set of operations for mathematical curves.
///
/// # Methods
///
/// ## Core Operations
/// - `generate_curve<T: Into<Decimal>>(&self, x_values: Vec<T>, curve_type: CurveType) -> Result<Curve, CurvesError>`
///   Creates a new curve from x-values and specified type.
///
/// - `interpolate<T: Into<Decimal>>(&self, x: T, curve: &Curve, interpolation: InterpolationType) -> Option<T>`
///   Calculates curve value at point x using specified interpolation method.
///
/// - `analyze_curve(&self, curve: &Curve) -> CurveAnalysisResult`
///   Performs statistical analysis on the curve.
///
/// ## Curve Transformations
/// - `merge_curves(&self, curves: Vec<&Curve>, operation: MergeOperation) -> Result<Curve, CurvesError>`
///   Combines multiple curves using specified operation (add, subtract, multiply, etc.).
///
/// - `slice_curve(&self, curve: &Curve, x1: Decimal, x2: Decimal) -> Result<Curve, CurvesError>`
///   Extracts portion of curve between x1 and x2.
///
/// - `translate_curve(&self, curve: &Curve, dx: Decimal, dy: Decimal) -> Result<Curve, CurvesError>`
///   Shifts curve by dx horizontally and dy vertically.
///
/// - `scale_curve(&self, curve: &Curve, sx: Decimal, sy: Decimal) -> Result<Curve, CurvesError>`
///   Scales curve by factors sx and sy.
///
/// ## Analysis Operations
/// - `find_intersections(&self, curve1: &Curve, curve2: &Curve) -> Result<Vec<Point2D>, CurvesError>`
///   Locates points where two curves intersect.
///
/// - `derivative_at(&self, curve: &Curve, x: Decimal) -> Result<Decimal, CurvesError>`
///   Calculates first derivative at point x.
///
/// - `get_extrema(&self, curve: &Curve) -> Result<(Point2D, Point2D), CurvesError>`
///   Finds minimum and maximum points on curve.
///
/// - `area_under_curve(&self, curve: &Curve, x1: Decimal, x2: Decimal) -> Result<Decimal, CurvesError>`
///   Calculates definite integral between x1 and x2.
pub trait CurveOperations {
    fn generate_curve<T: Into<Decimal>>(
        &self,
        x_values: Vec<T>,
        curve_type: CurveType,
    ) -> Result<Curve, CurvesError>;

    fn interpolate<T: Into<Decimal>>(
        &self,
        x: T,
        curve: &Curve,
        interpolation: InterpolationType,
    ) -> Option<T>;

    fn analyze_curve(&self, curve: &Curve) -> CurveAnalysisResult;

    // Combines multiple curves into a single curve using the specified operation
    fn merge_curves(
        &self,
        curves: Vec<&Curve>,
        operation: MergeOperation,
    ) -> Result<Curve, CurvesError>;

    // Extracts a section of the curve between x1 and x2
    fn slice_curve(&self, curve: &Curve, x1: Decimal, x2: Decimal) -> Result<Curve, CurvesError>;

    // Shifts the curve by dx and dy
    fn translate_curve(
        &self,
        curve: &Curve,
        dx: Decimal,
        dy: Decimal,
    ) -> Result<Curve, CurvesError>;

    // Scales the curve by sx and sy factors
    fn scale_curve(&self, curve: &Curve, sx: Decimal, sy: Decimal) -> Result<Curve, CurvesError>;

    // Finds intersection points between two curves
    fn find_intersections(
        &self,
        curve1: &Curve,
        curve2: &Curve,
    ) -> Result<Vec<Point2D>, CurvesError>;

    // Calculates derivative of the curve at point x
    fn derivative_at(&self, curve: &Curve, x: Decimal) -> Result<Decimal, CurvesError>;

    // Gets minimum and maximum y values in curve
    fn get_extrema(&self, curve: &Curve) -> Result<(Point2D, Point2D), CurvesError>;

    // Calculates area under curve between x1 and x2
    fn area_under_curve(
        &self,
        curve: &Curve,
        x1: Decimal,
        x2: Decimal,
    ) -> Result<Decimal, CurvesError>;
}
