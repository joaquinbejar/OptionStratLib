/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 26/8/24
******************************************************************************/
use crate::curves::analysis::statistics::CurveAnalysisResult;
use crate::curves::interpolation::types::InterpolationType;
use crate::curves::types::{Curve, CurveType};
use std::error::Error;

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
