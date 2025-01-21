/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 21/1/25
 ******************************************************************************/
mod utils;
mod interpolation;

pub use utils::GeometricObject;
pub use interpolation::traits::Interpolate;
pub use interpolation::linear::LinearInterpolation;
pub use interpolation::bilinear::BiLinearInterpolation;
pub use interpolation::cubic::CubicInterpolation;
pub use interpolation::spline::SplineInterpolation;