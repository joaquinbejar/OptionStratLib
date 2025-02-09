mod curve;
mod types;
mod utils;
pub mod visualization;
mod basic;

pub use curve::Curve;
pub use types::Point2D;
pub use utils::{create_constant_curve, create_linear_curve};
pub use basic::BasicCurves;