/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
mod construction;
mod interpolation;
mod utils;

mod operations;
mod visualization;

mod analysis;

pub use analysis::{
    AnalysisResult, BasicMetrics, Metrics, MetricsExtractor, RangeMetrics, RiskMetrics,
    ShapeMetrics, TrendMetrics,
};
pub use construction::{ConstructionMethod, ConstructionParams, ResultPoint};
pub use interpolation::bilinear::BiLinearInterpolation;
pub use interpolation::cubic::CubicInterpolation;
pub use interpolation::linear::LinearInterpolation;
pub use interpolation::spline::SplineInterpolation;
pub use interpolation::traits::HasX;
pub use interpolation::traits::Interpolate;
pub use interpolation::types::InterpolationType;
pub use operations::{Arithmetic, AxisOperations, MergeAxisInterpolate, MergeOperation};
pub use utils::{GeometricObject, Len};
pub use visualization::{PlotBuilder, PlotBuilderExt, PlotOptions, Plottable};
