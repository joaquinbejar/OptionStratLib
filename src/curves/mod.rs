mod curve;
mod traits;
mod types;
mod utils;
pub mod visualization;

pub use crate::analytics::projections::BasicCurves; // facade-compat: analytics
pub use curve::Curve;
pub use traits::Curvable;
pub use traits::StatisticalCurve;
pub use types::Point2D;
pub use utils::{create_constant_curve, create_linear_curve};
