/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
mod construction;
mod interpolation;
mod utils;

mod operations;

mod analysis;

pub use analysis::{
    AnalysisResult, BasicMetrics, Metrics, MetricsExtractor, RangeMetrics, RiskMetrics,
    ShapeMetrics, TrendMetrics,
};
pub use construction::{ConstructionMethod, ConstructionParams};
pub use interpolation::bilinear::BiLinearInterpolation;
pub use interpolation::cubic::CubicInterpolation;
pub use interpolation::linear::LinearInterpolation;
pub use interpolation::spline::SplineInterpolation;
pub use interpolation::traits::HasX;
pub use interpolation::traits::Interpolate;
pub use interpolation::types::InterpolationType;
pub use operations::{
    Arithmetic, AxisOperations, GeometricTransformations, MergeAxisInterpolate, MergeOperation,
};
pub use utils::GeometricObject;
pub(crate) use utils::powu_checked;

// `PlotBuilder` and `Plottable` are visualization-owned; the `geometrics` path
// is a 0.21 public path kept for compatibility.
pub use crate::visualization::{PlotBuilder, Plottable}; // facade-compat: visualization
