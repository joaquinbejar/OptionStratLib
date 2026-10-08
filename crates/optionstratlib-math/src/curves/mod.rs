mod curve;
mod traits;
mod types;
mod utils;

pub use curve::{Curve, CurveSpline};
pub use traits::Curvable;
pub use traits::StatisticalCurve;
pub use types::Point2D;
pub use utils::{create_constant_curve, create_linear_curve};
