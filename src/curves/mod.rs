mod curve;
mod traits;
mod types;
mod utils;
pub mod visualization;

pub use curve::Curve;
pub use traits::CurveOperations;
pub use types::Point2D;
pub use utils::{create_constant_curve, create_linear_curve};
