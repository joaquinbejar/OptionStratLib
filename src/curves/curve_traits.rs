/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::curves::analysis::statistics::CurveAnalysisResult;
use crate::curves::interpolation::types::InterpolationType;
use crate::curves::types::{Curve, CurveType};
use std::error::Error;

/// The `CurveOperations` trait defines a set of operations that can be performed on mathematical curves.
///
/// This trait includes methods for generating curves, interpolating values, and analyzing curves.
/// Implementors of this trait are expected to provide concrete implementations for these methods.
///
/// # Methods
///
/// - `generate_curve(&self, x_values: Vec<f64>, curve_type: CurveType) -> Result<Curve, Box<dyn Error>>`
///
///   Generates a curve based on the provided x-values and the specified curve type.
///
///   - `x_values`: A vector of floating-point numbers representing the x-values of the curve.
///   - `curve_type`: An enumeration indicating the type of curve to generate (e.g., linear, polynomial).
///   - Returns: A result containing a `Curve` object on success or an error on failure.
///
/// - `interpolate(&self, x: f64, curve: &Curve, interpolation: InterpolationType) -> Option<f64>`
///
///   Interpolates a value on the given curve at the specified x-coordinate.
///
///   - `x`: The x-coordinate at which to interpolate.
///   - `curve`: A reference to the curve on which interpolation is to be performed.
///   - `interpolation`: The type of interpolation to use (e.g., linear, spline).
///   - Returns: An optional floating-point number representing the interpolated y-value.
///
/// - `analyze_curve(&self, curve: &Curve) -> CurveAnalysisResult`
///
///   Analyzes the provided curve and returns a summary of the analysis.
///
///   - `curve`: A reference to the curve to be analyzed.
///   - Returns: A `CurveAnalysisResult` containing the results of the analysis.
///
#[allow(dead_code)]
pub trait CurveOperations {


    fn generate_curve(
        &self,
        x_values: Vec<f64>,
        curve_type: CurveType,
    ) -> Result<Curve, Box<dyn Error>>;
    fn interpolate(&self, x: f64, curve: &Curve, interpolation: InterpolationType) -> Option<f64>;
    fn analyze_curve(&self, curve: &Curve) -> CurveAnalysisResult;
}
