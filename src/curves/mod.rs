mod analysis;
pub mod construction;
mod curve_traits;
pub mod interpolation;
pub mod operations;
mod types;
mod visualization;
mod curve;
mod utils;

pub use types::Point2D;
pub use operations::arithmetic::MergeOperation;
pub use curve::Curve;