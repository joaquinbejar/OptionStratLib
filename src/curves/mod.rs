pub mod analysis;
mod curve;
mod curve_traits;
mod types;
mod utils;
pub mod visualization;

pub use curve::Curve;
pub use curve_traits::CurveOperations;
pub use types::Point2D;
pub use utils::{create_constant_curve, create_linear_curve};
