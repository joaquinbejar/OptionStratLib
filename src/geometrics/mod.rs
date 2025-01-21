/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 21/1/25
 ******************************************************************************/
mod utils;
mod interpolation;
mod construction;

mod operations;

pub use utils::{GeometricObject, Len};
pub use interpolation::traits::Interpolate;
pub use interpolation::linear::LinearInterpolation;
pub use interpolation::bilinear::BiLinearInterpolation;
pub use interpolation::cubic::CubicInterpolation;
pub use interpolation::spline::SplineInterpolation;
pub use interpolation::types::InterpolationType;
pub use construction::{ConstructionMethod, ConstructionParams};
pub use operations::{MergeOperation, CurveArithmetic};
pub use interpolation::traits::HasX;
